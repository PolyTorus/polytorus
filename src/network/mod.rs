//! Network module
//!
//! This module contains P2P networking functionality, blockchain integration,
//! network configuration management, network management, and message prioritization.

pub mod blockchain_integration;
pub mod message_priority;
pub mod network_config;
pub mod network_manager;
pub mod p2p_enhanced;

// Re-export commonly used types
pub use blockchain_integration::{BlockchainState, NetworkedBlockchainNode, SyncState};
pub use message_priority::{MessagePriority, PrioritizedMessage, PriorityMessageQueue};
pub use network_config::NetworkConfig;
pub use network_manager::{NetworkManager, NetworkManagerConfig, NodeHealth};
pub use p2p_enhanced::{EnhancedP2PNode, NetworkCommand, NetworkEvent, PeerId};
