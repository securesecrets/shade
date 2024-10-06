use s_toolkit::{permit::Permit, utils::types::Contract};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{c_std, s_toolkit, schemars, serde};

use crate::{
    state::{
        AdminShareInfo,
        GlobalSponsorDisplayRequestListState,
        RewardsDistInfo,
        TierState,
        UnclaimedDistInfo,
        UserRewardsLog,
        Validator,
        WinningSequence,
    },
    viewing_key::ViewingKey,
};
use c_std::{Addr, Binary, Uint128};

//////////////////////////////////////////////////////////////// Instantiation message ////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct InstantiateMsg {
    /// optional admin address, info.sender if missing
    pub admins: Option<Vec<Addr>>,
    /// optional triggerer address, info.sender if missing
    pub triggerers: Option<Vec<Addr>>,
    /// optional reviewer address, info.sender if missing
    pub reviewers: Option<Vec<Addr>>,
    /// optional triggerer revenue share for ending round and unbonding batches
    pub triggerer_share_percentage: u64,
    /// denomination of the coin this contract delegates
    pub denom: String,
    /// Pseudorandom number generator seed
    pub prng_seed: Binary,
    /// list of all the validators, this contract will delegate to
    pub validator: Vec<ValidatorInfo>,
    /// time in seconds taken by this chain to unbond the tokens delegated
    pub unbonding_duration: u64,
    /// time in seconds taken before users can claim rewards
    pub round_duration: u64,
    /// Rewards distribution between tiers stats
    pub rewards_distribution: RewardsDistInfo,
    /// price to buy on ticket
    pub ticket_price: Uint128,
    /// time taken after the round is ended before rewards are deemed unclaimable
    pub rewards_expiry_duration: u64,
    /// helps determine the number of decimals in a percentage
    pub common_divisor: u64,
    /// total admin share out of 100% of the total staking rewards
    pub total_admin_share: u64,
    /// rewards send to shade protocol of the total rewards
    pub shade_percentage_share: u64,
    /// rewards send to galactic_pools dao of the total rewards
    pub galactic_pools_percentage_share: u64,
    /// shade protocol's rewards recieving address
    pub shade_rewards_address: Addr,
    /// galactic_pools's rewards recieving address
    pub galactic_pools_rewards_address: Addr,
    /// grand-prize withdraw address
    pub grand_prize_address: Addr,
    /// percentage of unclaimed rewards that are kept to increase the number of rewards
    pub reserve_percentage: u64,
    /// Admins can only deposit sponsor deposits with approved title and message
    pub is_sponosorship_admin_controlled: bool,
    /// How long it takes before next batch is unbonded
    pub unbonding_batch_duration: u64,
    /// optional minimum amount that can be deposited
    pub minimum_deposit_amount: Option<Uint128>,
    /// setting number of number_of_tickers that can be run on txn send to avoid potential errors
    pub number_of_tickers_per_transaction: Uint128,
    /// fee paid by sponsors to edit there message title
    pub sponsor_msg_edit_fee: Option<Uint128>,
    /// exp contract
    pub exp_contract: Option<ExpContract>,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, JsonSchema)]
pub struct ValidatorInfo {
    pub address: String,
    pub weightage: u64,
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, JsonSchema)]
pub struct ExpContract {
    pub contract: s_toolkit::utils::types::Contract,
    pub vk: String,
}

