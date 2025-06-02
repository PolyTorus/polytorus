#![allow(non_snake_case)]

// src/lib.rs
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
