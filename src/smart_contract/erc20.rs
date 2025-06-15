//! ERC20 token standard implementation
//!
//! This module provides a complete ERC20 token implementation
//! following the Ethereum ERC20 standard specification.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::smart_contract::types::ContractResult;
use crate::Result;

/// ERC20 token events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ERC20Event {
    Transfer {
        from: String,
        to: String,
        value: u64,
    },
    Approval {
        owner: String,
        spender: String,
        value: u64,
    },
}

/// ERC20 contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERC20State {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>,
}

/// ERC20 contract implementation
#[derive(Debug, Clone)]
pub struct ERC20Contract {
    pub state: ERC20State,
    pub events: Vec<ERC20Event>,
}

impl ERC20Contract {
    /// Create a new ERC20 token contract
    pub fn new(
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u64,
        initial_owner: String,
    ) -> Self {
        let mut balances = HashMap::new();
        balances.insert(initial_owner.clone(), initial_supply);

        let state = ERC20State {
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
            balances,
            allowances: HashMap::new(),
        };

        let mut contract = Self {
            state,
            events: Vec::new(),
        };

        // Emit initial transfer event (from zero address)
        contract.events.push(ERC20Event::Transfer {
            from: "0x0000000000000000000000000000000000000000".to_string(),
            to: initial_owner,
            value: initial_supply,
        });

        contract
    }

    /// Get token name
    pub fn name(&self) -> &str {
        &self.state.name
    }

    /// Get token symbol
    pub fn symbol(&self) -> &str {
        &self.state.symbol
    }

    /// Get token decimals
    pub fn decimals(&self) -> u8 {
        self.state.decimals
    }

    /// Get total supply
    pub fn total_supply(&self) -> u64 {
        self.state.total_supply
    }

    /// Get balance of an account
    pub fn balance_of(&self, owner: &str) -> u64 {
        self.state.balances.get(owner).copied().unwrap_or(0)
    }

    /// Get allowance for spender from owner
    pub fn allowance(&self, owner: &str, spender: &str) -> u64 {
        self.state
            .allowances
            .get(owner)
            .and_then(|allowances| allowances.get(spender))
            .copied()
            .unwrap_or(0)
    }

    /// Transfer tokens from one account to another
    pub fn transfer(&mut self, from: &str, to: &str, value: u64) -> Result<ContractResult> {
        if from == to {
            return Ok(ContractResult {
                success: false,
                return_value: b"Cannot transfer to self".to_vec(),
                gas_used: 1000,
                logs: vec!["Transfer to self attempted".to_string()],
                state_changes: HashMap::new(),
            });
        }

        let from_balance = self.balance_of(from);
        if from_balance < value {
            return Ok(ContractResult {
                success: false,
                return_value: b"Insufficient balance".to_vec(),
                gas_used: 1000,
                logs: vec![format!("Insufficient balance: {} < {}", from_balance, value)],
                state_changes: HashMap::new(),
            });
        }

        // Update balances
        self.state.balances.insert(from.to_string(), from_balance - value);
        let to_balance = self.balance_of(to);
        self.state.balances.insert(to.to_string(), to_balance + value);

        // Emit transfer event
        self.events.push(ERC20Event::Transfer {
            from: from.to_string(),
            to: to.to_string(),
            value,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("balance_{}", from),
            (from_balance - value).to_le_bytes().to_vec(),
        );
        state_changes.insert(
            format!("balance_{}", to),
            (to_balance + value).to_le_bytes().to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 21000, // Standard ERC20 transfer gas cost
            logs: vec![format!("Transferred {} tokens from {} to {}", value, from, to)],
            state_changes,
        })
    }

    /// Approve spender to spend tokens on behalf of owner
    pub fn approve(&mut self, owner: &str, spender: &str, value: u64) -> Result<ContractResult> {
        if owner == spender {
            return Ok(ContractResult {
                success: false,
                return_value: b"Cannot approve self".to_vec(),
                gas_used: 1000,
                logs: vec!["Self approval attempted".to_string()],
                state_changes: HashMap::new(),
            });
        }

        // Set allowance
        self.state
            .allowances
            .entry(owner.to_string())
            .or_insert_with(HashMap::new)
            .insert(spender.to_string(), value);

        // Emit approval event
        self.events.push(ERC20Event::Approval {
            owner: owner.to_string(),
            spender: spender.to_string(),
            value,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("allowance_{}_{}", owner, spender),
            value.to_le_bytes().to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 46000, // Standard ERC20 approve gas cost
            logs: vec![format!("Approved {} tokens for {} by {}", value, spender, owner)],
            state_changes,
        })
    }

