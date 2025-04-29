#![allow(non_snake_case)]

// src/lib.rs
pub mod blockchain;
pub mod command;
pub mod crypto;
pub mod network;
pub mod webserver;

#[macro_use]
extern crate log;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[cfg(test)]
pub mod test_utils {
    use lazy_static::lazy_static;
    use std::sync::Mutex;
    
    // テスト実行中のブロックチェーン操作をロック
    lazy_static! {
        pub static ref BLOCKCHAIN_MUTEX: Mutex<()> = Mutex::new(());
        pub static ref WALLET_MUTEX: Mutex<()> = Mutex::new(());
        pub static ref UTXO_MUTEX: Mutex<()> = Mutex::new(());
    }
    
    // ファイルパス生成用のカウンタ
    use std::sync::atomic::{AtomicUsize, Ordering};
    pub static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    // テスト用の一意なIDを生成
    pub fn get_test_id() -> usize {
        TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}
