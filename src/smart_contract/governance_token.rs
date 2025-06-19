//! Governance Token Engine implementation
//!
//! This module provides a comprehensive governance token system
//! with voting power delegation and snapshot capabilities.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{smart_contract::types::ContractResult, Result};

/// Governance token events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceEvent {
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
    DelegateChanged {
        delegator: String,
        from_delegate: String,
        to_delegate: String,
    },
    DelegateVotesChanged {
        delegate: String,
        previous_balance: u64,
        new_balance: u64,
    },
    Snapshot {
        id: u64,
        block_number: u64,
    },
}

/// Checkpoint for tracking voting power over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub from_block: u64,
    pub votes: u64,
}

/// Governance token state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceTokenState {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>,
    pub delegates: HashMap<String, String>,
    pub checkpoints: HashMap<String, Vec<Checkpoint>>,
    pub num_checkpoints: HashMap<String, u32>,
    pub current_snapshot_id: u64,
    pub snapshots: HashMap<u64, HashMap<String, u64>>, // snapshot_id -> balances
    pub current_block: u64,
}

/// Governance token contract implementation
#[derive(Debug, Clone)]
pub struct GovernanceTokenContract {
    pub state: GovernanceTokenState,
    pub events: Vec<GovernanceEvent>,
}

impl GovernanceTokenContract {
    /// Create a new governance token contract
    pub fn new(
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u64,
        initial_owner: String,
    ) -> Self {
        let mut balances = HashMap::new();
        balances.insert(initial_owner.clone(), initial_supply);

        let state = GovernanceTokenState {
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
            balances,
            allowances: HashMap::new(),
            delegates: HashMap::new(),
            checkpoints: HashMap::new(),
            num_checkpoints: HashMap::new(),
            current_snapshot_id: 0,
            snapshots: HashMap::new(),
            current_block: 1,
        };

        let mut contract = Self {
            state,
            events: Vec::new(),
        };

        // Emit initial transfer event
        contract.events.push(GovernanceEvent::Transfer {
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
                logs: vec![format!(
                    "Insufficient balance: {} < {}",
                    from_balance, value
                )],
                state_changes: HashMap::new(),
            });
        }

        // Update balances
        self.state
            .balances
            .insert(from.to_string(), from_balance - value);
        let to_balance = self.balance_of(to);
        self.state
            .balances
            .insert(to.to_string(), to_balance + value);

        // Update voting power
        self.move_voting_power(from, to, value);

