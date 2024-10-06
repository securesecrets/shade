use c_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{c_std, schemars};

use crate::msg::ExpContract;

//////////////////////////////////////////////////////////////// CONFIG //////////////////////////////////////////////////////////////////

/// Configuration Information
#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
pub struct ConfigInfo {
    /// contract admin's  address
    pub admins: Vec<Addr>,
    /// contract triggerer's canonical address
    pub triggerers: Vec<Addr>,
    /// contract reviewer's canonical address
    pub reviewers: Vec<Addr>,
    /// helps determine the number of decimals in a percentage
    pub common_divisor: u64,
    /// denomination of the coin this contract delegates
    pub denom: String,
    /// Pseudorandom number generator seed
    pub prng_seed: Vec<u8>,
    /// canonical address of this contract
    pub contract_address: Addr,
    /// list of all the validators, this contract will delegate to
    pub validators: Vec<Validator>,
    /// index of the next validator, contract will delegate to
    pub next_validator_for_delegation: u8,
    /// index of the nect validator used for unbonding
    pub next_validator_for_unbonding: u8,
    /// index of the next batch going to unbond
    pub next_unbonding_batch_index: u64,
    /// time next batch unbond
    pub next_unbonding_batch_time: u64,
    /// amount to be unbonded next batch
    pub next_unbonding_batch_amount: Uint128,
    /// time in seconds it takes before next batch is unbonded
    pub unbonding_batch_duration: u64,
    /// time in seconds taken by this chain to unbond the tokens delegated
    pub unbonding_duration: u64,
    /// optional minimum amount that can be deposited
    pub minimum_deposit_amount: Option<Uint128>,
    /// contract status
    pub status: u8,
    /// fee paid by sponsors to edit there message title
    pub sponsor_msg_edit_fee: Option<Uint128>,
    /// exp contract
    pub exp_contract: Option<ExpContract>,
}
/// Validator Information
/// Config -> Validator
#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, Default)]
pub struct Validator {
    /// validator address
    pub address: String,
    /// amount delegated to this validator
    pub delegated: Uint128,
    /// % of amount must be delegated to this validator
    pub weightage: u64,
    /// delegated/(total_delegated*weightage)
    pub percentage_filled: u64,
}

//////////////////////////////////////////////////////////////// Round  //////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RoundInfo {
    /// entropy
    pub entropy: Vec<u8>,
    /// duration
    pub seed: Vec<u8>,
    /// duration of this round in # of seconds
    pub duration: u64,
    /// start time of current round
    pub start_time: u64,
    /// ending time of current round
    pub end_time: u64,
    /// rewards distribution between each tier
    pub rewards_distribution: RewardsDistInfo,
    /// index of the current round
    pub current_round_index: u64,
    /// price per one ticket
    pub ticket_price: Uint128,
    /// duration after round ends after which prizes are expired.
    pub rewards_expiry_duration: u64,
    /// % of rewards that are directed to admin
    pub admin_share: AdminShareInfo,
    /// % of rewards for triggerer
    pub triggerer_share_percentage: u64,
    /// shade's dao address
    pub shade_rewards_address: Addr,
    /// galacticpool's dao address
    pub galactic_pools_rewards_address: Addr,
    /// grand-prize contract address
    pub grand_prize_address: Addr,
    /// round when last time expired rewards were claimed
    pub unclaimed_rewards_last_claimed_round: Option<u64>,
    /// distribution of unclaimed rewards
    pub unclaimed_distribution: UnclaimedDistInfo,
    /// setting number of number_of_tickers that can be run on txn send to avoid potential errors
    pub number_of_tickers_per_transaction: Uint128,
}

/// pre-defined rewards distribution information between each tier
/// Round -> RewardsDistInfo
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, JsonSchema)]
pub struct RewardsDistInfo {
    pub tier_0: DistInfo,
    pub tier_1: DistInfo,
    pub tier_2: DistInfo,
    pub tier_3: DistInfo,
    pub tier_4: DistInfo,
    pub tier_5: DistInfo,
}
/// pre-defined rewards distribution information
/// Round -> RewardsDistInfo -> DistInfo
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, JsonSchema)]
pub struct DistInfo {
    pub total_number_of_winners: Uint128,
    pub percentage_of_rewards: u64,
}

/// % of rewards that are directed to admin
/// Round -> AdminShareInfo
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct AdminShareInfo {
    pub total_percentage_share: u64,
    pub shade_percentage_share: u64,
    pub galactic_pools_percentage_share: u64,
}

///  distribution of unclaimed rewards
/// Round -> UnclaimedDistInfo
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct UnclaimedDistInfo {
    /// % of rewards that are restaked and are used to increase overall all rewards
    pub reserves_percentage: u64,
    /// % of rewards that are added to the winning prizes
    pub propagate_percentage: u64,
}

//////////////////////////////////////////////////////////////// POOL STATE //////////////////////////////////////////////////////////////////

/// Global state in this Pool contract
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct PoolState {
    pub total_delegated: Uint128,
    /// token(s) that are auto-claimed when contract deposits to validator
    pub rewards_returned_to_contract: Uint128,
    pub total_reserves: Uint128,
    pub total_sponsored: Uint128,
    pub unbonding_batches: Vec<u64>,
}

/// Global liquidity state in this Pool contract for nth round.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy, Default)]
pub struct PoolLiqState {
    pub total_delegated: Option<Uint128>,
    pub total_liquidity: Option<Uint128>,
}

//////////////////////////////////////////////////////////////// REWARDS STATE //////////////////////////////////////////////////////////////////

