//! Integration tests for governance system
//!
//! This module tests the integration between governance token,
//! proposal manager, and voting system.

use polytorus::smart_contract::{
    governance_token::GovernanceTokenContract,
    proposal_manager::{ProposalManagerContract, ProposalState, VoteChoice},
    voting_system::{VotingConfig, VotingSystemContract},
};

#[test]
fn test_complete_governance_workflow() {
    // Create governance token
    let mut governance_token = GovernanceTokenContract::new(
        "Polytorus Governance Token".to_string(),
        "PGT".to_string(),
        18,
        10000000, // 10M total supply
        "alice".to_string(),
    );

    // Create proposal manager
    let mut proposal_manager = ProposalManagerContract::new(
        "governance_token".to_string(),
        10,      // voting delay
        100,     // voting period
        100000,  // proposal threshold (1% of total supply)
        2500000, // 25% quorum (actual value, not percentage)
        50,      // timelock delay
    );

    // Create voting system
    let config = VotingConfig {
        min_voting_period: 50,
        max_voting_period: 200,
        min_voting_delay: 5,
        max_voting_delay: 20,
        proposal_threshold_percentage: 100, // 1%
        quorum_percentage: 2500,            // 25%
        vote_differential: 500,             // 5%
        late_quorum_extension: 50,
    };

    let mut voting_system = VotingSystemContract::new(
        "governance_token".to_string(),
        "proposal_manager".to_string(),
        config,
    );

    // Set references
    voting_system.set_governance_token(governance_token.clone());
    voting_system.set_proposal_manager(proposal_manager.clone());

    // Step 1: Distribute tokens and delegate voting power
    governance_token.transfer("alice", "bob", 2000000).unwrap();
    governance_token
        .transfer("alice", "charlie", 1500000)
        .unwrap();
    governance_token
        .transfer("alice", "david", 1000000)
        .unwrap();

    // Self-delegate voting power
    governance_token.delegate("alice", "alice").unwrap();
    governance_token.delegate("bob", "bob").unwrap();
    governance_token.delegate("charlie", "charlie").unwrap();
    governance_token.delegate("david", "david").unwrap();

    // Verify voting power
    assert_eq!(governance_token.get_current_votes("alice"), 5500000);
    assert_eq!(governance_token.get_current_votes("bob"), 2000000);
    assert_eq!(governance_token.get_current_votes("charlie"), 1500000);
    assert_eq!(governance_token.get_current_votes("david"), 1000000);

    // Step 2: Create a proposal
    let proposal_result = proposal_manager
        .propose(
            "alice",
            "Upgrade Protocol".to_string(),
            "Proposal to upgrade the protocol to version 2.0".to_string(),
            vec!["protocol_contract".to_string()],
            vec![0],
            vec![vec![1, 2, 3, 4]], // upgrade call data
            5500000,                // Alice's voting power
        )
        .unwrap();

    assert!(proposal_result.success);
    assert_eq!(proposal_manager.proposal_count(), 1);

    let proposal = proposal_manager.get_proposal(1).unwrap();
    assert_eq!(proposal.title, "Upgrade Protocol");
    assert_eq!(proposal.proposer, "alice");

    // Step 3: Wait for voting to start
    assert_eq!(
        proposal_manager.get_proposal_state(1),
        ProposalState::Pending
    );

    // Advance blocks to start voting
    for _ in 0..11 {
        proposal_manager.advance_block();
        governance_token.advance_block();
    }

    assert_eq!(
        proposal_manager.get_proposal_state(1),
        ProposalState::Active
    );

    // Step 4: Cast votes directly through proposal manager
    // Alice votes FOR (5.5M voting power)
    let alice_power = governance_token.get_current_votes("alice");
    let vote_result = proposal_manager
        .cast_vote(1, "alice", VoteChoice::For, alice_power)
        .unwrap();
    assert!(vote_result.success);

    // Bob votes AGAINST (2M voting power)
    let bob_power = governance_token.get_current_votes("bob");
    let vote_result = proposal_manager
        .cast_vote(1, "bob", VoteChoice::Against, bob_power)
        .unwrap();
    assert!(vote_result.success);

    // Charlie votes FOR (1.5M voting power)
    let charlie_power = governance_token.get_current_votes("charlie");
    let vote_result = proposal_manager
        .cast_vote(1, "charlie", VoteChoice::For, charlie_power)
        .unwrap();
    assert!(vote_result.success);

    // David abstains (1M voting power)
    let david_power = governance_token.get_current_votes("david");
    let vote_result = proposal_manager
        .cast_vote(1, "david", VoteChoice::Abstain, david_power)
        .unwrap();
    assert!(vote_result.success);

    // Verify votes were recorded in proposal manager
    let proposal = proposal_manager.get_proposal(1).unwrap();
    assert!(proposal.votes.contains_key("alice"));
    assert!(proposal.votes.contains_key("bob"));
    assert!(proposal.votes.contains_key("charlie"));
    assert!(proposal.votes.contains_key("david"));

    assert_eq!(proposal.votes.get("alice").unwrap().choice, VoteChoice::For);
    assert_eq!(
        proposal.votes.get("bob").unwrap().choice,
        VoteChoice::Against
    );
    assert_eq!(
        proposal.votes.get("charlie").unwrap().choice,
        VoteChoice::For
    );
    assert_eq!(
        proposal.votes.get("david").unwrap().choice,
        VoteChoice::Abstain
    );

    // Check vote counts from proposal
    let proposal = proposal_manager.get_proposal(1).unwrap();
    let for_votes = proposal.for_votes;
    let against_votes = proposal.against_votes;
    let abstain_votes = proposal.abstain_votes;

    // Debug: Print actual voting power
    println!("Alice voting power: {alice_power}");
    println!("Bob voting power: {bob_power}");
    println!("Charlie voting power: {charlie_power}");
    println!("David voting power: {david_power}");
    println!("For: {for_votes}, Against: {against_votes}, Abstain: {abstain_votes}");

    assert_eq!(for_votes, alice_power + charlie_power); // Alice + Charlie
    assert_eq!(against_votes, bob_power); // Bob
    assert_eq!(abstain_votes, david_power); // David

    // Verify quorum is reached
    let total_votes = for_votes + against_votes + abstain_votes;
    assert!(total_votes >= proposal.quorum_threshold);

    // Step 5: End voting period
    for _ in 0..101 {
        proposal_manager.advance_block();
    }

    // Debug: Check proposal state calculation
    let proposal = proposal_manager.get_proposal(1).unwrap();
    let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
    let quorum_reached = total_votes >= proposal.quorum_threshold;
    let votes_for_percentage = (proposal.for_votes * 10000) / total_votes;

    println!("Total votes: {total_votes}");
    println!("Quorum threshold: {}", proposal.quorum_threshold);
    println!("Quorum reached: {quorum_reached}");
    println!(
        "For votes percentage: {votes_for_percentage} (threshold: {})",
        proposal.vote_threshold
    );

    // Proposal should have succeeded (7M for vs 2M against, quorum reached)
    assert_eq!(
        proposal_manager.get_proposal_state(1),
        ProposalState::Succeeded
    );

    // Step 6: Queue proposal for execution
    let queue_result = proposal_manager.queue_proposal(1).unwrap();
    if !queue_result.success {
        println!(
            "Queue failed: {}",
            String::from_utf8_lossy(&queue_result.return_value)
        );
        for log in &queue_result.logs {
            println!("Queue log: {log}");
        }
    }
    assert!(queue_result.success);

    // Step 7: Execute proposal after timelock
    for _ in 0..51 {
        proposal_manager.advance_block();
    }

    let execute_result = proposal_manager.execute_proposal(1).unwrap();
    if !execute_result.success {
        println!(
            "Execution failed: {}",
            String::from_utf8_lossy(&execute_result.return_value)
        );
        for log in &execute_result.logs {
            println!("Log: {log}");
        }
    }
    assert!(execute_result.success);

    assert_eq!(
        proposal_manager.get_proposal_state(1),
        ProposalState::Executed
    );
}

