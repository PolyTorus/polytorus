//! Webserver module
//!
//! This module contains web server functionality.

pub mod server;
pub mod createwallet;
pub mod listaddresses;
pub mod printchain;
pub mod reindex;
pub mod startminer;
pub mod startnode;

// Re-export commonly used types
pub use server::*;
