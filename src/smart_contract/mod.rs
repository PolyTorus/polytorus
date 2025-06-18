//! Smart contract module
//!
//! This module contains smart contract functionality.

pub mod contract;
pub mod engine;
pub mod erc20;
pub mod governance_token;
pub mod proposal_manager;
pub mod state;
pub mod types;
pub mod voting_system;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use contract::*;
pub use engine::*;
pub use erc20::*;
pub use governance_token::*;
pub use proposal_manager::*;
pub use state::*;
pub use types::*;
pub use voting_system::*;
