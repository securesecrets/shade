use crate::{
    c_std::{Addr, Binary, Decimal256, Uint128, Uint256},
    cosmwasm_schema::cw_serde,
    liquidity_book::lb_pair::SwapResult,
    snip20::Snip20ReceiveMsg,
    swap::core::{
        ContractInstantiationInfo, CustomFee, Fee, StableTokenData, TokenAmount, TokenPair,
        TokenPairAmount, TokenType,
    },
    utils::{
        asset::RawContract, ExecuteCallback,
        InstantiateCallback, Query,
    },
    Contract, BLOCK_SIZE,
};

use std::fmt::{Debug, Display};

use crate::swap::staking::StakingContractInstantiateInfo;

/// Represents the address of an exchange and the pair that it manages
#[cw_serde]
pub struct AMMPair {
    /// The pair that the contract manages.
    pub pair: TokenPair,
    /// Address of the contract that manages the exchange.
    pub address: Addr,
    //  Code hash of the AMM Pair
    pub code_hash: String,
    /// Used to enable or disable the AMMPair
    pub enabled: bool,
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
pub struct StablePairInfoResponse {
    pub stable_params: StableParams,
    pub stable_token0_data: StableTokenData,
    pub stable_token1_data: StableTokenData,
    //p is optional so that the PairInfo query can still return even when the calculation of p fails
    pub p: Option<Decimal256>,
}

#[cw_serde]
pub struct AMMSettings {
    pub lp_fee: Fee,
    pub shade_dao_fee: Fee,
    pub stable_lp_fee: Fee,
    pub stable_shade_dao_fee: Fee,
    pub shade_dao_address: Contract,
}

pub fn generate_pair_key(pair: &TokenPair) -> Vec<u8> {
    let mut bytes: Vec<&[u8]> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    values.push(pair.0.unique_key());
    values.push(pair.1.unique_key());
    values.push(pair.2.to_string());
    values.sort();
    bytes.push(values[0].as_bytes());
    bytes.push(values[1].as_bytes());
    bytes.push(values[2].as_bytes());
    bytes.concat()
}

#[cw_serde]
pub struct SwapInfo {
    pub total_fee_amount: Uint128,
    pub lp_fee_amount: Uint128,
    pub shade_dao_fee_amount: Uint128,
    pub result: SwapResult,
    pub price: String,
    pub new_input_pool: Uint128,
    pub new_output_pool: Uint128,
    pub index_of_input_token: u8,
    pub index_of_output_token: u8,
}

#[cw_serde]
pub struct VirtualSwapResponse {
    pub output: TokenPairAmount,
    pub swap_info: Option<SwapInfo>,
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
pub struct InitMsg {
    pub pair: TokenPair,
    pub token0_oracle_key: Option<String>,
    pub token1_oracle_key: Option<String>,
    pub lp_token_contract: ContractInstantiationInfo,
    // Leave none if initializing without factory
    pub factory_info: Option<Contract>,
    pub prng_seed: Binary,
    pub entropy: Binary,
    pub admin_auth: Contract,
    pub staking_contract: Option<StakingContractInstantiateInfo>,
    pub custom_fee: Option<CustomFee>,
    pub stable_params: Option<StableParams>,
    pub lp_token_decimals: u8,
    pub lp_token_custom_label: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddLiquidityToAMMContract {
        deposit: TokenPairAmount,
        expected_return: Option<Uint128>,
        staking: Option<bool>,
        execute_sslp_virtual_swap: Option<bool>,
        padding: Option<String>,
    },
    SwapTokens {
        /// The token type to swap from.
        offer: TokenAmount,
        expected_return: Option<Uint128>,
        to: Option<String>,
        padding: Option<String>,
    },
    // SNIP20 receiver interface
    Receive(Snip20ReceiveMsg),
    AddWhiteListAddress {
        address: String,
        padding: Option<String>,
    },
    RemoveWhitelistAddresses {
        addresses: Vec<String>,
        padding: Option<String>,
    },
    SetConfig {
        admin_auth: Option<Contract>,
        padding: Option<String>,
    },
    SetStableParams {
        stable_params: Option<StableParams>,
        padding: Option<String>,
    },
    SetCustomPairFee {
        custom_fee: Option<CustomFee>,
        padding: Option<String>,
    },
    SetViewingKey {
        viewing_key: String,
        padding: Option<String>,
    },
    SetOracleKeyAndDecimals {
        token: TokenType,
        oracle_key: String,
        padding: Option<String>,
    },
    SetStakingContract {
        staking: RawContract,
        padding: Option<String>,
    },
    SetContractStatus {
        contract_status: ContractStatus,
        padding: Option<String>,
    },
}

#[cw_serde]
pub enum ExecuteMsgResponse {
    SwapResult {
        price: String,
        amount_in: Uint128,
        amount_out: Uint128,
        total_fee_amount: Uint128,
        lp_fee_amount: Uint128,
        shade_dao_fee_amount: Uint128,
    },
}

#[cw_serde]
pub enum InvokeMsg {
    SwapTokens {
        expected_return: Option<Uint128>,
        to: Option<String>,
        padding: Option<String>,
    },
    RemoveLiquidity {
        /// If sender is removing LP for someone else, from should contain who that someone is
        from: Option<String>,
        single_sided_withdraw_type: Option<TokenType>, //None means 50/50 balanced withdraw, and a value here tells which token to send the withdraw in
        single_sided_expected_return: Option<Uint128>, //this field will be ignored on balanced withdraws
        padding: Option<String>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetConfig {},
    GetPairInfo {},
    GetWhiteListAddress {},
    GetTradeCount {},
    SwapSimulation {
        offer: TokenAmount,
        exclude_fee: Option<bool>,
    },
    GetShadeDaoInfo {},
    GetEstimatedLiquidity {
        deposit: TokenPairAmount,
        sender: Addr,
        execute_sslp_virtual_swap: Option<bool>,
    },
    GetContractStatus {},
}

#[cw_serde]
pub enum QueryMsgResponse {
    GetPairInfo {
        liquidity_token: Contract,
        factory: Option<Contract>,
        pair: TokenPair,
        amount_0: Uint128,
        amount_1: Uint128,
        total_liquidity: Uint128,
        contract_version: u32,
        fee_info: FeeInfo,
        stable_info: Option<StablePairInfoResponse>,
    },
    GetWhiteListAddress {
        addresses: Vec<Addr>,
    },
    GetTradeCount {
        count: u64,
    },
    GetClaimReward {
        amount: Uint128,
    },
    GetEstimatedPrice {
        estimated_price: String,
    },
    SwapSimulation {
        total_fee_amount: Uint128,
        lp_fee_amount: Uint128,
        shade_dao_fee_amount: Uint128,
        result: SwapResult,
        price: String,
    },
    GetShadeDaoInfo {
        shade_dao_address: String,
        shade_dao_fee: Fee,
        lp_fee: Fee,
        admin_auth: Contract,
    },
    GetEstimatedLiquidity {
        lp_token: Uint128,
        tokens_returned: Option<TokenAmount>,
        total_lp_token: Uint128,
    },
    GetConfig {
        factory_contract: Option<Contract>,
        lp_token: Contract,
        staking_contract: Option<Contract>,
        pair: TokenPair,
        custom_fee: Option<CustomFee>,
    },
    GetContractStatus {
        contract_status: ContractStatus,
    },
}

impl InstantiateCallback for InitMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
