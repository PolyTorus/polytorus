//! Network module
//!
//! This module contains P2P networking functionality, blockchain integration,
//! and network configuration management.

pub mod blockchain_integration;
pub mod network_config;
pub mod p2p_enhanced;

// Re-export commonly used types
pub use blockchain_integration::{BlockchainState, NetworkedBlockchainNode, SyncState};
pub use network_config::NetworkConfig;
pub use p2p_enhanced::{EnhancedP2PNode, NetworkCommand, NetworkEvent, PeerId};