#[test]
fn test_proposal_rejection_due_to_insufficient_votes() {
    let mut governance_token = GovernanceTokenContract::new(
        "Test Token".to_string(),
        "TEST".to_string(),
        18,
        1000000,
        "alice".to_string(),
    );

    let mut proposal_manager = ProposalManagerContract::new(
        "governance_token".to_string(),
        5,      // voting delay
        50,     // voting period
        10000,  // proposal threshold
        800000, // 80% quorum (very high threshold to ensure failure)
        25,     // timelock delay
    );

    let _voting_system = VotingSystemContract::new(
        "governance_token".to_string(),
        "proposal_manager".to_string(),
        VotingConfig::default(),
    );

    // Distribute some tokens
    governance_token.transfer("alice", "bob", 400000).unwrap();
    governance_token.delegate("alice", "alice").unwrap();
    governance_token.delegate("bob", "bob").unwrap();

    // Create proposal
    proposal_manager
        .propose(
            "alice",
            "Test Proposal".to_string(),
            "A test proposal".to_string(),
            vec!["target".to_string()],
            vec![0],
            vec![vec![1, 2, 3]],
            600000,
        )
        .unwrap();

    // Advance to voting period
    for _ in 0..6 {
        proposal_manager.advance_block();
        governance_token.advance_block();
    }

    // Only Alice votes (600k votes), Bob doesn't vote
    let alice_power = governance_token.get_current_votes("alice");
    proposal_manager
        .cast_vote(1, "alice", VoteChoice::For, alice_power)
        .unwrap();

    // End voting period
    for _ in 0..51 {
        proposal_manager.advance_block();
    }

    // Should be defeated due to insufficient quorum (need 800k, only got 600k)
    assert_eq!(
        proposal_manager.get_proposal_state(1),
        ProposalState::Defeated
    );
}

