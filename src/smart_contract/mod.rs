//! Smart contract module
//!
//! This module contains smart contract functionality.

pub mod contract;
pub mod engine;
pub mod erc20;
pub mod state;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use contract::*;
pub use engine::*;
pub use erc20::*;
pub use state::*;
pub use types::*;
