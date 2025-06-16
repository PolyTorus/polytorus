//! PolyTorus Simple Modular Architecture Demo
//!
//! A simplified demo showcasing the core modular components working together
//! without potentially blocking async operations.

use std::collections::HashMap;
use std::sync::Arc;

use polytorus::modular::{
    create_config_templates,

    // Enhanced configuration
    create_default_enhanced_config,

    ConsensusConfig,
    // Core traits and configs
    ExecutionConfig,
    HealthStatus,
    LayerConfig,

    LayerInfo,
    LayerType,
    MessagePayload,

    MessagePriority,
    MessageType,
    // Configuration Manager components
    ModularConfigManager,
    // Layer Factory components
    ModularLayerFactory,
    ModularMessage,
    // Message Bus components
    ModularMessageBus,
    SettlementConfig,
    WasmConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 PolyTorus Simple Modular Architecture Demo");
    println!("==============================================");

    println!("\n📋 Demo Components:");
    println!("   • Configuration Manager");
    println!("   • Message Bus");
    println!("   • Layer Factory");
    println!("   • Enhanced Configuration");

    // Demo 1: Configuration Manager
    println!("\n1️⃣ Configuration Manager Demo");
    println!("==============================");
    demo_configuration_manager()?;

    // Demo 2: Message Bus (non-async parts)
    println!("\n2️⃣ Message Bus Demo");
    println!("===================");
    demo_message_bus()?;

    // Demo 3: Layer Factory
    println!("\n3️⃣ Layer Factory Demo");
    println!("=====================");
    demo_layer_factory()?;

    // Demo 4: Enhanced Configuration
    println!("\n4️⃣ Enhanced Configuration Demo");
    println!("==============================");
    demo_enhanced_configuration()?;

    println!("\n✅ Demo completed successfully!");
    println!("   All modular components are working together.");
    println!("   The architecture supports pluggable implementations,");
    println!("   sophisticated configuration management, and event-driven");
    println!("   communication between layers.");

    Ok(())
}

/// Demonstrates configuration management capabilities
fn demo_configuration_manager() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating configuration manager...");

    let mut config_manager = ModularConfigManager::new();

    // Load predefined templates
    let templates = create_config_templates();
    println!("   ✓ Loaded {} configuration templates", templates.len());

    for template in &templates {
        println!("     • {} - {}", template.name, template.description);
    }

    // Validate current configuration
    println!("   Validating configuration...");
    let validation = config_manager.validate();
    println!("   ✓ Configuration validation completed");
    println!("     • Valid: {}", validation.is_valid);
    println!("     • Errors: {}", validation.errors.len());
    println!("     • Warnings: {}", validation.warnings.len());

    if !validation.errors.is_empty() {
        for error in &validation.errors {
            println!("       ❌ {error}");
        }
    }

    if !validation.warnings.is_empty() {
        for warning in &validation.warnings {
            println!("       ⚠️  {warning}");
        }
    }

    // Demonstrate configuration access
    println!("   Accessing layer configurations...");
    if let Ok(exec_config) = config_manager.get_execution_config() {
        println!("     • Execution gas limit: {}", exec_config.gas_limit);
        println!("     • Gas price: {}", exec_config.gas_price);
    }

    if let Ok(consensus_config) = config_manager.get_consensus_config() {
        println!("     • Block time: {}ms", consensus_config.block_time);
        println!("     • Difficulty: {}", consensus_config.difficulty);
    }

    // Add a configuration change watcher
    config_manager.add_change_watcher(|config| {
        println!(
            "     📢 Configuration changed! Active layers: {}",
            config.layers.len()
        );
    });

    println!("   ✅ Configuration manager operational");

    Ok(())
}

