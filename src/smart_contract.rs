//! Smart Contract execution engine using WASM
//!
//! This module provides functionality to execute WASM-based smart contracts
//! on the blockchain. It includes contract deployment, execution, and state management.

pub mod contract;
pub mod engine;
pub mod state;
pub mod types;

pub use contract::SmartContract;
pub use engine::ContractEngine;
pub use state::ContractState;
pub use types::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod advanced_tests;
