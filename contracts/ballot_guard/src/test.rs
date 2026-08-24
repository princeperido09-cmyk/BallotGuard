#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_mvp_vote_execution_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BallotGuardContract);
    let client = BallotGuardContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let voter = Address::generate(&env);

    client.initialize(&admin);
    let proposal_id = client.create_proposal(&admin);

    // Cast vote: Approve (true)
    client.cast_vote(&voter, &proposal_id, &true);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.yes_votes, 1);
    assert_eq!(proposal.no_votes, 0);
}

#[test]
#[should_panic(expected = "Voter has already cast a ballot for this proposal")]
fn test_double_voting_prevention_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BallotGuardContract);
    let client = BallotGuardContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let voter = Address::generate(&env);

    client.initialize(&admin);
    let proposal_id = client.create_proposal(&admin);

    // First vote succeeds
    client.cast_vote(&voter, &proposal_id, &true);
    
    // Duplicate vote must trigger panic
    client.cast_vote(&voter, &proposal_id, &false);
}

#[test]
fn test_state_verification_after_multiple_votes() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BallotGuardContract);
    let client = BallotGuardContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let voter3 = Address::generate(&env);

    client.initialize(&admin);
    let proposal_id = client.create_proposal(&admin);

    client.cast_vote(&voter1, &proposal_id, &true);
    client.cast_vote(&voter2, &proposal_id, &true);
    client.cast_vote(&voter3, &proposal_id, &false);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.yes_votes, 2);
    assert_eq!(proposal.no_votes, 1);
    assert_eq!(proposal.active, true);
}

#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_unauthorized_proposal_creation_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BallotGuardContract);
    let client = BallotGuardContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.initialize(&admin);

    // Attacker tries to create proposal
    client.create_proposal(&attacker);
}

#[test]
#[should_panic(expected = "Proposal does not exist")]
fn test_vote_nonexistent_proposal_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BallotGuardContract);
    let client = BallotGuardContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let voter = Address::generate(&env);

    client.initialize(&admin);

    // Vote on proposal 999 which hasn't been created
    client.cast_vote(&voter, &999u32, &true);
}