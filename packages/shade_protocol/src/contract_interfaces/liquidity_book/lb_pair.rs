use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, ContractInfo, Uint128, Uint256};

use crate::utils::{
    liquidity_book::{
        tokens::TokenType,
        types::{Bytes32, ContractInstantiationInfo, StaticFeeParameters},
    },
    ExecuteCallback, InstantiateCallback, Query,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub factory: ContractInfo,
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub pair_parameters: StaticFeeParameters,
    pub active_id: u32,
    pub lb_token_implementation: ContractInstantiationInfo,
    pub viewing_key: String,
    pub pair_name: String,
    pub entropy: String,
    pub protocol_fee_recipient: Addr,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

// TODO: should do something like this to help with code duplication
// pub struct ILBPair;
// impl ILBPair {
//     pub fn get_factory() {
//         todo!()
//     }
// }

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        swap_for_y: bool,
        to: Addr,
        amount_received: Uint128,
    },
    AddLiquidity {
        liquidity_parameters: LiquidityParameters,
    },

    RemoveLiquidity {
        remove_liquidity_params: RemoveLiquidity,
    },

    FlashLoan {},

    // Burn {
    //     from: Addr,
    //     to: Addr,
    //     ids: Vec<u32>,
    //     amounts_to_burn: Vec<Uint256>,
    // },
    CollectProtocolFees {},
    IncreaseOracleLength {
        new_length: u16,
    },
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
}

impl ExecuteCallback for ExecuteMsg {
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
    #[returns(LbTokenResponse)]
    GetLbToken {},
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
    GetBin { id: u32 },
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
    GetOracleSampleAt { look_up_timestamp: u64 },
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
}
impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct LbTokenResponse {
    pub lb_token: ContractInfo,
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

#[cw_serde]
pub struct TokenXResponse {
    pub token_x: TokenType,
}

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
    pub bin_reserve_x: u128,
    pub bin_reserve_y: u128,
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
    pub active_size: u16,
    pub last_updated: u64,
    pub first_timestamp: u64,
}

#[cw_serde]
pub struct OracleSampleAtResponse {
    pub cumulative_id: u64,
    pub cumulative_volatility: u64,
    pub cumulative_bin_crossed: u64,
}

#[cw_serde]
pub struct PriceFromIdResponse {
    pub price: Uint256,
}

#[cw_serde]
pub struct IdFromPriceResponse {
    pub id: u32,
}

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
    pub fee: Uint128,
}

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
pub struct LiquidityParameters {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub amount_x: Uint128,
    pub amount_y: Uint128,
    pub amount_x_min: Uint128,
    pub amount_y_min: Uint128,
    pub active_id_desired: u32,
    pub id_slippage: u32,    //TODO figure this out
    pub delta_ids: Vec<i64>, //TODO this as well
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
