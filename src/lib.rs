//! # PolyTorus - Post-Quantum Modular Blockchain Platform
//!
//! PolyTorus is a cutting-edge modular blockchain platform designed for the post-quantum era.
//! It features a sophisticated modular architecture with separate layers for consensus, execution,
//! settlement, and data availability, along with Diamond IO integration for indistinguishability obfuscation.
//!
//! ## Core Architecture
//!
//! The platform is built around a **modular design** where each layer can be independently
//! developed, tested, and deployed:
//!
//! * **✅ Consensus Layer**: Fully implemented PoW consensus with comprehensive validation
//! * **✅ Data Availability Layer**: Sophisticated Merkle proof system with 15 comprehensive tests  
//! * **✅ Settlement Layer**: Working optimistic rollup with fraud proofs and 13 tests
//! * **⚠️ Execution Layer**: Hybrid account/eUTXO model (needs more tests)
//! * **⚠️ Unified Orchestrator**: Event-driven coordination (needs integration tests)
//!
//! ## Key Features
//!
//! ### 🔒 Post-Quantum Cryptography
//! - **FN-DSA**: Quantum-resistant digital signatures
//! - **Diamond IO**: Indistinguishability obfuscation for privacy
//! - **Verkle Trees**: Efficient cryptographic accumulators
//!
//! ### 🏗️ Modular Architecture  
//! - **Layer Separation**: Independent development and optimization
//! - **Pluggable Components**: Trait-based interfaces for flexibility
//! - **Event-Driven Communication**: Sophisticated message bus system
//!
//! ### 🚀 Performance & Scalability
//! - **Optimistic Rollups**: Batch processing with fraud proofs
//! - **Parallel Processing**: Concurrent layer operation
//! - **Efficient Storage**: RocksDB-based modular storage
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use polytorus::modular::default_modular_config;
//! use polytorus::config::DataContext;
//! use std::path::PathBuf;
//!
//! // Initialize with default configuration
//! let config = default_modular_config();
//! let data_context = DataContext::new(PathBuf::from("blockchain_data"));
//!
//! println!("PolyTorus modular blockchain configuration ready!");
//! ```
//!
//! ## Module Organization
//!
//! - [`modular`] - Core modular blockchain architecture (primary implementation)
//! - [`diamond_io_integration`] - Privacy layer with indistinguishability obfuscation  
//! - [`crypto`] - Cryptographic primitives (ECDSA, FN-DSA, Verkle trees)
//! - [`network`] - P2P networking with priority queues and health monitoring
//! - [`smart_contract`] - WASM smart contract engine with ERC20 support
//! - [`blockchain`] - Legacy blockchain implementation (maintained for compatibility)
//!

#![allow(non_snake_case)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::type_complexity)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::manual_async_fn)]
#![allow(clippy::clone_on_copy)]

// Core modular blockchain - new primary architecture
pub mod modular;

// Diamond IO integration
pub mod diamond_io_integration;
pub mod diamond_io_integration_new;
pub mod diamond_smart_contracts;

// Legacy modules - maintained for backward compatibility
pub mod blockchain;
pub mod command;
pub mod config;
pub mod crypto;
pub mod network;
pub mod smart_contract;
pub mod test_helpers;
pub mod tui;
pub mod webserver;

// Kani verification utilities
#[cfg(kani)]
pub mod kani_macros;

#[cfg(kani)]
pub mod simple_kani_tests;

#[cfg(kani)]
pub mod basic_kani_test;

#[macro_use]
extern crate log;

pub type Result<T> = std::result::Result<T, anyhow::Error>;
