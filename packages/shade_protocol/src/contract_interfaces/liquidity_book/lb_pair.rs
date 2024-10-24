use crate::{
    c_std::{Addr, ContractInfo, Decimal256, Uint128, Uint256},
    cosmwasm_schema::{cw_serde, QueryResponses},
    snip20::Snip20ReceiveMsg,
    swap::core::{TokenAmount, TokenType},
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, Query},
    Contract,
};
use lb_libraries::types::{Bytes32, ContractImplementation, StaticFeeParameters};
use std::fmt::{Debug, Display};

#[cw_serde]
pub struct LbPair {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub contract: ContractInfo,
}

#[cw_serde]
pub struct LbPairInformation {
    pub bin_step: u16,
    pub lb_pair: LbPair,
    pub created_by_owner: bool,
    pub ignored_for_routing: bool,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub factory: ContractInfo,
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub pair_parameters: StaticFeeParameters,
    pub active_id: u32,
    pub lb_token_implementation: ContractImplementation,
    pub staking_contract_implementation: ContractImplementation,
    pub viewing_key: String,
    pub entropy: String,
    pub protocol_fee_recipient: Addr,
    pub admin_auth: RawContract,
    pub query_auth: RawContract,
    pub total_reward_bins: Option<u32>,
    // TODO: why should this be necessary? it's already limited by the max gas per block anyway
    pub max_bins_per_swap: Option<u32>,
    pub rewards_distribution_algorithm: RewardsDistributionAlgorithm,
    pub epoch_staking_index: u64,
    pub epoch_staking_duration: u64,
    pub expiry_staking_duration: Option<u64>,
    pub recover_staking_funds_receiver: Addr,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    SwapTokens {
        offer: TokenAmount,
        expected_return: Option<Uint128>,
        to: Option<String>,
        padding: Option<String>,
    },
    Receive(Snip20ReceiveMsg),
    AddLiquidity {
        liquidity_parameters: LiquidityParameters,
    },

    RemoveLiquidity {
        remove_liquidity_params: RemoveLiquidity,
    },

    CollectProtocolFees {},
    SetStaticFeeParameters {
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        variable_fee_control: u32,
        protocol_share: u16,
        max_volatility_accumulator: u32,
    },
    ForceDecay {},
    CalculateRewardsDistribution {},
    ResetRewardsConfig {
        distribution: Option<RewardsDistributionAlgorithm>,
        base_rewards_bins: Option<u32>,
    },
    SetContractStatus {
        contract_status: ContractStatus,
    },
}

#[cw_serde]
pub enum ContractStatus {
    Active,         // allows all operations
    FreezeAll,      // blocks everything except admin-protected config changes
    LpWithdrawOnly, // blocks everything except LP withdraws and admin-protected config changes
}
impl Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[cw_serde]
pub enum RewardsDistributionAlgorithm {
    TimeBasedRewards,
    VolumeBasedRewards,
}

// impl ExecuteMsg {
//     pub fn to_cosmos_msg(
//         &self,
//         code_hash: String,
//         contract_addr: String,
//         send_amount: Option<Uint128>,
//     ) -> StdResult<CosmosMsg> {
//         let mut msg = to_binary(self)?;
//         space_pad(&mut msg.0, 256);
//         let mut funds = Vec::new();
//         if let Some(amount) = send_amount {
//             funds.push(Coin {
//                 amount,
//                 denom: String::from("uscrt"),
//             });
//         }
//         let execute = WasmMsg::Execute {
//             contract_addr,
//             code_hash,
//             msg,
//             funds,
//         };
//         Ok(execute.into())
//     }
// }

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum InvokeMsg {
    SwapTokens {
        expected_return: Option<Uint128>,
        to: Option<String>,
        padding: Option<String>,
    },
}

