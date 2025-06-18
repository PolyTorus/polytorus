//! Webserver module
//!
//! This module contains web server functionality including modern REST API endpoints,
//! legacy compatibility endpoints, network management, and simulation capabilities.

pub mod api;
pub mod createwallet;
pub mod listaddresses;
pub mod network_api;
pub mod printchain;
pub mod reindex;
pub mod server;
pub mod simulation_api;
pub mod startminer;
pub mod startnode;

#[cfg(test)]
pub mod tests;

// Re-export commonly used types
pub use api::*;
pub use network_api::*;
pub use server::*;
pub use simulation_api::*;
