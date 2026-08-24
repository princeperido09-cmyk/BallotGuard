#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// Data keys used for contract storage access
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    ProposalCount,
    Proposal(u32),
    Voted(Address, u32),
}

// Data structure representing a proposal ballot
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub yes_votes: u32,
    pub no_votes: u32,
    pub active: bool,
}

#[contract]
pub struct BallotGuardContract;

#[contractimpl]
impl BallotGuardContract {
    /// Initializes the ballot guard contract and stores the admin address
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);
    }

    /// Creates a new proposal ballot (Admin only)
    pub fn create_proposal(env: Env, admin: Address) -> u32 {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Unauthorized admin");
        }

        let mut count: u32 = env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0);
        count += 1;

        let proposal = Proposal {
            id: count,
            yes_votes: 0,
            no_votes: 0,
            active: true,
        };

        env.storage().persistent().set(&DataKey::Proposal(count), &proposal);
        env.storage().instance().set(&DataKey::ProposalCount, &count);

        count
    }

    /// Core MVP Flow: Validates voter identity, checks double-voting, and increments vote tallies
    pub fn cast_vote(env: Env, voter: Address, proposal_id: u32, approve: bool) {
        // Require explicit authorization from the voter
        voter.require_auth();

        // Ensure proposal exists and is active
        let proposal_key = DataKey::Proposal(proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&proposal_key)
            .expect("Proposal does not exist");

        if !proposal.active {
            panic!("Proposal is inactive");
        }

        // Check if voter has already submitted a ballot for this proposal
        let vote_key = DataKey::Voted(voter.clone(), proposal_id);
        if env.storage().persistent().has(&vote_key) {
            panic!("Voter has already cast a ballot for this proposal");
        }

        // Tally vote on-chain
        if approve {
            proposal.yes_votes += 1;
        } else {
            proposal.no_votes += 1;
        }

        // Record vote status and update proposal state
        env.storage().persistent().set(&vote_key, &true);
        env.storage().persistent().set(&proposal_key, &proposal);

        // Emit on-chain audit event
        let topics = (symbol_short!("vote"), proposal_id);
        env.events().publish(topics, (voter, approve));
    }

    /// Query proposal details and current live vote counts
    pub fn get_proposal(env: Env, proposal_id: u32) -> Proposal {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("Proposal does not exist")
    }
}

#[cfg(test)]
mod test;