        // Emit transfer event
        self.events.push(GovernanceEvent::Transfer {
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
            gas_used: 25000,
            logs: vec![format!(
                "Transferred {} tokens from {} to {}",
                value, from, to
            )],
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

        self.state
            .allowances
            .entry(owner.to_string())
            .or_default()
            .insert(spender.to_string(), value);

        self.events.push(GovernanceEvent::Approval {
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
            gas_used: 46000,
            logs: vec![format!(
                "Approved {} tokens for {} by {}",
                value, spender, owner
            )],
            state_changes,
        })
    }

    /// Delegate votes to another address
    pub fn delegate(&mut self, delegator: &str, delegatee: &str) -> Result<ContractResult> {
        let current_delegate = self.delegates(delegator);

        if current_delegate == delegatee {
            return Ok(ContractResult {
                success: false,
                return_value: b"Already delegated to this address".to_vec(),
                gas_used: 1000,
                logs: vec!["Delegation to same address attempted".to_string()],
                state_changes: HashMap::new(),
            });
        }

        self.state
            .delegates
            .insert(delegator.to_string(), delegatee.to_string());

        let delegator_balance = self.balance_of(delegator);
        self.move_delegates(&current_delegate, delegatee, delegator_balance);

        self.events.push(GovernanceEvent::DelegateChanged {
            delegator: delegator.to_string(),
            from_delegate: current_delegate,
            to_delegate: delegatee.to_string(),
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("delegate_{}", delegator),
            delegatee.as_bytes().to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 30000,
            logs: vec![format!(
                "Delegated votes from {} to {}",
                delegator, delegatee
            )],
            state_changes,
        })
    }

    /// Get current votes for an account
    pub fn get_current_votes(&self, account: &str) -> u64 {
        let ncheckpoints = self
            .state
            .num_checkpoints
            .get(account)
            .copied()
            .unwrap_or(0);
        if ncheckpoints > 0 {
            self.state
                .checkpoints
                .get(account)
                .and_then(|checkpoints| checkpoints.get((ncheckpoints - 1) as usize))
                .map(|checkpoint| checkpoint.votes)
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Get votes at a specific block number
    pub fn get_prior_votes(&self, account: &str, block_number: u64) -> u64 {
        if block_number >= self.state.current_block {
            return 0;
        }

        let ncheckpoints = self
            .state
            .num_checkpoints
            .get(account)
            .copied()
            .unwrap_or(0);
        if ncheckpoints == 0 {
            return 0;
        }

        let checkpoints = self.state.checkpoints.get(account).unwrap();

        // Binary search for the checkpoint
        let mut low = 0;
        let mut high = ncheckpoints as usize;

        while low < high {
            let mid = (low + high) / 2;
            if checkpoints[mid].from_block <= block_number {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        if low > 0 {
            checkpoints[low - 1].votes
        } else {
            0
        }
    }

    /// Get delegate for an account
    pub fn delegates(&self, delegator: &str) -> String {
        self.state
            .delegates
            .get(delegator)
            .cloned()
            .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string())
    }

    /// Take a snapshot of current balances
    pub fn snapshot(&mut self) -> Result<ContractResult> {
        self.state.current_snapshot_id += 1;
        let snapshot_id = self.state.current_snapshot_id;

        // Store current balances
        self.state
            .snapshots
            .insert(snapshot_id, self.state.balances.clone());

        self.events.push(GovernanceEvent::Snapshot {
            id: snapshot_id,
            block_number: self.state.current_block,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            "current_snapshot_id".to_string(),
            snapshot_id.to_le_bytes().to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: snapshot_id.to_le_bytes().to_vec(),
            gas_used: 40000,
            logs: vec![format!("Created snapshot {}", snapshot_id)],
            state_changes,
        })
    }

    /// Get balance at a specific snapshot
    pub fn balance_of_at(&self, account: &str, snapshot_id: u64) -> u64 {
        if snapshot_id > self.state.current_snapshot_id {
            return 0;
        }

        self.state
            .snapshots
            .get(&snapshot_id)
            .and_then(|snapshot| snapshot.get(account))
            .copied()
            .unwrap_or(0)
    }

    /// Internal function to move voting power
    fn move_voting_power(&mut self, from: &str, to: &str, amount: u64) {
        let from_delegate = self.delegates(from);
        let to_delegate = self.delegates(to);

        // Always decrease from the from_delegate and increase to_delegate
        if from_delegate != "0x0000000000000000000000000000000000000000"
            && from != "0x0000000000000000000000000000000000000000"
        {
            self.decrease_votes(&from_delegate, amount);
        }
        if to_delegate != "0x0000000000000000000000000000000000000000"
            && to != "0x0000000000000000000000000000000000000000"
        {
            self.increase_votes(&to_delegate, amount);
        }
    }

    /// Internal function to move delegates
    fn move_delegates(&mut self, src_rep: &str, dst_rep: &str, amount: u64) {
        if src_rep != dst_rep && amount > 0 {
            if src_rep != "0x0000000000000000000000000000000000000000" {
                self.decrease_votes(src_rep, amount);
            }
            if dst_rep != "0x0000000000000000000000000000000000000000" {
                self.increase_votes(dst_rep, amount);
            }
        }
    }

    /// Internal function to increase votes
    fn increase_votes(&mut self, account: &str, amount: u64) {
        let current_votes = self.get_current_votes(account);
        let new_votes = current_votes + amount;
        self.write_checkpoint(account, new_votes);

        self.events.push(GovernanceEvent::DelegateVotesChanged {
            delegate: account.to_string(),
            previous_balance: current_votes,
            new_balance: new_votes,
        });
    }

    /// Internal function to decrease votes
    fn decrease_votes(&mut self, account: &str, amount: u64) {
        let current_votes = self.get_current_votes(account);
        let new_votes = current_votes.saturating_sub(amount);
        self.write_checkpoint(account, new_votes);

        self.events.push(GovernanceEvent::DelegateVotesChanged {
            delegate: account.to_string(),
            previous_balance: current_votes,
            new_balance: new_votes,
        });
    }

    /// Internal function to write checkpoint
    fn write_checkpoint(&mut self, account: &str, new_votes: u64) {
        let ncheckpoints = self
            .state
            .num_checkpoints
            .get(account)
            .copied()
            .unwrap_or(0);

        let checkpoints = self
            .state
            .checkpoints
            .entry(account.to_string())
            .or_default();

        if ncheckpoints > 0
            && checkpoints[(ncheckpoints - 1) as usize].from_block == self.state.current_block
        {
            // Update existing checkpoint for this block
            checkpoints[(ncheckpoints - 1) as usize].votes = new_votes;
        } else {
            // Create new checkpoint
            checkpoints.push(Checkpoint {
                from_block: self.state.current_block,
                votes: new_votes,
            });
            self.state
                .num_checkpoints
                .insert(account.to_string(), ncheckpoints + 1);
        }
    }

    /// Advance block number (for testing/simulation)
    pub fn advance_block(&mut self) {
        self.state.current_block += 1;
    }

    /// Get current block number
    pub fn current_block(&self) -> u64 {
        self.state.current_block
    }

    /// Get all events emitted by the contract
    pub fn get_events(&self) -> &[GovernanceEvent] {
        &self.events
    }

    /// Clear events
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_token_creation() {
        let contract = GovernanceTokenContract::new(
            "Governance Token".to_string(),
            "GOV".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        assert_eq!(contract.name(), "Governance Token");
        assert_eq!(contract.symbol(), "GOV");
        assert_eq!(contract.decimals(), 18);
        assert_eq!(contract.total_supply(), 1000000);
        assert_eq!(contract.balance_of("alice"), 1000000);
    }

    #[test]
    fn test_delegation() {
        let mut contract = GovernanceTokenContract::new(
            "Governance Token".to_string(),
            "GOV".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        // Alice delegates to Bob
        let result = contract.delegate("alice", "bob").unwrap();
        assert!(result.success);
        assert_eq!(contract.delegates("alice"), "bob");
        assert_eq!(contract.get_current_votes("bob"), 1000000);
    }

    #[test]
    fn test_transfer_with_delegation() {
        let mut contract = GovernanceTokenContract::new(
            "Governance Token".to_string(),
            "GOV".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        // Alice delegates to herself
        contract.delegate("alice", "alice").unwrap();
        assert_eq!(contract.get_current_votes("alice"), 1000000);

        // Transfer some tokens to Bob
        contract.transfer("alice", "bob", 100000).unwrap();

        // After transfer, Alice should have 900k voting power (since she delegates to herself)
        assert_eq!(contract.get_current_votes("alice"), 900000);

        // Bob delegates to Charlie
        contract.delegate("bob", "charlie").unwrap();

        // Alice still has 900k, Charlie gets Bob's 100k
        assert_eq!(contract.get_current_votes("alice"), 900000);
        assert_eq!(contract.get_current_votes("charlie"), 100000);
        assert_eq!(contract.get_current_votes("bob"), 0);
    }

    #[test]
    fn test_snapshot() {
        let mut contract = GovernanceTokenContract::new(
            "Governance Token".to_string(),
            "GOV".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        // Take initial snapshot
        let result = contract.snapshot().unwrap();
        assert!(result.success);
        assert_eq!(contract.balance_of_at("alice", 1), 1000000);

        // Transfer some tokens
        contract.transfer("alice", "bob", 100000).unwrap();

        // Take another snapshot
        contract.snapshot().unwrap();
        assert_eq!(contract.balance_of_at("alice", 2), 900000);
        assert_eq!(contract.balance_of_at("bob", 2), 100000);

        // Original snapshot should remain unchanged
        assert_eq!(contract.balance_of_at("alice", 1), 1000000);
        assert_eq!(contract.balance_of_at("bob", 1), 0);
    }

    #[test]
    fn test_prior_votes() {
        let mut contract = GovernanceTokenContract::new(
            "Governance Token".to_string(),
            "GOV".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        // Alice delegates to herself at block 1
        contract.delegate("alice", "alice").unwrap();
        assert_eq!(contract.get_current_votes("alice"), 1000000);
        let initial_block = contract.current_block();

        // Advance to block 2
        contract.advance_block();

        // Transfer half to Bob at block 2
        contract.transfer("alice", "bob", 500000).unwrap();

        // Check votes at different blocks
        assert_eq!(contract.get_prior_votes("alice", initial_block), 1000000);
        assert_eq!(contract.get_current_votes("alice"), 500000);
    }
}