//////////////////////////////////////////////////////////////// Handle message ////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    //USER
    Deposit {},
    RequestWithdraw {
        amount: Uint128,
    },
    Withdraw {
        amount: Uint128,
    },
    ClaimRewards {},
    CreateViewingKey {
        entropy: String,
    },
    SetViewingKey {
        key: String,
    },
    /// disallow the use of a permit
    RevokePermit {
        /// name of the permit that is no longer valid
        permit_name: String,
    },

    //SPONSORS
    Sponsor {
        title: Option<String>,
        message: Option<String>,
    },
    SponsorRequestWithdraw {
        amount: Uint128,
    },

    SponsorWithdraw {
        amount: Uint128,
    },
    SponsorMessageEdit {
        title: Option<String>,
        message: Option<String>,
        delete_title: bool,
        delete_message: bool,
    },

    // Admin
    UpdateConfig {
        unbonding_batch_duration: Option<u64>,
        unbonding_duration: Option<u64>,
        minimum_deposit_amount: Option<Uint128>,
        exp_contract: Option<ExpContract>,
    },
    UpdateRound {
        duration: Option<u64>,
        rewards_distribution: Option<RewardsDistInfo>,
        ticket_price: Option<Uint128>,
        rewards_expiry_duration: Option<u64>,
        admin_share: Option<AdminShareInfo>,
        triggerer_share_percentage: Option<u64>,
        shade_rewards_address: Option<Addr>,
        galactic_pools_rewards_address: Option<Addr>,
        grand_prize_address: Option<Addr>,
        unclaimed_distribution: Option<UnclaimedDistInfo>,
    },

    AddAdmin {
        admin: Addr,
    },
    RemoveAdmin {
        admin: Addr,
    },

    AddTriggerer {
        triggerer: Addr,
    },
    RemoveTriggerer {
        triggerer: Addr,
    },

    AddReviewer {
        reviewer: Addr,
    },
    RemoveReviewer {
        reviewer: Addr,
    },

    UpdateValidatorSet {
        updated_validator_set: Vec<ValidatorInfo>,
    },
    RebalanceValidatorSet {},
    /// set contract status level to determine which functions are allowed.  StopTransactions
    /// status prevent mints, burns, sends, and transfers, but allows all other functions
    SetContractStatus {
        /// status level
        level: ContractStatus,
    },
    SetSponsorshipAccess {
        /// status level
        is_sponosorship_admin_controlled: bool,
    },
    EndRound {},
    RequestReservesWithdraw {
        amount: Uint128,
    },
    UnbondBatch {},

    ReservesWithdraw {
        amount: Uint128,
    },

    ReviewSponsors {
        decisions: Vec<Review>,
    },
    RemoveSponsorCredentials {
        decisions: Vec<RemoveSponsorCredentialsDecisions>,
    },
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Review {
    pub index: u32,
    pub is_accpeted: bool,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct RemoveSponsorCredentialsDecisions {
    pub index: u32,
    pub remove_sponsor_title: bool,
    pub remove_sponsor_message: bool,
}

//////////////////////////////////////////////////////////////// Handle Answer ////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    // Native
    Initialize {
        status: ResponseStatus,
    },
    Deposit {
        status: ResponseStatus,
    },
    Sponsor {
        status: ResponseStatus,
    },
    SponsorMessageEdit {
        status: ResponseStatus,
    },
    RemoveSponsorCredentials {
        status: ResponseStatus,
    },
    ReviewSponsorMessages {
        status: ResponseStatus,
    },
    Redelegate {
        status: ResponseStatus,
    },
    RequestWithdraw {
        status: ResponseStatus,
    },
    RequestWithdrawSponsor {
        status: ResponseStatus,
    },
    RequestAdminWithdraw {
        status: ResponseStatus,
    },
    Withdraw {
        status: ResponseStatus,
    },
    UnbondBatch {
        status: ResponseStatus,
    },
    SponsorWithdraw {
        status: ResponseStatus,
    },
    ReservesWithdraw {
        status: ResponseStatus,
    },

    TriggeringCostWithdraw {
        status: ResponseStatus,
    },
    // Base
    Transfer {
        status: ResponseStatus,
    },
    Send {
        status: ResponseStatus,
    },
    Burn {
        status: ResponseStatus,
    },
    RegisterReceive {
        status: ResponseStatus,
    },
    CreateViewingKey {
        key: ViewingKey,
    },
    SetViewingKey {
        status: ResponseStatus,
    },
    RevokePermit {
        status: ResponseStatus,
    },

    // Other
    EndRound {
        status: ResponseStatus,
    },
    AddAdmin {
        status: ResponseStatus,
    },
    RemoveAdmin {
        status: ResponseStatus,
    },
    AddTriggerer {
        status: ResponseStatus,
    },
    RemoveTriggerer {
        status: ResponseStatus,
    },
    AddReviewer {
        status: ResponseStatus,
    },
    RemoveReviewer {
        status: ResponseStatus,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    UpdateRound {
        status: ResponseStatus,
    },
    ChangeAdmin {
        status: ResponseStatus,
    },
    ChangeAdminShare {
        status: ResponseStatus,
    },
    ChangeTriggerer {
        status: ResponseStatus,
    },
    ChangeReviewer {
        status: ResponseStatus,
    },
    ChangeTriggererShare {
        status: ResponseStatus,
    },
    ChangeValidator {
        status: ResponseStatus,
    },

    ChangeUnbondingTime {
        status: ResponseStatus,
    },
    SetContractStatus {
        status: ResponseStatus,
    },

    ClaimRewards {
        status: ResponseStatus,
        winning_amount: Uint128,
    },

    UpdateValidatorSet {
        status: ResponseStatus,
    },
    RebalanceValidatorSet {
        status: ResponseStatus,
    },
    SetSponsorshipAccess {
        status: ResponseStatus,
    },
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

//////////////////////////////////////////////////////////////// Query Message ////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    //PUBLIC
    /// display the contract's configuration
    ContractConfig {},
    ContractStatus {},

    Round {},

    CurrentRewards {},
    PoolState {},
    PoolStateLiquidityStats {},
    PoolStateLiquidityStatsSpecific {
        round_index: u64,
    },

    RewardsStats {},

    PastRecords {},
    PastAllRecords {},

    //AUTHENTICATED
    Delegated {
        address: String,
        key: String,
    },

    Withdrawable {
        address: String,
        key: String,
    },
    Unbondings {
        address: String,
        key: String,
    },
    Liquidity {
        key: String,
        address: String,
        round_index: u64,
    },

    SponsorInfo {
        key: String,
        address: String,
    },
    SponsorWithdrawable {
        key: String,
        address: String,
    },
    SponsorUnbondings {
        key: String,
        address: String,
    },

    Records {
        address: String,
        page_size: Option<u32>,
        start_page: Option<u32>,
        key: String,
    },
    SponsorMessageRequestCheck {
        page_size: Option<u32>,
        start_page: Option<u32>,
    },
    Sponsors {
        page_size: Option<u32>,
        start_page: Option<u32>,
    },
    /// perform queries by passing permits instead of viewing keys
    WithPermit {
        /// permit used to verify querier identity
        permit: Permit<GalacticPoolsPermissions>,
        /// query to perform
        query: QueryWithPermit,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&String>, ViewingKey) {
        match self {
            Self::Delegated { address, key } => (vec![address], ViewingKey(key.clone())),
            Self::Unbondings { address, key } => (vec![address], ViewingKey(key.clone())),
            Self::Withdrawable { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::Liquidity { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::SponsorInfo { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::SponsorWithdrawable { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
            Self::SponsorUnbondings { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
            Self::Records { address, key, .. } => (vec![address], ViewingKey(key.clone())),

            _ => panic!("This query type does not require authentication"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GalacticPoolsPermissions {
    Delegated,
    UserInfo,
    SponsorInfo,
    Liquidity,
    Withdrawable,
    SponsorWithdrawable,
    Unbondings,
    SponsorUnbondings,
    Records,
    /// Owner permission indicates that the bearer of this permit should be granted all
    /// the access of the creator/signer of the permit.  SNIP-721 uses this to grant
    /// viewing access to all data that the permit creator owns and is whitelisted for.
    /// For SNIP-721 use, a permit with Owner permission should NEVER be given to
    /// anyone else.  If someone wants to share private data, they should whitelist
    /// the address they want to share with via a SetWhitelistedApproval tx, and that
    /// address will view the data by creating their own permit with Owner permission
    Owner,
}

/// queries using permits instead of viewing keys
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    Delegated {},
    UserInfo {},
    SponsorInfo {},
    Liquidity {
        round_index: u64,
    },

    Withdrawable {},
    SponsorWithdrawable {},
    Unbondings {},
    SponsorUnbondings {},
    Records {
        page_size: Option<u32>,
        start_page: Option<u32>,
    },
    Test {},
}

//////////////////////////////////////////////////////////////// Query Answer/Resposes ////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContractConfigResponse {
    pub admins: Vec<Addr>,
    pub triggerers: Vec<Addr>,
    pub reviewers: Vec<Addr>,
    pub denom: String,
    pub contract_address: Addr,
    pub validators: Vec<Validator>,
    pub next_unbonding_batch_time: u64,
    pub next_unbonding_batch_amount: Uint128,
    pub unbonding_batch_duration: u64,
    pub unbonding_duration: u64,
    pub minimum_deposit_amount: Option<Uint128>,
    pub exp_contract: Option<Contract>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContractStatusResponse {
    pub status: ContractStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RoundResponse {
    pub duration: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub rewards_distribution: RewardsDistInfo,
    pub current_round_index: u64,
    pub ticket_price: Uint128,
    pub rewards_expiry_duration: u64,
    pub admin_share: AdminShareInfo,
    pub triggerer_share_percentage: u64,
    pub unclaimed_distribution: UnclaimedDistInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DelegatedResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]

pub struct LiquidityResponse {
    //total liq
    pub total_liq: Uint128,
    //total tickets
    pub total_tickets: Uint128,
    //ticket price
    pub ticket_price: Uint128,
    //user liq
    pub user_liq: Uint128,
    //user tickets
    pub user_tickets: Uint128,
    pub tickets_used: Uint128,
    pub expiry_date: Option<u64>,
    pub total_rewards: Uint128,
    pub unclaimed_rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]

pub struct UserInfoResponse {
    pub amount_delegated: Uint128,
    pub amount_unbonding: Uint128,
    pub starting_round: Option<u64>,
    pub total_won: Uint128,
    pub last_claim_rewards_round: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SponsorInfoResponse {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]

pub struct WithdrawablelResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnbondingsResponse {
    pub vec: Vec<RequestWithdrawQueryResponse>,
    pub len: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy)]
pub struct RequestWithdrawQueryResponse {
    pub amount: Uint128,
    pub batch_index: u64,
    // if batch haven't unbonded yet
    pub next_batch_unbonding_time: Option<u64>,
    // if batch is already unbonded
    pub unbonding_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RecordsResponse {
    pub vec: Vec<UserRewardsLog>,
    pub len: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SponsorMessageRequestResponse {
    pub vec: Vec<GlobalSponsorDisplayRequestListState>,
    pub len: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SponsorsResponse {
    pub vec: Vec<SponsorDisplayInfo>,
    pub len: u32,
}
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct SponsorDisplayInfo {
    pub amount_sponsored: Uint128,
    pub title: Option<String>,
    pub message: Option<String>,
    pub addr_list_index: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CurrentRewardsResponse {
    pub rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PoolStateInfoResponse {
    pub total_delegated: Uint128,
    pub rewards_returned_to_contract: Uint128,
    pub total_reserves: Uint128,
    pub total_sponsored: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RewardStatsResponse {
    pub distribution_per_tiers: TierState,
    pub ticket_price: Uint128,
    pub winning_sequence: WinningSequence,
    pub rewards_expiration_date: Option<u64>,
    pub total_rewards: Uint128,
    pub total_claimed: Uint128,
    pub total_exp: Option<Uint128>,
    pub total_exp_claimed: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SponsorMessageReqResponse {
    pub vec: Vec<GlobalSponsorDisplayRequestListState>,
    pub len: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]

pub struct PoolStateLiquidityStatsResponse {
    pub total_liquidity: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PastRecordsResponse {
    pub past_rewards: Vec<(u64, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PastAllRecordsResponse {
    pub past_rewards: Vec<(u64, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UserPastRecordsResponse {
    pub winning_history: Vec<(u64, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UserAllPastRecordsResponse {
    pub winning_history: Vec<(u64, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CreateViewingKeyResponse {
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ViewingKeyErrorResponse {
    pub msg: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Normal,
    StopTransactions,
    StopAll,
}

impl ContractStatus {
    /// Returns u8 representation of the ContractStatus
    pub fn to_u8(&self) -> u8 {
        match self {
            ContractStatus::Normal => 0,
            ContractStatus::StopTransactions => 1,
            ContractStatus::StopAll => 2,
        }
    }
}

// Take a Vec<u8> and pad it up to a multiple of `block_size`, using spaces at the end.
pub fn space_pad(block_size: usize, message: &mut Vec<u8>) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}
