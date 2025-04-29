#![allow(non_snake_case)]

// src/lib.rs
pub mod blockchain;
pub mod command;
pub mod crypto;
pub mod network;
pub mod webserver;
pub mod config;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub type Result<T> = std::result::Result<T, failure::Error>;
