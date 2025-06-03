//! Network module for P2P blockchain communication
//!
//! This module provides both legacy TCP-based networking and modern libp2p-based
//! networking for blockchain nodes. The P2P implementation supports various deployment
//! environments including local development, cloud, and distributed networks.

pub mod server; // Legacy TCP-based server
pub mod tests;
pub mod p2p; // libp2p-based networking
pub mod network_config; // Generic network configuration
pub mod manager; // High-level network manager
pub mod p2p_tests; // Tests for P2P networking

// Re-export commonly used types
pub use manager::{NetworkManager, NetworkStats, PeerStatus};
pub use network_config::NetworkConfig;
pub use p2p::{NetworkEvent, NetworkCommand};
