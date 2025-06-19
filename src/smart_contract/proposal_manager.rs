//! Proposal Management System
//!
//! This module provides a comprehensive proposal management system
//! for governance operations with voting periods and execution.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{smart_contract::types::ContractResult, Result};

/// Proposal state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalState {
    Pending,
    Active,
    Canceled,
    Defeated,
    Succeeded,
    Queued,
    Expired,
    Executed,
}

/// Vote choice enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

/// Individual vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub choice: VoteChoice,
    pub voting_power: u64,
    pub timestamp: u64,
}

/// Proposal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub targets: Vec<String>,    // Contract addresses to call
    pub values: Vec<u64>,        // ETH values for each call
    pub calldatas: Vec<Vec<u8>>, // Function call data
    pub start_block: u64,
    pub end_block: u64,
    pub snapshot_block: u64,   // Block number for voting power calculation
    pub quorum_threshold: u64, // Minimum votes needed
    pub vote_threshold: u64,   // Percentage needed to pass (out of 10000)
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    pub canceled: bool,
    pub executed: bool,
    pub queued: bool,   // Whether proposal is queued for execution
    pub queued_at: u64, // When it was queued
    pub votes: HashMap<String, Vote>,
    pub created_at: u64,
    pub execution_delay: u64, // Delay before execution after success
}

/// Proposal events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalEvent {
    ProposalCreated {
        proposal_id: u64,
        proposer: String,
        title: String,
        start_block: u64,
        end_block: u64,
    },
    VoteCast {
        proposal_id: u64,
        voter: String,
        choice: VoteChoice,
        voting_power: u64,
    },
    ProposalCanceled {
        proposal_id: u64,
    },
    ProposalQueued {
        proposal_id: u64,
        execution_time: u64,
    },
    ProposalExecuted {
        proposal_id: u64,
    },
}

/// Proposal manager state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalManagerState {
    pub proposals: HashMap<u64, Proposal>,
    pub proposal_count: u64,
    pub voting_delay: u64,       // Blocks between proposal and voting start
    pub voting_period: u64,      // Duration of voting in blocks
    pub proposal_threshold: u64, // Minimum tokens needed to propose
    pub quorum_numerator: u64,   // Quorum as fraction of total supply
    pub timelock_delay: u64,     // Minimum delay before execution
    pub current_block: u64,
    pub governance_token: String, // Address of governance token contract
}

/// Proposal manager contract
#[derive(Debug, Clone)]
pub struct ProposalManagerContract {
    pub state: ProposalManagerState,
    pub events: Vec<ProposalEvent>,
}

impl ProposalManagerContract {
    /// Create a new proposal manager
    pub fn new(
        governance_token: String,
        voting_delay: u64,
        voting_period: u64,
        proposal_threshold: u64,
        quorum_numerator: u64,
        timelock_delay: u64,
    ) -> Self {
        let state = ProposalManagerState {
            proposals: HashMap::new(),
            proposal_count: 0,
            voting_delay,
            voting_period,
            proposal_threshold,
            quorum_numerator,
            timelock_delay,
            current_block: 1,
            governance_token,
        };

        Self {
            state,
            events: Vec::new(),
        }
    }

    /// Create a new proposal
    pub fn propose(
        &mut self,
        proposer: &str,
        title: String,
        description: String,
        targets: Vec<String>,
        values: Vec<u64>,
        calldatas: Vec<Vec<u8>>,
        proposer_votes: u64,
    ) -> Result<ContractResult> {
        // Check if proposer has enough voting power
        if proposer_votes < self.state.proposal_threshold {
            return Ok(ContractResult {
                success: false,
                return_value: b"Insufficient voting power to propose".to_vec(),
                gas_used: 5000,
                logs: vec![format!(
                    "Proposal threshold not met: {} < {}",
                    proposer_votes, self.state.proposal_threshold
                )],
                state_changes: HashMap::new(),
            });
        }

        // Validate proposal structure
        if targets.len() != values.len() || targets.len() != calldatas.len() {
            return Ok(ContractResult {
                success: false,
                return_value: b"Proposal arrays length mismatch".to_vec(),
                gas_used: 2000,
                logs: vec!["Proposal structure validation failed".to_string()],
                state_changes: HashMap::new(),
            });
        }

        if targets.is_empty() {
            return Ok(ContractResult {
                success: false,
                return_value: b"Empty proposal not allowed".to_vec(),
                gas_used: 2000,
                logs: vec!["Empty proposal rejected".to_string()],
                state_changes: HashMap::new(),
            });
        }

        self.state.proposal_count += 1;
        let proposal_id = self.state.proposal_count;

        let start_block = self.state.current_block + self.state.voting_delay;
        let end_block = start_block + self.state.voting_period;
        let snapshot_block = self.state.current_block;

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.to_string(),
            title: title.clone(),
            description,
            targets,
            values,
            calldatas,
            start_block,
            end_block,
            snapshot_block,
            quorum_threshold: self.state.quorum_numerator, // Will be calculated with total supply
            vote_threshold: 5000,                          // 50% (out of 10000)
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            canceled: false,
            executed: false,
            queued: false,
            queued_at: 0,
            votes: HashMap::new(),
            created_at: self.state.current_block,
            execution_delay: self.state.timelock_delay,
        };