    /// Transfer tokens from one account to another on behalf of owner
    pub fn transfer_from(
        &mut self,
        spender: &str,
        from: &str,
        to: &str,
        value: u64,
    ) -> Result<ContractResult> {
        let allowance = self.allowance(from, spender);
        if allowance < value {
            return Ok(ContractResult {
                success: false,
                return_value: b"Insufficient allowance".to_vec(),
                gas_used: 1000,
                logs: vec![format!("Insufficient allowance: {} < {}", allowance, value)],
                state_changes: HashMap::new(),
            });
        }

        // Perform the transfer
        let transfer_result = self.transfer(from, to, value)?;
        if !transfer_result.success {
            return Ok(transfer_result);
        }

        // Update allowance
        self.state
            .allowances
            .get_mut(from)
            .unwrap()
            .insert(spender.to_string(), allowance - value);

        let mut state_changes = transfer_result.state_changes;
        state_changes.insert(
            format!("allowance_{}_{}", from, spender),
            (allowance - value).to_le_bytes().to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 34000, // Standard ERC20 transferFrom gas cost
            logs: vec![format!(
                "Transferred {} tokens from {} to {} by {}",
                value, from, to, spender
            )],
            state_changes,
        })
    }

    /// Increase allowance for a spender
    pub fn increase_allowance(&mut self, owner: &str, spender: &str, added_value: u64) -> Result<ContractResult> {
        let current_allowance = self.allowance(owner, spender);
        let new_allowance = current_allowance.saturating_add(added_value);
        
        self.approve(owner, spender, new_allowance)
    }

    /// Decrease allowance for a spender
    pub fn decrease_allowance(&mut self, owner: &str, spender: &str, subtracted_value: u64) -> Result<ContractResult> {
        let current_allowance = self.allowance(owner, spender);
        let new_allowance = current_allowance.saturating_sub(subtracted_value);
        
        self.approve(owner, spender, new_allowance)
    }

    /// Mint new tokens (only for token creators/admin)
    pub fn mint(&mut self, to: &str, value: u64) -> Result<ContractResult> {
        let current_balance = self.balance_of(to);
        self.state.balances.insert(to.to_string(), current_balance + value);
        self.state.total_supply += value;

        // Emit transfer event from zero address
        self.events.push(ERC20Event::Transfer {
            from: "0x0000000000000000000000000000000000000000".to_string(),
            to: to.to_string(),
            value,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("balance_{}", to),
            (current_balance + value).to_le_bytes().to_vec(),
        );
        state_changes.insert("total_supply".to_string(), self.state.total_supply.to_le_bytes().to_vec());

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 32000,
            logs: vec![format!("Minted {} tokens to {}", value, to)],
            state_changes,
        })
    }

    /// Burn tokens from an account
    pub fn burn(&mut self, from: &str, value: u64) -> Result<ContractResult> {
        let current_balance = self.balance_of(from);
        if current_balance < value {
            return Ok(ContractResult {
                success: false,
                return_value: b"Insufficient balance to burn".to_vec(),
                gas_used: 1000,
                logs: vec![format!("Insufficient balance to burn: {} < {}", current_balance, value)],
                state_changes: HashMap::new(),
            });
        }

        self.state.balances.insert(from.to_string(), current_balance - value);
        self.state.total_supply -= value;

        // Emit transfer event to zero address
        self.events.push(ERC20Event::Transfer {
            from: from.to_string(),
            to: "0x0000000000000000000000000000000000000000".to_string(),
            value,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("balance_{}", from),
            (current_balance - value).to_le_bytes().to_vec(),
        );
        state_changes.insert("total_supply".to_string(), self.state.total_supply.to_le_bytes().to_vec());

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 15000,
            logs: vec![format!("Burned {} tokens from {}", value, from)],
            state_changes,
        })
    }

    /// Get all events emitted by the contract
    pub fn get_events(&self) -> &[ERC20Event] {
        &self.events
    }

    /// Clear events (typically called after processing)
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erc20_creation() {
        let contract = ERC20Contract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        assert_eq!(contract.name(), "Test Token");
        assert_eq!(contract.symbol(), "TEST");
        assert_eq!(contract.decimals(), 18);
        assert_eq!(contract.total_supply(), 1000000);
        assert_eq!(contract.balance_of("alice"), 1000000);
        assert_eq!(contract.balance_of("bob"), 0);
    }

    #[test]
    fn test_transfer() {
        let mut contract = ERC20Contract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        let result = contract.transfer("alice", "bob", 100).unwrap();
        assert!(result.success);
        assert_eq!(contract.balance_of("alice"), 999900);
        assert_eq!(contract.balance_of("bob"), 100);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let mut contract = ERC20Contract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        // Alice approves Bob to spend 200 tokens
        let result = contract.approve("alice", "bob", 200).unwrap();
        assert!(result.success);
        assert_eq!(contract.allowance("alice", "bob"), 200);

        // Bob transfers 100 tokens from Alice to Charlie
        let result = contract.transfer_from("bob", "alice", "charlie", 100).unwrap();
        assert!(result.success);
        assert_eq!(contract.balance_of("alice"), 999900);
        assert_eq!(contract.balance_of("charlie"), 100);
        assert_eq!(contract.allowance("alice", "bob"), 100);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut contract = ERC20Contract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        let result = contract.transfer("bob", "alice", 100).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_mint_and_burn() {
        let mut contract = ERC20Contract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        // Mint 500 tokens to Bob
        let result = contract.mint("bob", 500).unwrap();
        assert!(result.success);
        assert_eq!(contract.balance_of("bob"), 500);
        assert_eq!(contract.total_supply(), 1000500);

        // Burn 200 tokens from Bob
        let result = contract.burn("bob", 200).unwrap();
        assert!(result.success);
        assert_eq!(contract.balance_of("bob"), 300);
        assert_eq!(contract.total_supply(), 1000300);
    }
}
