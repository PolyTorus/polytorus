//! Network module
//! 
//! This module contains P2P networking functionality, blockchain integration,
//! and network configuration management.

pub mod p2p_enhanced;
pub mod blockchain_integration;
pub mod network_config;

// Re-export commonly used types
pub use p2p_enhanced::{EnhancedP2PNode, NetworkEvent, NetworkCommand, PeerId};
pub use blockchain_integration::{NetworkedBlockchainNode, BlockchainState, SyncState};
pub use network_config::NetworkConfig;