        self.state.proposals.insert(proposal_id, proposal);

        self.events.push(ProposalEvent::ProposalCreated {
            proposal_id,
            proposer: proposer.to_string(),
            title,
            start_block,
            end_block,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            "proposal_count".to_string(),
            proposal_id.to_le_bytes().to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: proposal_id.to_le_bytes().to_vec(),
            gas_used: 50000,
            logs: vec![format!("Created proposal {} by {}", proposal_id, proposer)],
            state_changes,
        })
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(
        &mut self,
        proposal_id: u64,
        voter: &str,
        choice: VoteChoice,
        voting_power: u64,
    ) -> Result<ContractResult> {
        let proposal = match self.state.proposals.get_mut(&proposal_id) {
            Some(p) => p,
            None => {
                return Ok(ContractResult {
                    success: false,
                    return_value: b"Proposal not found".to_vec(),
                    gas_used: 2000,
                    logs: vec![format!("Proposal {} not found", proposal_id)],
                    state_changes: HashMap::new(),
                });
            }
        };

        // Check if voting is active
        if self.state.current_block < proposal.start_block {
            return Ok(ContractResult {
                success: false,
                return_value: b"Voting not yet started".to_vec(),
                gas_used: 2000,
                logs: vec!["Voting period not active".to_string()],
                state_changes: HashMap::new(),
            });
        }

        if self.state.current_block > proposal.end_block {
            return Ok(ContractResult {
                success: false,
                return_value: b"Voting period ended".to_vec(),
                gas_used: 2000,
                logs: vec!["Voting period has ended".to_string()],
                state_changes: HashMap::new(),
            });
        }

        if proposal.canceled {
            return Ok(ContractResult {
                success: false,
                return_value: b"Proposal was canceled".to_vec(),
                gas_used: 2000,
                logs: vec!["Cannot vote on canceled proposal".to_string()],
                state_changes: HashMap::new(),
            });
        }

        // Check if voter already voted
        if proposal.votes.contains_key(voter) {
            return Ok(ContractResult {
                success: false,
                return_value: b"Already voted".to_vec(),
                gas_used: 2000,
                logs: vec![format!("Voter {} already voted", voter)],
                state_changes: HashMap::new(),
            });
        }

        if voting_power == 0 {
            return Ok(ContractResult {
                success: false,
                return_value: b"No voting power".to_vec(),
                gas_used: 2000,
                logs: vec!["No voting power to cast vote".to_string()],
                state_changes: HashMap::new(),
            });
        }

        // Record the vote
        let vote = Vote {
            voter: voter.to_string(),
            choice,
            voting_power,
            timestamp: self.state.current_block,
        };

        proposal.votes.insert(voter.to_string(), vote);

        // Update vote counts
        match choice {
            VoteChoice::For => proposal.for_votes += voting_power,
            VoteChoice::Against => proposal.against_votes += voting_power,
            VoteChoice::Abstain => proposal.abstain_votes += voting_power,
        }

        self.events.push(ProposalEvent::VoteCast {
            proposal_id,
            voter: voter.to_string(),
            choice,
            voting_power,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("vote_{}_{}", proposal_id, voter),
            serde_json::to_vec(&choice).unwrap_or_default(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 25000,
            logs: vec![format!(
                "Vote cast by {} on proposal {} with power {}",
                voter, proposal_id, voting_power
            )],
            state_changes,
        })
    }

    /// Cancel a proposal (only by proposer or governance)
    pub fn cancel_proposal(&mut self, proposal_id: u64, canceler: &str) -> Result<ContractResult> {
        let proposal = match self.state.proposals.get_mut(&proposal_id) {
            Some(p) => p,
            None => {
                return Ok(ContractResult {
                    success: false,
                    return_value: b"Proposal not found".to_vec(),
                    gas_used: 2000,
                    logs: vec![format!("Proposal {} not found", proposal_id)],
                    state_changes: HashMap::new(),
                });
            }
        };

        // Only proposer can cancel their own proposal
        if proposal.proposer != canceler {
            return Ok(ContractResult {
                success: false,
                return_value: b"Only proposer can cancel".to_vec(),
                gas_used: 2000,
                logs: vec!["Unauthorized cancellation attempt".to_string()],
                state_changes: HashMap::new(),
            });
        }

        if proposal.executed {
            return Ok(ContractResult {
                success: false,
                return_value: b"Cannot cancel executed proposal".to_vec(),
                gas_used: 2000,
                logs: vec!["Cannot cancel executed proposal".to_string()],
                state_changes: HashMap::new(),
            });
        }

        proposal.canceled = true;

        self.events
            .push(ProposalEvent::ProposalCanceled { proposal_id });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("proposal_{}_canceled", proposal_id),
            b"true".to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 15000,
            logs: vec![format!("Proposal {} canceled", proposal_id)],
            state_changes,
        })
    }

    /// Queue a successful proposal for execution
    pub fn queue_proposal(&mut self, proposal_id: u64) -> Result<ContractResult> {
        let state = self.get_proposal_state(proposal_id);

        if state != ProposalState::Succeeded {
            return Ok(ContractResult {
                success: false,
                return_value: b"Proposal not in succeeded state".to_vec(),
                gas_used: 2000,
                logs: vec![format!("Proposal {} not succeeded", proposal_id)],
                state_changes: HashMap::new(),
            });
        }

        let execution_time = self.state.current_block + self.state.timelock_delay;

        // Update proposal state
        if let Some(proposal) = self.state.proposals.get_mut(&proposal_id) {
            proposal.queued = true;
            proposal.queued_at = self.state.current_block;
        }

        self.events.push(ProposalEvent::ProposalQueued {
            proposal_id,
            execution_time,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("proposal_{}_queued", proposal_id),
            execution_time.to_le_bytes().to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 20000,
            logs: vec![format!("Proposal {} queued for execution", proposal_id)],
            state_changes,
        })
    }

    /// Execute a queued proposal
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<ContractResult> {
        let state = self.get_proposal_state(proposal_id);
        if state != ProposalState::Queued {
            return Ok(ContractResult {
                success: false,
                return_value: b"Proposal not queued for execution".to_vec(),
                gas_used: 2000,
                logs: vec![format!("Proposal {} not queued", proposal_id)],
                state_changes: HashMap::new(),
            });
        }

        let proposal = match self.state.proposals.get_mut(&proposal_id) {
            Some(p) => p,
            None => {
                return Ok(ContractResult {
                    success: false,
                    return_value: b"Proposal not found".to_vec(),
                    gas_used: 2000,
                    logs: vec![format!("Proposal {} not found", proposal_id)],
                    state_changes: HashMap::new(),
                });
            }
        };

        proposal.executed = true;

        self.events
            .push(ProposalEvent::ProposalExecuted { proposal_id });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("proposal_{}_executed", proposal_id),
            b"true".to_vec(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 100000, // Higher gas for potential execution
            logs: vec![format!("Proposal {} executed", proposal_id)],
            state_changes,
        })
    }

    /// Get proposal state
    pub fn get_proposal_state(&self, proposal_id: u64) -> ProposalState {
        let proposal = match self.state.proposals.get(&proposal_id) {
            Some(p) => p,
            None => return ProposalState::Pending,
        };

        if proposal.canceled {
            return ProposalState::Canceled;
        }

        if proposal.executed {
            return ProposalState::Executed;
        }

        if self.state.current_block < proposal.start_block {
            return ProposalState::Pending;
        }

        if self.state.current_block <= proposal.end_block {
            return ProposalState::Active;
        }

        // Voting has ended, determine result
        let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let quorum_reached = total_votes >= proposal.quorum_threshold;
        let votes_for_percentage = if total_votes > 0 {
            (proposal.for_votes * 10000) / total_votes
        } else {
            0
        };

        if !quorum_reached || votes_for_percentage < proposal.vote_threshold {
            return ProposalState::Defeated;
        }

        // Check if proposal is queued
        if proposal.queued {
            let execution_ready_time = proposal.queued_at + self.state.timelock_delay;
            if self.state.current_block >= execution_ready_time {
                ProposalState::Queued
            } else {
                ProposalState::Succeeded
            }
        } else {
            ProposalState::Succeeded
        }
    }

    /// Get proposal details
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&Proposal> {
        self.state.proposals.get(&proposal_id)
    }

    /// Get all proposals
    pub fn get_all_proposals(&self) -> Vec<&Proposal> {
        self.state.proposals.values().collect()
    }

    /// Get proposal count
    pub fn proposal_count(&self) -> u64 {
        self.state.proposal_count
    }

    /// Advance block number (for testing/simulation)
    pub fn advance_block(&mut self) {
        self.state.current_block += 1;
    }

    /// Get current block number
    pub fn current_block(&self) -> u64 {
        self.state.current_block
    }

    /// Get events
    pub fn get_events(&self) -> &[ProposalEvent] {
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
    fn test_proposal_creation() {
        let mut manager = ProposalManagerContract::new(
            "gov_token".to_string(),
            10,   // voting delay
            100,  // voting period
            1000, // proposal threshold
            2500, // 25% quorum
            50,   // timelock delay
        );

        let result = manager
            .propose(
                "alice",
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                vec!["target1".to_string()],
                vec![0],
                vec![vec![1, 2, 3]],
                1500, // voting power
            )
            .unwrap();

        assert!(result.success);
        assert_eq!(manager.proposal_count(), 1);

        let proposal = manager.get_proposal(1).unwrap();
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.proposer, "alice");
    }

    #[test]
    fn test_voting() {
        let mut manager = ProposalManagerContract::new(
            "gov_token".to_string(),
            5,    // voting delay
            20,   // voting period
            100,  // proposal threshold
            1000, // quorum
            10,   // timelock delay
        );

        // Create proposal
        manager
            .propose(
                "alice",
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                vec!["target1".to_string()],
                vec![0],
                vec![vec![1, 2, 3]],
                500,
            )
            .unwrap();

        // Advance to voting period
        for _ in 0..6 {
            manager.advance_block();
        }

        // Cast votes
        let result = manager.cast_vote(1, "bob", VoteChoice::For, 600).unwrap();
        assert!(result.success);

        let result = manager
            .cast_vote(1, "charlie", VoteChoice::Against, 400)
            .unwrap();
        assert!(result.success);

        let proposal = manager.get_proposal(1).unwrap();
        assert_eq!(proposal.for_votes, 600);
        assert_eq!(proposal.against_votes, 400);
    }

    #[test]
    fn test_proposal_states() {
        let mut manager = ProposalManagerContract::new(
            "gov_token".to_string(),
            5,   // voting delay
            10,  // voting period
            100, // proposal threshold
            500, // quorum
            5,   // timelock delay
        );

        // Create proposal
        manager
            .propose(
                "alice",
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                vec!["target1".to_string()],
                vec![0],
                vec![vec![1, 2, 3]],
                200,
            )
            .unwrap();

        // Initially pending
        assert_eq!(manager.get_proposal_state(1), ProposalState::Pending);

        // Advance to voting period
        for _ in 0..6 {
            manager.advance_block();
        }
        assert_eq!(manager.get_proposal_state(1), ProposalState::Active);

        // Cast successful vote
        manager.cast_vote(1, "bob", VoteChoice::For, 800).unwrap();

        // Advance past voting period
        for _ in 0..11 {
            manager.advance_block();
        }
        assert_eq!(manager.get_proposal_state(1), ProposalState::Succeeded);
    }

    #[test]
    fn test_proposal_cancellation() {
        let mut manager = ProposalManagerContract::new(
            "gov_token".to_string(),
            5,   // voting delay
            10,  // voting period
            100, // proposal threshold
            500, // quorum
            5,   // timelock delay
        );

        // Create proposal
        manager
            .propose(
                "alice",
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                vec!["target1".to_string()],
                vec![0],
                vec![vec![1, 2, 3]],
                200,
            )
            .unwrap();

        // Cancel proposal
        let result = manager.cancel_proposal(1, "alice").unwrap();
        assert!(result.success);
        assert_eq!(manager.get_proposal_state(1), ProposalState::Canceled);
    }
}
