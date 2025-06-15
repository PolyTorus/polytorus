//! Blockchain module
//!
//! This module contains the core blockchain functionality.

pub mod block;
pub mod types;

#[cfg(kani)]
pub mod kani_verification;

// Re-export commonly used types
pub use block::{Block, FinalizedBlock};
pub use types::*;
