#![allow(non_snake_case)]

// src/lib.rs
pub mod blockchain;
pub mod command;
pub mod crypto;
pub mod network;
pub mod webserver;
pub mod contract;

#[macro_use]
extern crate log;

pub type Result<T> = std::result::Result<T, failure::Error>;
