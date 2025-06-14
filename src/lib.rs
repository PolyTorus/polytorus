#![allow(non_snake_case)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::type_complexity)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::manual_async_fn)]
#![allow(clippy::clone_on_copy)]

// src/lib.rs
// Core modular blockchain - new primary architecture
pub mod modular;

// Diamond IO integration for advanced cryptographic operations
pub mod diamond_io_integration;
pub mod diamond_smart_contracts;

// Legacy modules - maintained for backward compatibility
pub mod blockchain;
pub mod command;
pub mod config;
pub mod crypto;
pub mod network;
pub mod smart_contract;
pub mod test_helpers;
pub mod webserver;

#[macro_use]
extern crate log;

pub type Result<T> = std::result::Result<T, failure::Error>;
