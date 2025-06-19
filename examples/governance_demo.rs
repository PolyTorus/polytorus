//! Governance System Demo
//!
//! This example demonstrates the complete governance system including:
//! - Governance token with delegation
//! - Proposal creation and management
//! - Voting system with comprehensive features

use polytorus::smart_contract::{
    governance_token::GovernanceTokenContract,
    proposal_manager::{ProposalManagerContract, ProposalState, VoteChoice},
    voting_system::{VotingConfig, VotingSystemContract},
};

fn main() {
    println!("🏛️  Polytorus Governance System Demo");
    println!("=====================================\n");

    // Step 1: Initialize Governance Token
    println!("📊 Step 1: Creating Governance Token");
    let mut governance_token = GovernanceTokenContract::new(
        "Polytorus Governance Token".to_string(),
        "PGT".to_string(),
        18,
        100_000_000, // 100M total supply
        "foundation".to_string(),
    );

    println!(
        "✅ Created governance token: {} ({})",
        governance_token.name(),
        governance_token.symbol()
    );
    println!(
        "   Total Supply: {} tokens",
        governance_token.total_supply()
    );
    println!(
        "   Foundation Balance: {} tokens\n",
        governance_token.balance_of("foundation")
    );

    // Step 2: Distribute Tokens to Community
    println!("💰 Step 2: Distributing Tokens to Community");

    let distributions = vec![
        ("alice", 15_000_000, "Early Contributor"),
        ("bob", 12_000_000, "Developer"),
        ("charlie", 10_000_000, "Validator"),
        ("david", 8_000_000, "Community Member"),
        ("eve", 5_000_000, "Researcher"),
    ];

    for (recipient, amount, role) in &distributions {
        governance_token
            .transfer("foundation", recipient, *amount)
            .unwrap();
        println!("   Transferred {amount} tokens to {recipient} ({role})");
    }

    let foundation_remaining = governance_token.balance_of("foundation");
    println!("   Foundation Remaining: {foundation_remaining} tokens\n");

    // Step 3: Setup Voting Delegation
    println!("🗳️  Step 3: Setting up Voting Delegation");

    // Each participant delegates voting power to themselves
    let participants = vec!["foundation", "alice", "bob", "charlie", "david", "eve"];
    for participant in &participants {
        governance_token.delegate(participant, participant).unwrap();
        let voting_power = governance_token.get_current_votes(participant);
        println!("   {participant} delegated to self: {voting_power} voting power");
    }

    // Some cross-delegation examples
    governance_token.delegate("eve", "alice").unwrap(); // Eve delegates to Alice
    governance_token.delegate("david", "charlie").unwrap(); // David delegates to Charlie

    println!("\n   After Cross-Delegation:");
    for participant in &participants {
        let voting_power = governance_token.get_current_votes(participant);
        if voting_power > 0 {
            println!("   {participant} has {voting_power} voting power");
        }
    }
    println!();

    // Step 4: Create Proposal Manager
    println!("📋 Step 4: Creating Proposal Management System");
    let mut proposal_manager = ProposalManagerContract::new(
        "governance_token".to_string(),
        20,         // 20 block voting delay
        200,        // 200 block voting period
        1_000_000,  // 1M token proposal threshold (1% of supply)
        25_000_000, // 25% quorum requirement
        100,        // 100 block timelock delay
    );

    println!("✅ Proposal Manager Configuration:");
    println!("   Voting Delay: 20 blocks");
    println!("   Voting Period: 200 blocks");
    println!("   Proposal Threshold: 1M tokens (1%)");
    println!("   Quorum Requirement: 25M tokens (25%)");
    println!("   Timelock Delay: 100 blocks\n");

    // Step 5: Create Voting System
    println!("🗳️  Step 5: Creating Integrated Voting System");
    let config = VotingConfig {
        min_voting_period: 100,
        max_voting_period: 500,
        min_voting_delay: 10,
        max_voting_delay: 50,
        proposal_threshold_percentage: 100, // 1%
        quorum_percentage: 2500,            // 25%
        vote_differential: 500,             // 5% minimum difference
        late_quorum_extension: 100,         // 100 block extension
    };

    let mut voting_system = VotingSystemContract::new(
        "governance_token".to_string(),
        "proposal_manager".to_string(),
        config,
    );

    // Link contracts
    voting_system.set_governance_token(governance_token.clone());
    voting_system.set_proposal_manager(proposal_manager.clone());

    println!("✅ Voting System Created with Configuration:");
    println!("   Quorum Percentage: 25%");
    println!("   Vote Differential: 5%");
    println!("   Late Quorum Extension: 100 blocks\n");

    // Step 6: Create First Proposal
    println!("📝 Step 6: Creating Protocol Upgrade Proposal");
    let proposal_result = proposal_manager.propose(
        "alice",
        "Protocol Upgrade v2.0".to_string(),
        "Proposal to upgrade Polytorus protocol to version 2.0 with improved quantum resistance and enhanced Diamond IO features. This upgrade includes:\n1. New quantum-safe cryptographic primitives\n2. Enhanced modular architecture\n3. Improved smart contract execution engine\n4. Better governance mechanisms".to_string(),
        vec!["protocol_contract".to_string(), "governance_contract".to_string()],
        vec![0, 0],
        vec![
            vec![0x01, 0x02, 0x03, 0x04], // upgrade protocol call
            vec![0x05, 0x06, 0x07, 0x08], // update governance call
        ],
        20_000_000, // Alice's voting power
    ).unwrap();

    if proposal_result.success {
        println!("✅ Proposal Created Successfully!");
        let proposal_id = u64::from_le_bytes(proposal_result.return_value.try_into().unwrap());
        println!("   Proposal ID: {proposal_id}");

        let proposal = proposal_manager.get_proposal(proposal_id).unwrap();
        println!("   Title: {}", proposal.title);
        println!("   Proposer: {}", proposal.proposer);
        println!(
            "   Current State: {:?}",
            proposal_manager.get_proposal_state(proposal_id)
        );
        println!("   Start Block: {}", proposal.start_block);
        println!("   End Block: {}", proposal.end_block);
        println!();

        // Step 7: Advance to Voting Period
        println!("⏰ Step 7: Advancing to Voting Period");
        println!("   Current Block: {}", proposal_manager.current_block());

        for i in 1..=21 {
            proposal_manager.advance_block();
            governance_token.advance_block();
            if i % 5 == 0 {
                println!(
                    "   Block {} - State: {:?}",
                    proposal_manager.current_block(),
                    proposal_manager.get_proposal_state(proposal_id)
                );
            }
        }

        println!("   Voting is now ACTIVE!\n");

        // Step 8: Cast Votes
        println!("🗳️  Step 8: Community Voting");

        // Update contracts in voting system
        voting_system.set_governance_token(governance_token.clone());
        voting_system.set_proposal_manager(proposal_manager.clone());

        let votes = vec![
            (
                "alice",
                VoteChoice::For,
                "I authored this proposal and believe it will significantly improve our protocol",
            ),
            (
                "bob",
                VoteChoice::For,
                "The technical improvements are necessary for long-term scalability",
            ),
            (
                "charlie",
                VoteChoice::Against,
                "We need more testing before such a major upgrade",
            ),
            (
                "foundation",
                VoteChoice::For,
                "This aligns with our roadmap and vision",
            ),
        ];

        for (voter, choice, reason) in votes {
            let vote_result = voting_system
                .cast_vote_with_reason(proposal_id, voter, choice, reason.to_string())
                .unwrap();

            if vote_result.success {
                let voting_power = voting_system.get_voting_power(voter);
                println!("   ✅ {voter} voted {choice:?} with {voting_power} voting power");
                println!("      Reason: \"{reason}\"");
            }
        }

        // Display current vote tally
        println!("\n📊 Current Vote Tally:");
        if let Some((for_votes, against_votes, abstain_votes)) =
            voting_system.get_proposal_votes(proposal_id)
        {
            let total_votes = for_votes + against_votes + abstain_votes;
            let quorum = voting_system.get_quorum(proposal_id);

            println!(
                "   For: {} votes ({:.1}%)",
                for_votes,
                (for_votes as f64 / total_votes as f64) * 100.0
            );
            println!(
                "   Against: {} votes ({:.1}%)",
                against_votes,
                (against_votes as f64 / total_votes as f64) * 100.0
            );
            println!(
                "   Abstain: {} votes ({:.1}%)",
                abstain_votes,
                (abstain_votes as f64 / total_votes as f64) * 100.0
            );
            println!("   Total: {total_votes} votes");
            println!("   Quorum Required: {quorum} votes");
            println!(
                "   Quorum Reached: {}",
                voting_system.is_quorum_reached(proposal_id)
            );
        }

        // Step 9: End Voting Period
        println!("\n⏰ Step 9: Ending Voting Period");
        for i in 1..=201 {
            proposal_manager.advance_block();
            if i % 50 == 0 {
                println!(
                    "   Block {} - {} blocks remaining",
                    proposal_manager.current_block(),
                    201 - i
                );
            }
        }

        let final_state = proposal_manager.get_proposal_state(proposal_id);
        println!("   Final Proposal State: {final_state:?}");

        // Step 10: Execute if Successful
        if final_state == ProposalState::Succeeded {
            println!("\n🎉 Step 10: Proposal Succeeded - Queuing for Execution");

            let queue_result = proposal_manager.queue_proposal(proposal_id).unwrap();
            if queue_result.success {
                println!("   ✅ Proposal queued for execution");
                println!("   Waiting for timelock period...");

                // Wait for timelock
                for i in 1..=101 {
                    proposal_manager.advance_block();
                    if i % 25 == 0 {
                        println!("   Timelock: {} blocks remaining", 101 - i);
                    }
                }

                // Execute proposal
                let execute_result = proposal_manager.execute_proposal(proposal_id).unwrap();
                if execute_result.success {
                    println!("   🎊 PROPOSAL EXECUTED SUCCESSFULLY!");
                    println!("   Protocol upgrade is now in effect.");
                } else {
                    println!("   ❌ Execution failed");
                }
            }
        } else {
            println!("\n❌ Proposal did not succeed");
            match final_state {
                ProposalState::Defeated => {
                    println!("   Reason: Defeated (insufficient support or quorum not reached)")
                }
                ProposalState::Canceled => println!("   Reason: Canceled by proposer"),
                _ => println!("   Reason: {final_state:?}"),
            }
        }

        // Step 11: Display Final Statistics
        println!("\n📈 Final Governance Statistics");
        println!("=============================");
        println!("Total Proposals: {}", proposal_manager.proposal_count());
        println!(
            "Active Proposals: {}",
            voting_system.get_active_proposals().len()
        );
        println!(
            "Completed Proposals: {}",
            voting_system.get_completed_proposals().len()
        );

        println!("\nToken Distribution:");
        for participant in &participants {
            let balance = governance_token.balance_of(participant);
            let voting_power = governance_token.get_current_votes(participant);
            if balance > 0 {
                println!("   {participant}: {balance} tokens, {voting_power} voting power");
            }
        }

        println!("\nVoting Records:");
        for participant in &participants {
            let records = voting_system.get_voting_records(participant);
            if !records.is_empty() {
                println!(
                    "   {} participated in {} proposals",
                    participant,
                    records.len()
                );
            }
        }
    } else {
        println!(
            "❌ Failed to create proposal: {}",
            String::from_utf8_lossy(&proposal_result.return_value)
        );
    }

    println!("\n🏁 Demo Complete!");
    println!("The Polytorus governance system successfully demonstrated:");
    println!("✅ Governance token with delegation capabilities");
    println!("✅ Comprehensive proposal management");
    println!("✅ Integrated voting system with advanced features");
    println!("✅ Complete governance workflow from proposal to execution");
}
