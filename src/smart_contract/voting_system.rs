//! Basic Voting System
//!
//! This module provides a comprehensive voting system that integrates
//! with governance tokens and proposal management.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    governance_token::GovernanceTokenContract,
    proposal_manager::{ProposalManagerContract, ProposalState, VoteChoice},
};
use crate::{smart_contract::types::ContractResult, Result};

/// Voting system events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingEvent {
    VotingSystemCreated {
        governance_token: String,
        proposal_manager: String,
    },
    VoteCast {
        proposal_id: u64,
        voter: String,
        choice: VoteChoice,
        voting_power: u64,
        reason: String,
    },
    VotingPowerDelegated {
        delegator: String,
        delegatee: String,
        amount: u64,
    },
    QuorumUpdated {
        old_quorum: u64,
        new_quorum: u64,
    },
}

/// Voting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingConfig {
    pub min_voting_period: u64,
    pub max_voting_period: u64,
    pub min_voting_delay: u64,
    pub max_voting_delay: u64,
    pub proposal_threshold_percentage: u64, // Out of 10000
    pub quorum_percentage: u64,             // Out of 10000
    pub vote_differential: u64,             // Minimum difference between for/against
    pub late_quorum_extension: u64,         // Extension if quorum reached late
}

/// Voting system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingSystemState {
    pub governance_token_address: String,
    pub proposal_manager_address: String,
    pub config: VotingConfig,
    pub total_voting_power: u64,
    pub active_proposals: Vec<u64>,
    pub completed_proposals: Vec<u64>,
    pub voting_records: HashMap<String, Vec<u64>>, // voter -> proposal_ids
}

/// Voting system contract
#[derive(Debug, Clone)]
pub struct VotingSystemContract {
    pub state: VotingSystemState,
    pub events: Vec<VotingEvent>,
    pub governance_token: Option<GovernanceTokenContract>,
    pub proposal_manager: Option<ProposalManagerContract>,
}

impl VotingSystemContract {
    /// Create a new voting system
    pub fn new(
        governance_token_address: String,
        proposal_manager_address: String,
        config: VotingConfig,
    ) -> Self {
        let state = VotingSystemState {
            governance_token_address: governance_token_address.clone(),
            proposal_manager_address: proposal_manager_address.clone(),
            config,
            total_voting_power: 0,
            active_proposals: Vec::new(),
            completed_proposals: Vec::new(),
            voting_records: HashMap::new(),
        };

        let mut contract = Self {
            state,
            events: Vec::new(),
            governance_token: None,
            proposal_manager: None,
        };

        contract.events.push(VotingEvent::VotingSystemCreated {
            governance_token: governance_token_address,
            proposal_manager: proposal_manager_address,
        });

        contract
    }

    /// Set governance token contract reference
    pub fn set_governance_token(&mut self, token: GovernanceTokenContract) {
        self.governance_token = Some(token);
    }

    /// Set proposal manager contract reference
    pub fn set_proposal_manager(&mut self, manager: ProposalManagerContract) {
        self.proposal_manager = Some(manager);
    }

    /// Cast vote with reason
    pub fn cast_vote_with_reason(
        &mut self,
        proposal_id: u64,
        voter: &str,
        choice: VoteChoice,
        reason: String,
    ) -> Result<ContractResult> {
        // Get voting power from governance token
        let voting_power = match &self.governance_token {
            Some(token) => token.get_current_votes(voter),
            None => {
                return Ok(ContractResult {
                    success: false,
                    return_value: b"Governance token not set".to_vec(),
                    gas_used: 2000,
                    logs: vec!["Governance token contract not available".to_string()],
                    state_changes: HashMap::new(),
                });
            }
        };

        if voting_power == 0 {
            return Ok(ContractResult {
                success: false,
                return_value: b"No voting power".to_vec(),
                gas_used: 2000,
                logs: vec![format!("Voter {} has no voting power", voter)],
                state_changes: HashMap::new(),
            });
        }

        // Cast vote through proposal manager
        let vote_result = match &mut self.proposal_manager {
            Some(manager) => manager.cast_vote(proposal_id, voter, choice, voting_power)?,
            None => {
                return Ok(ContractResult {
                    success: false,
                    return_value: b"Proposal manager not set".to_vec(),
                    gas_used: 2000,
                    logs: vec!["Proposal manager contract not available".to_string()],
                    state_changes: HashMap::new(),
                });
            }
        };

        if !vote_result.success {
            return Ok(vote_result);
        }

        // Record the vote
        self.state
            .voting_records
            .entry(voter.to_string())
            .or_default()
            .push(proposal_id);

        // Update active proposals list
        if !self.state.active_proposals.contains(&proposal_id) {
            self.state.active_proposals.push(proposal_id);
        }

        self.events.push(VotingEvent::VoteCast {
            proposal_id,
            voter: voter.to_string(),
            choice,
            voting_power,
            reason: reason.clone(),
        });

        let mut state_changes = vote_result.state_changes;
        state_changes.insert(
            format!("voting_record_{}_{}", voter, proposal_id),
            serde_json::to_vec(&choice).unwrap_or_default(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: vote_result.gas_used + 5000,
            logs: vec![format!(
                "Vote cast by {} on proposal {} with power {} - Reason: {}",
                voter, proposal_id, voting_power, reason
            )],
            state_changes,
        })
    }

