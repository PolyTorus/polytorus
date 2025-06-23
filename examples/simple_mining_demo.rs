//! Simple Mining Demo for PolyTorus
//!
//! This is a simplified version that demonstrates basic mining functionality
//! without complex ContainerLab dependencies.

use std::{sync::Arc, time::Duration};
use tokio::time::{interval, sleep};
use polytorus::{
    config::DataContext,
    modular::{
        default_modular_config, UnifiedModularOrchestrator,
    },
    crypto::wallets::Wallets,
    crypto::types::EncryptionType,
    Result,
};

#[derive(Clone)]
pub struct SimpleMiner {
    pub node_id: String,
    pub orchestrator: Arc<UnifiedModularOrchestrator>,
    pub mining_address: String,
}

pub struct SimpleMiningDemo {
    miners: Vec<SimpleMiner>,
    simulation_duration: u64,
}

impl SimpleMiningDemo {
    pub fn new(num_miners: usize, simulation_duration: u64) -> Self {
        Self {
            miners: Vec::with_capacity(num_miners),
            simulation_duration,
        }
    }

    pub async fn setup_miners(&mut self, num_miners: usize) -> Result<()> {
        println!("🔧 Setting up {} miners...", num_miners);

        for i in 0..num_miners {
            let node_id = format!("miner-{}", i);
            let data_context = DataContext::new(format!("./data/simple_mining/{}", node_id).into());
            data_context.ensure_directories()?;

            // Create mining wallet
            let mut wallets = Wallets::new_with_context(data_context.clone())?;
            let mining_address = wallets.create_wallet(EncryptionType::ECDSA);
            wallets.save_all()?;

            // Create orchestrator
            let config = default_modular_config();
            let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(
                config,
                data_context,
            ).await?;

            let miner = SimpleMiner {
                node_id: node_id.clone(),
                orchestrator: Arc::new(orchestrator),
                mining_address: mining_address.clone(),
            };

            self.miners.push(miner);
            
            println!("   ✅ Miner {} created with address: {}", node_id, mining_address);
            sleep(Duration::from_millis(1000)).await;
        }

        Ok(())
    }

    pub async fn start_mining_simulation(&self) -> Result<()> {
        println!("⛏️  Starting mining simulation for {} seconds...", self.simulation_duration);

        let mut tasks = Vec::new();

        // Start mining task for each miner
        for (i, miner) in self.miners.iter().enumerate() {
            let miner_clone = miner.clone();
            let mining_interval = 15000 + (i as u64 * 2000); // Stagger mining attempts

            let task = tokio::spawn(async move {
                let mut interval = interval(Duration::from_millis(mining_interval));
                let mut blocks_mined = 0u64;

                for block_number in 0..10 { // Mine up to 10 blocks
                    interval.tick().await;

                    match Self::attempt_mining(&miner_clone, block_number).await {
                        Ok(success) => {
                            if success {
                                blocks_mined += 1;
                                println!(
                                    "   ⛏️  {} successfully mined block #{} (total: {})",
                                    miner_clone.node_id, block_number, blocks_mined
                                );
                            } else {
                                println!(
                                    "   ⏭️  {} mining attempt {} failed (normal)",
                                    miner_clone.node_id, block_number
                                );
                            }
                        }
                        Err(e) => {
                            println!(
                                "   ❌ {} mining error on block {}: {}",
                                miner_clone.node_id, block_number, e
                            );
                        }
                    }
                }

                println!("   🏁 {} finished mining with {} blocks", miner_clone.node_id, blocks_mined);
                blocks_mined
            });

            tasks.push(task);
        }

        // Start transaction generation in background
        let miners_clone = self.miners.clone();
        let tx_task = tokio::spawn(async move {
            Self::generate_transactions_static(&miners_clone).await.unwrap_or(());
            0u64 // Return 0 to match the expected type
        });
        tasks.push(tx_task);

        // Wait for simulation duration or all tasks to complete
        let duration = self.simulation_duration;
        let timeout_task = tokio::spawn(async move {
            sleep(Duration::from_secs(duration)).await;
            println!("⏰ Simulation time limit reached");
        });

        // Wait for either timeout or all mining tasks to complete
        tokio::select! {
            _ = timeout_task => {
                println!("⏹️  Simulation stopped due to timeout");
            }
            results = futures::future::join_all(tasks) => {
                let total_blocks: u64 = results.iter()
                    .filter_map(|r| r.as_ref().ok())
                    .sum();
                println!("✅ All mining tasks completed. Total blocks mined: {}", total_blocks);
            }
        }

        Ok(())
    }

    async fn attempt_mining(miner: &SimpleMiner, block_number: u64) -> Result<bool> {
        // Simulate mining work
        println!(
            "     🔨 {} attempting to mine block #{}...",
            miner.node_id, block_number
        );

        // Get current state
        let state = miner.orchestrator.get_state().await;
        
        // Simulate proof-of-work (in real implementation, this would be actual mining)
        let mining_success = (block_number + state.current_block_height) % 3 == 0; // 33% success rate
        
        if mining_success {
            // Simulate adding the block to the chain
            sleep(Duration::from_millis(500)).await; // Simulate block processing time
            
            println!(
                "     ✨ {} found valid proof for block #{}!",
                miner.node_id, block_number
            );
            return Ok(true);
        }

        Ok(false)
    }

    async fn generate_transactions_static(miners: &[SimpleMiner]) -> Result<()> {
        println!("💸 Starting transaction generation...");
        
        let mut tx_count = 0u64;
        let mut interval = interval(Duration::from_secs(5));

        for _ in 0..20 { // Generate 20 transactions
            interval.tick().await;

            if miners.len() >= 2 {
                let from_idx = tx_count as usize % miners.len();
                let to_idx = (tx_count as usize + 1) % miners.len();
                
                let from_miner = &miners[from_idx];
                let to_miner = &miners[to_idx];
                
                let amount = 100 + (tx_count % 900);
                
                println!(
                    "   💸 TX {}: {} -> {} ({} units)",
                    tx_count, from_miner.node_id, to_miner.node_id, amount
                );
                
                tx_count += 1;
            }
        }

        println!("📊 Transaction generation completed: {} transactions", tx_count);
        Ok(())
    }

    pub async fn show_final_stats(&self) {
        println!("\n📈 Mining Simulation Results:");
        println!("============================");
        
        for miner in &self.miners {
            let state = miner.orchestrator.get_state().await;
            println!(
                "📊 {}: Block height: {}, Running: {}",
                miner.node_id, state.current_block_height, state.is_running
            );
        }
        
        println!("\n🎯 Simulation completed successfully!");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("⛏️  PolyTorus Simple Mining Demo");
    println!("================================");

    let num_miners = 3;
    let duration = 120; // 2 minutes

    println!("📊 Configuration:");
    println!("   Miners: {}", num_miners);
    println!("   Duration: {}s", duration);
    println!();

    let mut demo = SimpleMiningDemo::new(num_miners, duration);

    // Setup miners
    demo.setup_miners(num_miners).await?;
    
    println!("\n🚀 Starting mining simulation...");
    
    // Run simulation
    demo.start_mining_simulation().await?;
    
    // Show results
    demo.show_final_stats().await;

    Ok(())
}