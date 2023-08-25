use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint256};

use shade_protocol::lb_libraries::tokens::TokenType;

// TODO: I wonder if we should move all the message types to the interface libs?
// that way there is a single "source of truth" as they say

pub use shade_protocol::liquidity_book::lb_pair::{ExecuteMsg, InstantiateMsg};

// #[cw_serde]
// pub struct InstantiateMsg {
//     pub factory: String,
//     pub token_x: ContractInfo,
//     pub token_y: ContractInfo,
//     pub bin_step: u16,
//     pub pair_parameters: StaticFeeParameters,
//     pub active_id: u32,
//     pub lb_token: ContractInfo,
// }

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FactoryResponse)]
    GetFactory {},
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

// We define a custom struct for each query response
#[cw_serde]
pub struct FactoryResponse {
    pub factory: Addr,
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