impl ExecuteCallback for InvokeMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct MintResponse {
    pub amounts_received: Bytes32,
    pub amounts_left: Bytes32,
    pub liquidity_minted: Vec<Uint256>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StakingResponse)]
    GetStakingContract {},
    #[returns(LbTokenResponse)]
    GetLbToken {},
    #[returns(GetPairInfoResponse)]
    GetPairInfo {},
    #[returns(SwapSimulationResponse)]
    SwapSimulation {
        offer: TokenAmount,
        exclude_fee: Option<bool>,
    },
    #[returns(FactoryResponse)]
    GetFactory {},
    #[returns(TokensResponse)]
    GetTokens {},
    #[returns(TokenXResponse)]
    GetTokenX {},
    #[returns(TokenYResponse)]
    GetTokenY {},
    #[returns(BinStepResponse)]
    GetBinStep {},
    #[returns(ReservesResponse)]
    GetReserves {},
    #[returns(ActiveIdResponse)]
    GetActiveId {},
    #[returns(BinResponse)]
    GetBinReserves { id: u32 },
    #[returns(BinsResponse)]
    GetBinsReserves { ids: Vec<u32> },
    #[returns(AllBinsResponse)]
    GetAllBinsReserves {
        id: Option<u32>,
        page: Option<u32>,
        page_size: Option<u32>,
    },
    #[returns(UpdatedBinsAtHeightResponse)]
    GetUpdatedBinAtHeight { height: u64 },
    #[returns(UpdatedBinsAtMultipleHeightResponse)]
    GetUpdatedBinAtMultipleHeights { heights: Vec<u64> },

    #[returns(UpdatedBinsAfterHeightResponse)]
    GetUpdatedBinAfterHeight {
        height: u64,
        page: Option<u32>,
        page_size: Option<u32>,
    },

    #[returns(BinUpdatingHeightsResponse)]
    GetBinUpdatingHeights {
        page: Option<u32>,
        page_size: Option<u32>,
    },

    #[returns(NextNonEmptyBinResponse)]
    GetNextNonEmptyBin { swap_for_y: bool, id: u32 },
    #[returns(ProtocolFeesResponse)]
    GetProtocolFees {},
    #[returns(StaticFeeParametersResponse)]
    GetStaticFeeParameters {},
    #[returns(VariableFeeParametersResponse)]
    GetVariableFeeParameters {},
    #[returns(OracleParametersResponse)]
    GetOracleParameters {},
    #[returns(OracleSampleAtResponse)]
    GetOracleSampleAt { oracle_id: u16 },
    #[returns(OracleSamplesAtResponse)]
    GetOracleSamplesAt { oracle_ids: Vec<u16> },
    #[returns(OracleSamplesAfterResponse)]
    GetOracleSamplesAfter {
        oracle_id: u16,
        page_size: Option<u16>,
    },
    #[returns(PriceFromIdResponse)]
    GetPriceFromId { id: u32 },
    #[returns(IdFromPriceResponse)]
    GetIdFromPrice { price: Uint256 },
    #[returns(SwapInResponse)]
    GetSwapIn {
        amount_out: Uint128,
        swap_for_y: bool,
    },
    #[returns(SwapOutResponse)]
    GetSwapOut {
        amount_in: Uint128,
        swap_for_y: bool,
    },
    #[returns(TotalSupplyResponse)]
    TotalSupply { id: u32 },
    #[returns(RewardsDistributionResponse)]
    GetRewardsDistribution { epoch_id: Option<u64> },
}
impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct StakingResponse {
    pub contract: ContractInfo,
}

#[cw_serde]
pub struct LbTokenResponse {
    pub contract: ContractInfo,
}
#[cw_serde]
pub struct GetPairInfoResponse {
    pub liquidity_token: ContractInfo,
    pub factory: Option<ContractInfo>,
    pub pair: TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint256,
    pub contract_version: u32,
    pub fee_info: FeeInfo,
    pub stable_info: Option<StablePairInfoResponse>,
}
#[cw_serde]
pub struct SwapSimulationResponse {
    pub total_fee_amount: Uint128,
    pub lp_fee_amount: Uint128,
    pub shade_dao_fee_amount: Uint128,
    pub result: SwapResult,
    pub price: String,
}
// We define a custom struct for each query response
#[cw_serde]
pub struct FactoryResponse {
    pub factory: Addr,
}

#[cw_serde]
pub struct TokensResponse {
    pub token_x: TokenType,
    pub token_y: TokenType,
}

// TODO: is it worth it to have queries for the individual tokens?
#[cw_serde]
pub struct TokenXResponse {
    pub token_x: TokenType,
}

// TODO: is it worth it to have queries for the individual tokens?
#[cw_serde]
pub struct TokenYResponse {
    pub token_y: TokenType,
}

#[cw_serde]
pub struct BinStepResponse {
    pub bin_step: u16,
}

#[cw_serde]
pub struct ReservesResponse {
    pub reserve_x: u128,
    pub reserve_y: u128,
}

#[cw_serde]
pub struct ActiveIdResponse {
    pub active_id: u32,
}

#[cw_serde]
pub struct BinResponse {
    pub bin_id: u32,
    pub bin_reserve_x: u128,
    pub bin_reserve_y: u128,
}

#[cw_serde]
pub struct UpdatedBinsAtHeightResponse(pub Vec<BinResponse>);

#[cw_serde]
pub struct UpdatedBinsAtMultipleHeightResponse(pub Vec<BinResponse>);

// TODO: better naming
#[cw_serde]
pub struct UpdatedBinsAfterHeightResponse {
    pub bins: Vec<BinResponse>,
    pub current_block_height: u64,
}

#[cw_serde]
pub struct BinUpdatingHeightsResponse(pub Vec<u64>);

#[cw_serde]
pub struct BinsResponse(pub Vec<BinResponse>);

#[cw_serde]
pub struct AllBinsResponse {
    pub reserves: Vec<BinResponse>,
    pub last_id: u32,
    pub current_block_height: u64,
}

#[cw_serde]
pub struct NextNonEmptyBinResponse {
    pub next_id: u32,
}

#[cw_serde]
pub struct ProtocolFeesResponse {
    pub protocol_fee_x: u128,
    pub protocol_fee_y: u128,
}

#[cw_serde]
pub struct StaticFeeParametersResponse {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub protocol_share: u16,
    pub max_volatility_accumulator: u32,
}

