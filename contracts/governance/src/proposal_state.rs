use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use secret_toolkit::snip20::batch::SendAction;
use shade_protocol::governance::{
    proposal::{Proposal, ProposalStatus},
    vote::VoteTally,
};
use shade_protocol::utils::generic_response::ResponseStatus;

// Proposals
pub static PROPOSAL_KEY: &[u8] = b"proposals";
pub static PROPOSAL_VOTE_DEADLINE_KEY: &[u8] = b"proposal_vote_deadline_key";
pub static PROPOSAL_FUNDING_DEADLINE_KEY: &[u8] = b"proposal_funding_deadline_key";
pub static PROPOSAL_STATUS_KEY: &[u8] = b"proposal_status_key";
pub static PROPOSAL_RUN_KEY: &[u8] = b"proposal_run_key";
pub static PROPOSAL_FUNDING_KEY: &[u8] = b"proposal_funding_key";
pub static PROPOSAL_FUNDING_BATCH_KEY: &[u8] = b"proposal_funding_batch_key";
pub static PROPOSAL_VOTES_KEY: &str = "proposal_votes";
pub static TOTAL_PROPOSAL_VOTES_KEY: &[u8] = b"total_proposal_votes";
pub static TOTAL_PROPOSAL_KEY: &[u8] = b"total_proposals";

// Total proposal counter
pub fn total_proposals_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, TOTAL_PROPOSAL_KEY)
}

pub fn total_proposals_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, TOTAL_PROPOSAL_KEY)
}

// Individual proposals
pub fn proposal_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Proposal> {
    bucket_read(PROPOSAL_KEY, storage)
}

pub fn proposal_w<S: Storage>(storage: &mut S) -> Bucket<S, Proposal> {
    bucket(PROPOSAL_KEY, storage)
}

// Proposal funding deadline
pub fn proposal_funding_deadline_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, u64> {
    bucket_read(PROPOSAL_FUNDING_DEADLINE_KEY, storage)
}

pub fn proposal_funding_deadline_w<S: Storage>(storage: &mut S) -> Bucket<S, u64> {
    bucket(PROPOSAL_FUNDING_DEADLINE_KEY, storage)
}

// Proposal voting deadline
pub fn proposal_voting_deadline_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, u64> {
    bucket_read(PROPOSAL_VOTE_DEADLINE_KEY, storage)
}

pub fn proposal_voting_deadline_w<S: Storage>(storage: &mut S) -> Bucket<S, u64> {
    bucket(PROPOSAL_VOTE_DEADLINE_KEY, storage)
}

// Proposal status
pub fn proposal_status_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, ProposalStatus> {
    bucket_read(PROPOSAL_STATUS_KEY, storage)
}

pub fn proposal_status_w<S: Storage>(storage: &mut S) -> Bucket<S, ProposalStatus> {
    bucket(PROPOSAL_STATUS_KEY, storage)
}

// Proposal total funding
pub fn proposal_funding_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(PROPOSAL_FUNDING_KEY, storage)
}

pub fn proposal_funding_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(PROPOSAL_FUNDING_KEY, storage)
}

// Proposal funding batch
pub fn proposal_funding_batch_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Vec<SendAction>> {
    bucket_read(PROPOSAL_FUNDING_BATCH_KEY, storage)
}

pub fn proposal_funding_batch_w<S: Storage>(storage: &mut S) -> Bucket<S, Vec<SendAction>> {
    bucket(PROPOSAL_FUNDING_BATCH_KEY, storage)
}

// Proposal run status - will be available after proposal is run
pub fn proposal_run_status_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, ResponseStatus> {
    bucket_read(PROPOSAL_RUN_KEY, storage)
}

pub fn proposal_run_status_w<S: Storage>(storage: &mut S) -> Bucket<S, ResponseStatus> {
    bucket(PROPOSAL_RUN_KEY, storage)
}

// Individual proposal user votes
pub fn proposal_votes_r<S: Storage>(
    storage: &S,
    proposal: Uint128,
) -> ReadonlyBucket<S, VoteTally> {
    bucket_read(
        (proposal.to_string() + PROPOSAL_VOTES_KEY).as_bytes(),
        storage,
    )
}

pub fn proposal_votes_w<S: Storage>(storage: &mut S, proposal: Uint128) -> Bucket<S, VoteTally> {
    bucket(
        (proposal.to_string() + PROPOSAL_VOTES_KEY).as_bytes(),
        storage,
    )
}

// Total proposal votes
pub fn total_proposal_votes_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, VoteTally> {
    bucket_read(TOTAL_PROPOSAL_VOTES_KEY, storage)
}

pub fn total_proposal_votes_w<S: Storage>(storage: &mut S) -> Bucket<S, VoteTally> {
    bucket(TOTAL_PROPOSAL_VOTES_KEY, storage)
}