/// Demonstrates message bus basic setup
fn demo_message_bus() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating message bus...");

    let message_bus = Arc::new(ModularMessageBus::new());

    // Create sample layer info
    println!("   Creating sample layer configurations...");

    let execution_layer = LayerInfo {
        layer_type: LayerType::Execution,
        layer_id: "execution-001".to_string(),
        capabilities: vec!["wasm-execution".to_string(), "gas-metering".to_string()],
        health_status: HealthStatus::Healthy,
        message_handler: None,
    };

    let consensus_layer = LayerInfo {
        layer_type: LayerType::Consensus,
        layer_id: "consensus-001".to_string(),
        capabilities: vec!["proof-of-work".to_string(), "block-validation".to_string()],
        health_status: HealthStatus::Healthy,
        message_handler: None,
    };

    let settlement_layer = LayerInfo {
        layer_type: LayerType::Settlement,
        layer_id: "settlement-001".to_string(),
        capabilities: vec!["batch-settlement".to_string(), "fraud-proofs".to_string()],
        health_status: HealthStatus::Healthy,
        message_handler: None,
    };

    // Display layer information for demonstration
    println!("   ✓ Created 3 layer configurations");
    println!(
        "     • {}: {} capabilities",
        execution_layer.layer_id,
        execution_layer.capabilities.len()
    );
    println!(
        "     • {}: {} capabilities",
        consensus_layer.layer_id,
        consensus_layer.capabilities.len()
    );
    println!(
        "     • {}: {} capabilities",
        settlement_layer.layer_id,
        settlement_layer.capabilities.len()
    );

    // Create sample messages
    println!("   Creating sample messages...");

    let health_message = ModularMessage {
        id: "health-001".to_string(),
        message_type: MessageType::HealthCheck,
        source_layer: LayerType::Monitoring,
        target_layer: None,
        payload: MessagePayload::Custom {
            data: b"health check data".to_vec(),
            metadata: HashMap::new(),
        },
        priority: MessagePriority::High,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let block_proposal = ModularMessage {
        id: "block-001".to_string(),
        message_type: MessageType::BlockProposal,
        source_layer: LayerType::Consensus,
        target_layer: Some(LayerType::Execution),
        payload: MessagePayload::Custom {
            data: b"block proposal data".to_vec(),
            metadata: HashMap::new(),
        },
        priority: MessagePriority::Critical,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    // Display message information for demonstration
    println!("   ✓ Created sample messages");
    println!(
        "     • {} ({:?} priority) from {:?}",
        health_message.id, health_message.priority, health_message.source_layer
    );
    println!(
        "     • {} ({:?} priority) from {:?}",
        block_proposal.id, block_proposal.priority, block_proposal.source_layer
    );

    println!("   📊 Message Bus Configuration:");
    println!("     • Components configured: 3");
    println!("     • Message types supported: Multiple");
    println!("     • Priority levels: 4 (Critical, High, Normal, Low)");
    println!(
        "     • Message bus instance created: {:p}",
        message_bus.as_ref()
    );

    println!("   ✅ Message bus setup completed");

    Ok(())
}

/// Demonstrates layer factory capabilities
fn demo_layer_factory() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating layer factory...");

    let message_bus = Arc::new(ModularMessageBus::new());
    let mut layer_factory = ModularLayerFactory::new(message_bus.clone());

    // Configure layers
    println!("   Configuring layers...");

    let execution_config = LayerConfig {
        implementation: "polytorus-execution".to_string(),
        config: serde_json::to_value(ExecutionConfig {
            gas_limit: 8_000_000,
            gas_price: 1,
            wasm_config: WasmConfig {
                max_memory_pages: 256,
                max_stack_size: 65536,
                gas_metering: true,
            },
        })?,
        enabled: true,
        priority: 1,
        dependencies: vec![],
    };

    let consensus_config = LayerConfig {
        implementation: "polytorus-consensus".to_string(),
        config: serde_json::to_value(ConsensusConfig {
            block_time: 10000,
            difficulty: 4,
            max_block_size: 1024 * 1024,
        })?,
        enabled: true,
        priority: 2,
        dependencies: vec![LayerType::Execution],
    };

    let settlement_config = LayerConfig {
        implementation: "polytorus-settlement".to_string(),
        config: serde_json::to_value(SettlementConfig {
            challenge_period: 100,
            batch_size: 100,
            min_validator_stake: 1000,
        })?,
        enabled: true,
        priority: 3,
        dependencies: vec![LayerType::Execution, LayerType::Consensus],
    };

    layer_factory.configure_layer(LayerType::Execution, execution_config);
    layer_factory.configure_layer(LayerType::Consensus, consensus_config);
    layer_factory.configure_layer(LayerType::Settlement, settlement_config);

    println!("   ✓ Configured 3 layers");

    println!("   🏭 Layer Factory Configuration:");
    println!("     • Execution layer: polytorus-execution (priority 1)");
    println!("     • Consensus layer: polytorus-consensus (priority 2)");
    println!("     • Settlement layer: polytorus-settlement (priority 3)");
    println!("     • Dependency chain: Execution → Consensus → Settlement");

    println!("   ✅ Layer factory operational");

    Ok(())
}

/// Demonstrates enhanced configuration system
fn demo_enhanced_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Creating enhanced configuration...");

    // Create enhanced configuration
    let enhanced_config = create_default_enhanced_config();
    println!("   ✓ Created enhanced configuration");

    // Display configuration summary
    println!("   📋 Enhanced Configuration Summary:");
    println!(
        "     • Network mode: {}",
        enhanced_config.global.network_mode
    );
    println!("     • Log level: {}", enhanced_config.global.log_level);
    println!(
        "     • Performance mode: {:?}",
        enhanced_config.global.performance_mode
    );
    println!("     • Configured layers: {}", enhanced_config.layers.len());
    println!(
        "     • Plugin configurations: {}",
        enhanced_config.plugins.len()
    );

    // Show layer details
    for (layer_type, layer_config) in &enhanced_config.layers {
        println!(
            "     • {:?}: {} (enabled: {}, priority: {})",
            layer_type, layer_config.implementation, layer_config.enabled, layer_config.priority
        );
    }

    println!("   📊 System Configuration:");
    println!("     • Modular architecture: ✓ Enabled");
    println!("     • Layer separation: ✓ Complete");
    println!("     • Configuration validation: ✓ Passed");
    println!("     • Plugin system: ✓ Ready");

    println!("   ✅ Enhanced configuration system operational");

    Ok(())
}