#[test]
fn test_delegation_changes_voting_power() {
    let mut governance_token = GovernanceTokenContract::new(
        "Test Token".to_string(),
        "TEST".to_string(),
        18,
        1000000,
        "alice".to_string(),
    );

    // Transfer tokens to Bob
    governance_token.transfer("alice", "bob", 300000).unwrap();

    // Initially, no one has voting power
    assert_eq!(governance_token.get_current_votes("alice"), 0);
    assert_eq!(governance_token.get_current_votes("bob"), 0);

    // Alice delegates to herself
    governance_token.delegate("alice", "alice").unwrap();
    assert_eq!(governance_token.get_current_votes("alice"), 700000);

    // Bob delegates to Alice
    governance_token.delegate("bob", "alice").unwrap();
    assert_eq!(governance_token.get_current_votes("alice"), 1000000);
    assert_eq!(governance_token.get_current_votes("bob"), 0);

    // Bob changes delegation to himself
    governance_token.delegate("bob", "bob").unwrap();
    assert_eq!(governance_token.get_current_votes("alice"), 700000);
    assert_eq!(governance_token.get_current_votes("bob"), 300000);
}

#[test]
fn test_snapshot_voting_power() {
    let mut governance_token = GovernanceTokenContract::new(
        "Test Token".to_string(),
        "TEST".to_string(),
        18,
        1000000,
        "alice".to_string(),
    );

    // Alice delegates to herself at block 1
    governance_token.delegate("alice", "alice").unwrap();
    assert_eq!(governance_token.get_current_votes("alice"), 1000000);

    // Take snapshot of current voting power
    let snapshot_result = governance_token.snapshot().unwrap();
    assert!(snapshot_result.success);

    // Advance blocks
    governance_token.advance_block();
    governance_token.advance_block();

    // Transfer some tokens to Bob at block 3
    governance_token.transfer("alice", "bob", 400000).unwrap();
    governance_token.delegate("bob", "bob").unwrap();

    // Current voting power should be updated
    assert_eq!(governance_token.get_current_votes("alice"), 600000);
    assert_eq!(governance_token.get_current_votes("bob"), 400000);

    // But snapshot should preserve original balances
    assert_eq!(governance_token.balance_of_at("alice", 1), 1000000);
    assert_eq!(governance_token.balance_of_at("bob", 1), 0);

    // Historical voting power should also be preserved
    assert_eq!(governance_token.get_prior_votes("alice", 1), 1000000);
    assert_eq!(governance_token.get_prior_votes("bob", 1), 0);
}

