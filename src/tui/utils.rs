//! Utility functions for the TUI

use std::fmt;

#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: String,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Confirmed => write!(f, "Confirmed"),
            TransactionStatus::Failed => write!(f, "Failed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalletInfo {
    pub address: String,
    pub balance: u64,
    pub label: Option<String>,
}

impl WalletInfo {
    pub fn new(address: String, balance: u64) -> Self {
        Self {
            address,
            balance,
            label: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn display_name(&self) -> &str {
        self.label.as_ref().unwrap_or(&self.address)
    }
}

pub fn format_balance(amount: u64) -> String {
    let btc_amount = amount as f64 / 100_000_000.0;
    if btc_amount == 0.0 {
        "0 satoshi".to_string()
    } else if btc_amount < 0.00000001 {
        format!("{} satoshi", amount)
    } else {
        format!("{:.8} BTC", btc_amount)
    }
}

pub fn format_address(address: &str, max_len: usize) -> String {
    if address.len() <= max_len {
        address.to_string()
    } else {
        let start_len = (max_len - 3) / 2;
        let end_len = max_len - 3 - start_len;
        format!(
            "{}...{}",
            &address[..start_len],
            &address[address.len() - end_len..]
        )
    }
}

pub fn format_timestamp(timestamp: &str) -> String {
    // For now, just return the timestamp as-is
    // In a real implementation, you'd parse and format it nicely
    timestamp.to_string()
}

pub fn validate_address(address: &str) -> bool {
    // Basic address validation - in a real implementation this would be more sophisticated
    !address.is_empty() && address.len() >= 26 && address.len() <= 62
}

pub fn validate_amount(amount_str: &str) -> Result<u64, String> {
    if amount_str.is_empty() {
        return Err("Amount cannot be empty".to_string());
    }

    match amount_str.parse::<f64>() {
        Ok(amount) if amount <= 0.0 => Err("Amount must be positive".to_string()),
        Ok(amount) => {
            let satoshis = (amount * 100_000_000.0) as u64;
            if satoshis == 0 {
                Err("Amount too small".to_string())
            } else {
                Ok(satoshis)
            }
        }
        Err(_) => Err("Invalid amount format".to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub block_height: u64,
    pub is_syncing: bool,
    pub network_hash_rate: String,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            connected_peers: 0,
            block_height: 0,
            is_syncing: false,
            network_hash_rate: "0 H/s".to_string(),
        }
    }
}
