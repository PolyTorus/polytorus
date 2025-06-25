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

// Unified smart contract architecture
pub mod privacy_engine;
pub mod unified_engine;
pub mod unified_manager;
pub mod unified_storage;
pub mod wasm_engine;

// Advanced storage and engine implementations
pub mod database_storage;
pub mod enhanced_unified_engine;

#[cfg(test)]
pub mod test_utils;

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