    /// Cast vote without reason
    pub fn cast_vote(
        &mut self,
        proposal_id: u64,
        voter: &str,
        choice: VoteChoice,
    ) -> Result<ContractResult> {
        self.cast_vote_with_reason(proposal_id, voter, choice, "".to_string())
    }

    /// Delegate voting power
    pub fn delegate_votes(&mut self, delegator: &str, delegatee: &str) -> Result<ContractResult> {
        let delegation_result = match &mut self.governance_token {
            Some(token) => token.delegate(delegator, delegatee)?,
            None => {
                return Ok(ContractResult {
                    success: false,
                    return_value: b"Governance token not set".to_vec(),
                    gas_used: 2000,
                    logs: vec!["Governance token contract not available".to_string()],
                    state_changes: HashMap::new(),
                });
            }
        };

        if delegation_result.success {
            let amount = match &self.governance_token {
                Some(token) => token.balance_of(delegator),
                None => 0,
            };

            self.events.push(VotingEvent::VotingPowerDelegated {
                delegator: delegator.to_string(),
                delegatee: delegatee.to_string(),
                amount,
            });
        }

        Ok(delegation_result)
    }

    /// Get voting power for an account
    pub fn get_voting_power(&self, account: &str) -> u64 {
        match &self.governance_token {
            Some(token) => token.get_current_votes(account),
            None => 0,
        }
    }

    /// Get voting power at a specific block
    pub fn get_voting_power_at(&self, account: &str, block_number: u64) -> u64 {
        match &self.governance_token {
            Some(token) => token.get_prior_votes(account, block_number),
            None => 0,
        }
    }

    /// Check if account has voted on proposal
    pub fn has_voted(&self, proposal_id: u64, voter: &str) -> bool {
        match &self.proposal_manager {
            Some(manager) => {
                if let Some(proposal) = manager.get_proposal(proposal_id) {
                    proposal.votes.contains_key(voter)
                } else {
                    false
                }
            }
            None => false,
        }
    }

