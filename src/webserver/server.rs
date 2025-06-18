//! Modern Web Server Implementation
//!
//! This module provides a comprehensive HTTP API server for the PolyTorus blockchain,
//! including wallet management, blockchain operations, smart contracts, and network monitoring.

use std::sync::Arc;

use actix_web::{middleware::Logger, web, App, HttpServer};
use tokio::sync::mpsc;

use crate::{
    config::DataContext,
    modular::{default_modular_config, UnifiedModularOrchestrator},
    network::NetworkCommand,
    webserver::{
        api::*,
        network_api::{NetworkApiState, *},
        simulation_api::*,
    },
    Result,
};

/// Configuration for the web server
#[derive(Debug, Clone)]
pub struct WebServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub enable_logging: bool,
    pub max_payload_size: usize,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7000,
            enable_cors: true,
            enable_logging: true,
            max_payload_size: 1024 * 1024, // 1MB
        }
    }
}

/// Main web server structure
pub struct WebServer {
    pub config: WebServerConfig,
    orchestrator: Option<Arc<UnifiedModularOrchestrator>>,
}

impl WebServer {
    /// Create a new web server with default configuration
    pub fn new() -> Self {
        Self {
            config: WebServerConfig::default(),
            orchestrator: None,
        }
    }

    /// Create a new web server with custom configuration
    pub fn with_config(config: WebServerConfig) -> Self {
        Self {
            config,
            orchestrator: None,
        }
    }

    /// Set the blockchain orchestrator for the web server
    pub fn with_orchestrator(mut self, orchestrator: Arc<UnifiedModularOrchestrator>) -> Self {
        self.orchestrator = Some(orchestrator);
        self
    }

    /// Run the web server
    pub async fn run(self) -> Result<()> {
        let bind_address = format!("{}:{}", self.config.host, self.config.port);
        println!("🌐 Starting PolyTorus Web Server on {}", bind_address);

        // Create network command channel
        let (network_tx, _network_rx) = mpsc::unbounded_channel::<NetworkCommand>();
        let network_api_state = Arc::new(NetworkApiState::new(network_tx));

        // Create simulation state for multi-node testing
        let simulation_state =
            SimulationState::new("webserver-node".to_string(), "./data/webserver".to_string());

        // Create orchestrator if not provided
        let orchestrator = if let Some(orch) = self.orchestrator {
            orch
        } else {
            let config = default_modular_config();
            let data_context = DataContext::default();
            data_context.ensure_directories()?;

            Arc::new(
                UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context)
                    .await?,
            )
        };

        println!("✅ Blockchain orchestrator initialized");
        println!("📡 Network API endpoints enabled");
        println!("🔄 Simulation API endpoints enabled");
        println!("💼 Wallet and blockchain API endpoints enabled");

        let config_clone = self.config.clone();
        let server = HttpServer::new(move || {
            // Build the base app
            let base_app = App::new()
                .app_data(web::Data::new(network_api_state.clone()))
                .app_data(web::Data::new(simulation_state.clone()))
                .app_data(web::Data::new(orchestrator.clone()))
                .app_data(web::PayloadConfig::new(config_clone.max_payload_size));

            // Apply middleware based on configuration - for simplicity, always enable both
            let app = base_app.wrap(Logger::default()).wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            );

            app
                // Health and status endpoints
                .route("/health", web::get().to(health_check))
                .route("/status", web::get().to(get_server_status))
                // Wallet management endpoints
                .route("/api/wallet/create", web::post().to(api_create_wallet))
                .route(
                    "/api/wallet/create/{encryption}",
                    web::post().to(api_create_wallet_with_type),
                )
                .route("/api/wallet/addresses", web::get().to(api_list_addresses))
                .route(
                    "/api/wallet/balance/{address}",
                    web::get().to(api_get_balance),
                )
                // Blockchain operations
                .route(
                    "/api/blockchain/status",
                    web::get().to(api_blockchain_status),
                )
                .route(
                    "/api/blockchain/config",
                    web::get().to(api_blockchain_config),
                )
                .route(
                    "/api/blockchain/metrics",
                    web::get().to(api_blockchain_metrics),
                )
                .route("/api/blockchain/layers", web::get().to(api_layer_status))
                // Smart contract endpoints
                .route("/api/contract/deploy", web::post().to(api_deploy_contract))
                .route("/api/contract/call", web::post().to(api_call_contract))
                .route("/api/contract/list", web::get().to(api_list_contracts))
                .route(
                    "/api/contract/{address}/state",
                    web::get().to(api_contract_state),
                )
                // ERC20 token endpoints
                .route("/api/erc20/deploy", web::post().to(api_erc20_deploy))
                .route("/api/erc20/transfer", web::post().to(api_erc20_transfer))
                .route(
                    "/api/erc20/{contract}/balance/{address}",
                    web::get().to(api_erc20_balance),
                )
                .route("/api/erc20/{contract}/info", web::get().to(api_erc20_info))
                .route("/api/erc20/list", web::get().to(api_erc20_list))
                // Governance endpoints
                .route(
                    "/api/governance/propose",
                    web::post().to(api_governance_propose),
                )
                .route("/api/governance/vote", web::post().to(api_governance_vote))
                .route(
                    "/api/governance/proposals",
                    web::get().to(api_governance_list),
                )
                // Network API endpoints
                .service(get_network_health)
                .service(get_peer_info)
                .service(get_message_queue_stats)
                .service(blacklist_peer)
                .service(unblacklist_peer)
                // Simulation API endpoints (for multi-node testing)
                .route("/transaction", web::post().to(submit_transaction))
                .route("/send", web::post().to(send_transaction))
                .route("/stats", web::get().to(get_stats))
                // Legacy endpoints (for backward compatibility)
                .route(
                    "/create_wallet/{encryption}",
                    web::post().to(legacy_create_wallet),
                )
                .route("/list-addresses", web::get().to(legacy_list_addresses))
        });

        let server = server
            .bind(&bind_address)
            .map_err(|e| anyhow::anyhow!("Failed to bind server to {}: {}", bind_address, e))?;

        println!("🚀 Web server started successfully!");
        println!("📋 Available endpoints:");
        println!("  Health: GET /health");
        println!("  Status: GET /status");
        println!("  Wallets: /api/wallet/*");
        println!("  Blockchain: /api/blockchain/*");
        println!("  Contracts: /api/contract/*");
        println!("  ERC20: /api/erc20/*");
        println!("  Governance: /api/governance/*");
        println!("  Network: /api/network/*");

        server
            .run()
            .await
            .map_err(|e| anyhow::anyhow!("Server runtime error: {}", e))?;

        Ok(())
    }

    /// Run the web server with a simple interface (for testing)
    pub async fn run_simple() -> std::io::Result<()> {
        let server = Self::new();
        server
            .run()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))
    }
}

impl Default for WebServer {
    fn default() -> Self {
        Self::new()
    }
}
