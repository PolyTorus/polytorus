//! Network module for P2P blockchain communication
//!
//! This module provides modern libp2p-based networking for blockchain nodes.
//! The P2P implementation supports various deployment environments including
//! local development, cloud, and distributed networks.

pub mod network_config; // Generic network configuration
pub mod p2p; // libp2p-based networking
pub mod p2p_tests;

// Re-export commonly used types
pub use network_config::NetworkConfig;
pub use p2p::{NetworkCommand, NetworkEvent};