    /// Get vote choice for a voter on a proposal
    pub fn get_vote(&self, proposal_id: u64, voter: &str) -> Option<VoteChoice> {
        match &self.proposal_manager {
            Some(manager) => {
                if let Some(proposal) = manager.get_proposal(proposal_id) {
                    proposal.votes.get(voter).map(|vote| vote.choice)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Get proposal vote counts
    pub fn get_proposal_votes(&self, proposal_id: u64) -> Option<(u64, u64, u64)> {
        self.proposal_manager.as_ref().and_then(|manager| {
            manager.get_proposal(proposal_id).map(|proposal| {
                (
                    proposal.for_votes,
                    proposal.against_votes,
                    proposal.abstain_votes,
                )
            })
        })
    }

    /// Get proposal state
    pub fn get_proposal_state(&self, proposal_id: u64) -> Option<ProposalState> {
        self.proposal_manager
            .as_ref()
            .map(|manager| manager.get_proposal_state(proposal_id))
    }

    /// Get quorum for a proposal
    pub fn get_quorum(&self, _proposal_id: u64) -> u64 {
        match &self.governance_token {
            Some(token) => {
                let total_supply = token.total_supply();
                (total_supply * self.state.config.quorum_percentage) / 10000
            }
            None => 0,
        }
    }

    /// Check if quorum is reached for a proposal
    pub fn is_quorum_reached(&self, proposal_id: u64) -> bool {
        let quorum = self.get_quorum(proposal_id);
        if let Some((for_votes, against_votes, abstain_votes)) =
            self.get_proposal_votes(proposal_id)
        {
            let total_votes = for_votes + against_votes + abstain_votes;
            total_votes >= quorum
        } else {
            false
        }
    }

    /// Get voting records for an account
    pub fn get_voting_records(&self, voter: &str) -> Vec<u64> {
        self.state
            .voting_records
            .get(voter)
            .cloned()
            .unwrap_or_default()
    }

    /// Get active proposals
    pub fn get_active_proposals(&self) -> &[u64] {
        &self.state.active_proposals
    }

    /// Get completed proposals
    pub fn get_completed_proposals(&self) -> &[u64] {
        &self.state.completed_proposals
    }

    /// Update voting configuration (governance only)
    pub fn update_config(&mut self, new_config: VotingConfig) -> Result<ContractResult> {
        // Validate configuration
        if new_config.min_voting_period > new_config.max_voting_period {
            return Ok(ContractResult {
                success: false,
                return_value: b"Invalid voting period range".to_vec(),
                gas_used: 2000,
                logs: vec!["Invalid configuration: min > max voting period".to_string()],
                state_changes: HashMap::new(),
            });
        }

        if new_config.min_voting_delay > new_config.max_voting_delay {
            return Ok(ContractResult {
                success: false,
                return_value: b"Invalid voting delay range".to_vec(),
                gas_used: 2000,
                logs: vec!["Invalid configuration: min > max voting delay".to_string()],
                state_changes: HashMap::new(),
            });
        }

        if new_config.quorum_percentage > 10000 {
            return Ok(ContractResult {
                success: false,
                return_value: b"Invalid quorum percentage".to_vec(),
                gas_used: 2000,
                logs: vec!["Invalid configuration: quorum > 100%".to_string()],
                state_changes: HashMap::new(),
            });
        }

        let old_quorum = self.state.config.quorum_percentage;
        self.state.config = new_config;

        self.events.push(VotingEvent::QuorumUpdated {
            old_quorum,
            new_quorum: self.state.config.quorum_percentage,
        });

        let mut state_changes = HashMap::new();
        state_changes.insert(
            "config".to_string(),
            serde_json::to_vec(&self.state.config).unwrap_or_default(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 15000,
            logs: vec!["Voting configuration updated".to_string()],
            state_changes,
        })
    }

    /// Refresh proposal status
    pub fn refresh_proposals(&mut self) -> Result<ContractResult> {
        let mut moved_to_completed = Vec::new();

        // Collect proposals to move first
        let active_proposals = self.state.active_proposals.clone();
        for proposal_id in active_proposals {
            if let Some(state) = self.get_proposal_state(proposal_id) {
                match state {
                    ProposalState::Active | ProposalState::Pending => {}
                    _ => {
                        moved_to_completed.push(proposal_id);
                    }
                }
            }
        }

        // Update active proposals list
        self.state
            .active_proposals
            .retain(|&proposal_id| !moved_to_completed.contains(&proposal_id));

        // Move completed proposals
        self.state
            .completed_proposals
            .extend(moved_to_completed.clone());

        let mut state_changes = HashMap::new();
        state_changes.insert(
            "active_proposals".to_string(),
            serde_json::to_vec(&self.state.active_proposals).unwrap_or_default(),
        );
        state_changes.insert(
            "completed_proposals".to_string(),
            serde_json::to_vec(&self.state.completed_proposals).unwrap_or_default(),
        );

        Ok(ContractResult {
            success: true,
            return_value: b"true".to_vec(),
            gas_used: 10000,
            logs: vec![format!(
                "Moved {} proposals to completed",
                moved_to_completed.len()
            )],
            state_changes,
        })
    }

    /// Get events
    pub fn get_events(&self) -> &[VotingEvent] {
        &self.events
    }

    /// Clear events
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
}

/// Default voting configuration
impl Default for VotingConfig {
    fn default() -> Self {
        Self {
            min_voting_period: 100,             // 100 blocks minimum
            max_voting_period: 50400,           // ~1 week at 12s/block
            min_voting_delay: 1,                // 1 block minimum
            max_voting_delay: 7200,             // ~1 day at 12s/block
            proposal_threshold_percentage: 100, // 1% of total supply
            quorum_percentage: 2500,            // 25% of total supply
            vote_differential: 500,             // 5% minimum difference
            late_quorum_extension: 7200,        // ~1 day extension
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_contract::{
        governance_token::GovernanceTokenContract, proposal_manager::ProposalManagerContract,
    };

    fn setup_voting_system() -> VotingSystemContract {
        let config = VotingConfig::default();
        VotingSystemContract::new(
            "gov_token".to_string(),
            "proposal_manager".to_string(),
            config,
        )
    }

    fn setup_full_system() -> (
        VotingSystemContract,
        GovernanceTokenContract,
        ProposalManagerContract,
    ) {
        let mut voting_system = setup_voting_system();

        let governance_token = GovernanceTokenContract::new(
            "Governance Token".to_string(),
            "GOV".to_string(),
            18,
            1000000,
            "alice".to_string(),
        );

        let proposal_manager = ProposalManagerContract::new(
            "gov_token".to_string(),
            5,    // voting delay
            100,  // voting period
            1000, // proposal threshold
            2500, // quorum
            50,   // timelock delay
        );

        voting_system.set_governance_token(governance_token.clone());
        voting_system.set_proposal_manager(proposal_manager.clone());

        (voting_system, governance_token, proposal_manager)
    }

    #[test]
    fn test_voting_system_creation() {
        let voting_system = setup_voting_system();

        assert_eq!(voting_system.state.governance_token_address, "gov_token");
        assert_eq!(
            voting_system.state.proposal_manager_address,
            "proposal_manager"
        );
        assert_eq!(voting_system.state.config.quorum_percentage, 2500);
    }

    #[test]
    fn test_voting_power_delegation() {
        let (mut voting_system, _governance_token, _) = setup_full_system();

        // Alice delegates to Bob
        let result = voting_system.delegate_votes("alice", "bob").unwrap();
        assert!(result.success);

        // Check voting power
        assert_eq!(voting_system.get_voting_power("bob"), 1000000);
        assert_eq!(voting_system.get_voting_power("alice"), 0);
    }

    #[test]
    fn test_integrated_voting() {
        let (mut voting_system, _governance_token, mut proposal_manager) = setup_full_system();

        // Alice delegates to herself
        voting_system.delegate_votes("alice", "alice").unwrap();

        // Create a proposal
        proposal_manager
            .propose(
                "alice",
                "Test Proposal".to_string(),
                "A test proposal".to_string(),
                vec!["target1".to_string()],
                vec![0],
                vec![vec![1, 2, 3]],
                1000000,
            )
            .unwrap();

        // Advance to voting period
        for _ in 0..6 {
            proposal_manager.advance_block();
        }

        // Update the proposal manager in voting system
        voting_system.set_proposal_manager(proposal_manager);

        // Cast vote
        let result = voting_system
            .cast_vote_with_reason(
                1,
                "alice",
                VoteChoice::For,
                "I support this proposal".to_string(),
            )
            .unwrap();

        assert!(result.success);
        assert!(voting_system.has_voted(1, "alice"));
        assert_eq!(voting_system.get_vote(1, "alice"), Some(VoteChoice::For));
    }

    #[test]
    fn test_quorum_calculation() {
        let (voting_system, _, _) = setup_full_system();

        // Quorum should be 25% of total supply (1000000)
        let quorum = voting_system.get_quorum(1);
        assert_eq!(quorum, 250000);
    }

    #[test]
    fn test_config_update() {
        let mut voting_system = setup_voting_system();

        let new_config = VotingConfig {
            quorum_percentage: 3000, // 30%
            ..Default::default()
        };

        let result = voting_system.update_config(new_config).unwrap();
        assert!(result.success);
        assert_eq!(voting_system.state.config.quorum_percentage, 3000);
    }

    #[test]
    fn test_invalid_config_update() {
        let mut voting_system = setup_voting_system();

        let invalid_config = VotingConfig {
            min_voting_period: 200,
            max_voting_period: 100, // Invalid: min > max
            ..Default::default()
        };

        let result = voting_system.update_config(invalid_config).unwrap();
        assert!(!result.success);
    }
}