#[cw_serde]
pub struct VariableFeeParametersResponse {
    pub volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub id_reference: u32,
    pub time_of_last_update: u64,
}

#[cw_serde]
pub struct OracleParametersResponse {
    pub sample_lifetime: u8,
    pub size: u16,
    pub last_updated: u64,
    pub first_timestamp: u64,
}

#[cw_serde]
pub struct OracleSampleResponse {
    pub oracle_id: u16,
    pub cumulative_txns: u16,
    pub cumulative_id: u64,
    pub cumulative_volatility: u64,
    pub cumulative_bin_crossed: u64,
    pub cumulative_volume_x: u128,
    pub cumulative_volume_y: u128,
    pub cumulative_fee_x: u128,
    pub cumulative_fee_y: u128,
    pub lifetime: u8,
    pub created_at: u64,
}

#[cw_serde]
pub struct OracleSampleAtResponse {
    pub sample: OracleSampleResponse,
}

// TODO: shouldn't this be Vec<OracleSample> instead?
#[cw_serde]
pub struct OracleSamplesAtResponse {
    pub samples: Vec<OracleSampleResponse>,
}

#[cw_serde]
pub struct OracleSamplesAfterResponse {
    pub samples: Vec<OracleSampleResponse>,
}

#[cw_serde]
pub struct PriceFromIdResponse {
    pub price: Uint256,
}

#[cw_serde]
pub struct IdFromPriceResponse {
    pub id: u32,
}

// TODO: shouldn't this also have dao fees and lp fees?
#[cw_serde]
pub struct SwapInResponse {
    pub amount_in: Uint128,
    pub amount_out_left: Uint128,
    pub fee: Uint128,
}

#[cw_serde]
pub struct SwapOutResponse {
    pub amount_in_left: Uint128,
    pub amount_out: Uint128,
    pub total_fees: Uint128,
    pub shade_dao_fees: Uint128,
    pub lp_fees: Uint128,
}

// TODO: huh?
#[cw_serde]
#[derive(QueryResponses)]
pub enum LbTokenQueryMsg {
    #[returns(TotalSupplyResponse)]
    TotalSupply { id: u32 },
}

#[cw_serde]
pub struct TotalSupplyResponse {
    pub total_supply: Uint256,
}

#[cw_serde]
pub struct RewardsDistributionResponse {
    pub distribution: RewardsDistribution,
}

#[cw_serde]
pub struct RewardsDistribution {
    pub ids: Vec<u32>,
    pub weightages: Vec<u16>,
    pub denominator: u16,
}

#[cw_serde]
pub struct LiquidityParameters {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub amount_x: Uint128,
    pub amount_y: Uint128,
    pub amount_x_min: Uint128,
    pub amount_y_min: Uint128,
    pub active_id_desired: u32,
    pub id_slippage: u32,
    pub delta_ids: Vec<i64>,
    pub distribution_x: Vec<u64>,
    pub distribution_y: Vec<u64>,
    pub deadline: u64,
}

#[cw_serde]
pub struct RemoveLiquidity {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub amount_x_min: Uint128,
    pub amount_y_min: Uint128,
    pub ids: Vec<u32>,
    pub amounts: Vec<Uint256>,
    pub deadline: u64,
}

#[cw_serde]
pub struct FeeInfo {
    pub shade_dao_address: Addr,
    pub lp_fee: Fee,
    pub shade_dao_fee: Fee,
    pub stable_lp_fee: Fee,
    pub stable_shade_dao_fee: Fee,
}

#[cw_serde]
pub struct StablePairInfoResponse {
    pub stable_params: StableParams,
    pub stable_token0_data: StableTokenData,
    pub stable_token1_data: StableTokenData,
    //p is optional so that the PairInfo query can still return even when the calculation of p fails
    pub p: Option<Decimal256>,
}

#[cw_serde]
pub struct CustomIterationControls {
    pub epsilon: Uint256, // assumed to have same decimals as SignedDecimal
    pub max_iter_newton: u16,
    pub max_iter_bisect: u16,
}

#[cw_serde]
pub struct StableParams {
    pub a: Decimal256,
    pub gamma1: Uint256,
    pub gamma2: Uint256,
    pub oracle: Contract,
    pub min_trade_size_x_for_y: Decimal256,
    pub min_trade_size_y_for_x: Decimal256,
    pub max_price_impact_allowed: Decimal256,
    pub custom_iteration_controls: Option<CustomIterationControls>,
}

#[cw_serde]
pub struct StableTokenData {
    pub oracle_key: String,
    pub decimals: u8,
}

#[cw_serde]
pub struct Fee {
    pub nom: u64,
    pub denom: u64,
}

impl Fee {
    pub fn new(nom: u64, denom: u64) -> Self {
        Self { nom, denom }
    }
}

#[cw_serde]
pub struct CustomFee {
    pub shade_dao_fee: Fee,
    pub lp_fee: Fee,
}

#[cw_serde]
pub struct TokenPair {
    pub token_0: TokenType,
    pub token_1: TokenType,
}

#[cw_serde]
pub struct SwapResult {
    pub return_amount: Uint128,
}