#[test]
fn test_proposal_cancellation() {
    let mut proposal_manager = ProposalManagerContract::new(
        "governance_token".to_string(),
        5,    // voting delay
        50,   // voting period
        1000, // proposal threshold
        2000, // quorum
        25,   // timelock delay
    );

    // Create proposal
    let result = proposal_manager
        .propose(
            "alice",
            "Test Proposal".to_string(),
            "A test proposal".to_string(),
            vec!["target".to_string()],
            vec![0],
            vec![vec![1, 2, 3]],
            1500,
        )
        .unwrap();
    assert!(result.success);

    // Proposal should be pending
    assert_eq!(
        proposal_manager.get_proposal_state(1),
        ProposalState::Pending
    );

    // Alice cancels the proposal
    let cancel_result = proposal_manager.cancel_proposal(1, "alice").unwrap();
    assert!(cancel_result.success);

    // Proposal should be canceled
    assert_eq!(
        proposal_manager.get_proposal_state(1),
        ProposalState::Canceled
    );

    // Non-proposer cannot cancel
    proposal_manager
        .propose(
            "bob",
            "Another Proposal".to_string(),
            "Another test".to_string(),
            vec!["target".to_string()],
            vec![0],
            vec![vec![4, 5, 6]],
            1500,
        )
        .unwrap();

    let cancel_result = proposal_manager.cancel_proposal(2, "alice").unwrap();
    assert!(!cancel_result.success);
}

#[test]
fn test_voting_system_integration() {
    let governance_token = GovernanceTokenContract::new(
        "Test Token".to_string(),
        "TEST".to_string(),
        18,
        1000000,
        "alice".to_string(),
    );

    let proposal_manager = ProposalManagerContract::new(
        "governance_token".to_string(),
        1,    // voting delay
        10,   // voting period
        1000, // proposal threshold
        2000, // quorum
        5,    // timelock delay
    );

    let mut voting_system = VotingSystemContract::new(
        "governance_token".to_string(),
        "proposal_manager".to_string(),
        VotingConfig::default(),
    );

    voting_system.set_governance_token(governance_token);
    voting_system.set_proposal_manager(proposal_manager);

    // Test voting power retrieval
    assert_eq!(voting_system.get_voting_power("alice"), 0); // Not delegated yet

    // Test delegation through voting system
    let delegate_result = voting_system.delegate_votes("alice", "alice").unwrap();
    assert!(delegate_result.success);
    assert_eq!(voting_system.get_voting_power("alice"), 1000000);

    // Test voting records
    assert_eq!(voting_system.get_voting_records("alice").len(), 0);
    assert_eq!(voting_system.get_active_proposals().len(), 0);
    assert_eq!(voting_system.get_completed_proposals().len(), 0);
}

#[test]
fn test_voting_config_validation() {
    let mut voting_system = VotingSystemContract::new(
        "governance_token".to_string(),
        "proposal_manager".to_string(),
        VotingConfig::default(),
    );

    // Valid config update
    let valid_config = VotingConfig {
        min_voting_period: 50,
        max_voting_period: 200,
        min_voting_delay: 5,
        max_voting_delay: 20,
        proposal_threshold_percentage: 200,
        quorum_percentage: 3000,
        vote_differential: 1000,
        late_quorum_extension: 50,
    };

    let result = voting_system.update_config(valid_config).unwrap();
    assert!(result.success);

    // Invalid config - min > max voting period
    let invalid_config = VotingConfig {
        min_voting_period: 200,
        max_voting_period: 100,
        min_voting_delay: 5,
        max_voting_delay: 20,
        proposal_threshold_percentage: 200,
        quorum_percentage: 3000,
        vote_differential: 1000,
        late_quorum_extension: 50,
    };

    let result = voting_system.update_config(invalid_config).unwrap();
    assert!(!result.success);

    // Invalid config - quorum > 100%
    let invalid_config = VotingConfig {
        min_voting_period: 50,
        max_voting_period: 200,
        min_voting_delay: 5,
        max_voting_delay: 20,
        proposal_threshold_percentage: 200,
        quorum_percentage: 15000, // > 10000 (100%)
        vote_differential: 1000,
        late_quorum_extension: 50,
    };

    let result = voting_system.update_config(invalid_config).unwrap();
    assert!(!result.success);
}