/// State of the rewards for specific nth round.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct RewardsState {
    pub distribution_per_tiers: TierState,
    pub ticket_price: Uint128,
    pub winning_sequence: WinningSequence,
    pub rewards_expiration_date: Option<u64>,
    pub total_rewards: Uint128,
    pub total_claimed: Uint128,
    pub total_exp: Option<Uint128>,
    pub total_exp_claimed: Option<Uint128>,
}
/// State of the rewards per tier
/// RewardsState -> TierState
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct TierState {
    pub tier_0: RewardsClaimed,
    pub tier_1: RewardsClaimed,
    pub tier_2: RewardsClaimed,
    pub tier_3: RewardsClaimed,
    pub tier_4: RewardsClaimed,
    pub tier_5: RewardsClaimed,
}

/// State of the claimed rewards
/// RewardsState -> TierState -> RewardsClaimed
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct RewardsClaimed {
    pub num_of_rewards: Uint128,
    pub claimed: RewardsPerTierInfo,
}

/// Winning Sequence of the prize
/// RewardsState -> WinningSequence
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct WinningSequence {
    pub tier_0: DigitsInfo,
    pub tier_1: DigitsInfo,
    pub tier_2: DigitsInfo,
    pub tier_3: DigitsInfo,
    pub tier_4: DigitsInfo,
    pub tier_5: DigitsInfo,
}

/// Range consists of Information about Range of digit for specific tier and winning digit
/// Range otherwise known as Difficulty. Bigger the range more difficult it will be to win prize
/// Winning Number: between 0 and range
/// RewardsState -> WinningSequence
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct DigitsInfo {
    pub range: Uint128,
    pub winning_number: Uint128,
}

//////////////////////////////////////////////////////////////// USER INFO AND STATE //////////////////////////////////////////////////////////////////

/// User's State
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct UserInfo {
    pub amount_delegated: Uint128,
    pub amount_withdrawable: Uint128,
    pub amount_unbonding: Uint128,
    pub unbonding_batches: Vec<u64>,
    pub starting_round: Option<u64>,
    pub total_won: Uint128,
    pub last_claim_rewards_round: Option<u64>,
}

/// State of user liquidity information
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy, Default)]
pub struct UserLiqState {
    pub amount_delegated: Option<Uint128>,
    pub liquidity: Option<Uint128>,
    pub tickets_used: Option<Uint128>,
}

/// A log user winnings
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy, Default)]
pub struct UserRewardsLog {
    pub round: u64,
    pub tickets: Uint128,
    pub ticket_price: Uint128,
    pub rewards_per_tier: Option<TierLog>,
    pub liquidity: Option<Uint128>,
    pub total_amount_won: Option<Uint128>,
    pub total_exp_gained: Option<Uint128>,
}
/// A log of user winnings -> per tier
/// UserRewardsLog -> TierLog
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy, Default)]
pub struct TierLog {
    pub tier_5: RewardsPerTierInfo,
    pub tier_4: RewardsPerTierInfo,
    pub tier_3: RewardsPerTierInfo,
    pub tier_2: RewardsPerTierInfo,
    pub tier_1: RewardsPerTierInfo,
    pub tier_0: RewardsPerTierInfo,
}

/// A log user winnings -> per tier -> Information
/// UserRewardsLog -> TierLog -> RewardsPerTierInfo
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy, Default)]
pub struct RewardsPerTierInfo {
    pub num_of_rewards_claimed: Uint128,
    pub reward_per_match: Uint128,
}

//////////////////////////////////////////////////////////////// SPONOSR INFO AND STATE //////////////////////////////////////////////////////////////////

/// State of Sponsor information
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct SponsorInfo {
    pub amount_sponsored: Uint128,
    pub amount_withdrawable: Uint128,
    pub amount_unbonding: Uint128,
    pub title: Option<String>,
    pub message: Option<String>,
    /// index of the sponsors in storage
    pub addr_list_index: Option<u32>,
    pub unbonding_batches: Vec<u64>,
    pub has_requested: bool,
}

/// To avoid spam contract will only allow inspected titles and messages.
/// This struct contains a list of all the titles/messages that sponsors have requested to display
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct GlobalSponsorDisplayRequestListState {
    pub addr: String,
    pub index: Option<u32>,
    pub deque_store_index: Option<u32>,
    pub title: Option<String>,
    pub message: Option<String>,
}

// Helps provide a unique id to a sponsor
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct GlobalSponsorState {
    //total sponsors are offset minus # of empty_slots
    pub offset: u32,
    pub empty_slots: Vec<u32>,
}

//////////////////////////////////////////////////////////////// ADMIN //////////////////////////////////////////////////////////////////

/// state of amount available for admins to withdraw
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct AdminWithdraw {
    pub amount_withdrawable: Uint128,
}

/// Information about specific nth unbonding batch
#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
pub struct UnbondingBatch {
    pub unbonding_time: Option<u64>,
    pub amount: Option<Uint128>,
}

//////////////////////////////////////////////////////////////// USER + SPONSOR + ADMIN //////////////////////////////////////////////////////////////////

/// Request withdraw struct
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy)]
pub struct RequestWithdraw {
    pub amount: Uint128,
    pub unbonding_batch_index: u64,
    pub approximate_unbonding_time: u64,
}

//Use as value holder when claiming rewards
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy)]
pub struct TierCounter {
    pub tier_5: Uint128,
    pub tier_4: Uint128,
    pub tier_3: Uint128,
    pub tier_2: Uint128,
    pub tier_1: Uint128,
    pub tier_0: Uint128,
}
