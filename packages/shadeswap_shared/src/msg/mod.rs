use crate::core::ContractInstantiationInfo;
use cosmwasm_std::Addr;
use cosmwasm_std::Binary;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    utils::{ExecuteCallback, InstantiateCallback, Query},
    BLOCK_SIZE,
};

pub mod staking;

pub mod router {

    use super::*;
    use crate::core::TokenAmount;
    use shade_protocol::{
        liquidity_book::lb_pair::SwapResult, snip20::Snip20ReceiveMsg,
        utils::liquidity_book::tokens::TokenType, Contract,
    };

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsgResponse {
        SwapResult {
            amount_in: Uint128,
            amount_out: Uint128,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum InvokeMsg {
        SwapTokensForExact {
            path: Vec<Hop>,
            expected_return: Option<Uint128>,
            recipient: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub prng_seed: Binary,
        pub entropy: Binary,
        pub admin_auth: Contract,
        pub airdrop_address: Option<Contract>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct Hop {
        pub addr: String,
        pub code_hash: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        // SNIP20 receiver interface
        Receive(Snip20ReceiveMsg),
        SwapTokensForExact {
            /// The token type to swap from.
            offer: TokenAmount,
            expected_return: Option<Uint128>,
            path: Vec<Hop>,
            recipient: Option<String>,
            padding: Option<String>,
        },
        RegisterSNIP20Token {
            token_addr: String,
            token_code_hash: String,
            oracle_key: Option<String>,
            padding: Option<String>,
        },
        RecoverFunds {
            token: TokenType,
            amount: Uint128,
            to: String,
            msg: Option<Binary>,
            padding: Option<String>,
        },
        SetConfig {
            admin_auth: Option<Contract>,
            padding: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        SwapSimulation {
            offer: TokenAmount,
            path: Vec<Hop>,
            exclude_fee: Option<bool>,
        },
        GetConfig {},
        RegisteredTokens {},
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        SwapSimulation {
            total_fee_amount: Uint128,
            lp_fee_amount: Uint128,
            shade_dao_fee_amount: Uint128,
            result: SwapResult,
            price: String,
        },
        GetConfig {
            admin_auth: Contract,
            airdrop_address: Option<Contract>,
        },
        RegisteredTokens {
            tokens: Vec<Addr>,
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
}

pub mod amm_pair {
    use std::fmt::{Debug, Display};

    use super::*;
    use crate::{
        core::{
            ContractInstantiationInfo, CustomFee, Fee, StableTokenData, TokenAmount, TokenPair,
            TokenPairAmount,
        },
        staking::StakingContractInstantiateInfo,
    };
    use cosmwasm_std::{Addr, Decimal256, Uint256};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use shade_protocol::{
        liquidity_book::lb_pair::SwapResult, snip20::Snip20ReceiveMsg, utils::asset::RawContract,
        utils::liquidity_book::tokens::TokenType, Contract,
    };

    /// Represents the address of an exchange and the pair that it manages
    #[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
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

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
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

    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
    pub struct CustomIterationControls {
        pub epsilon: Uint256, // assumed to have same decimals as SignedDecimal
        pub max_iter_newton: u16,
        pub max_iter_bisect: u16,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct StablePairInfoResponse {
        pub stable_params: StableParams,
        pub stable_token0_data: StableTokenData,
        pub stable_token1_data: StableTokenData,
        //p is optional so that the PairInfo query can still return even when the calculation of p fails
        pub p: Option<Decimal256>,
    }

    #[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug, Clone)]
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

    #[derive(Serialize, Deserialize, PartialEq, Debug, JsonSchema, Clone)]
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

    pub struct VirtualSwapResponse {
        pub output: TokenPairAmount,
        pub swap_info: Option<SwapInfo>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug, JsonSchema)]
    pub struct FeeInfo {
        pub shade_dao_address: Addr,
        pub lp_fee: Fee,
        pub shade_dao_fee: Fee,
        pub stable_lp_fee: Fee,
        pub stable_shade_dao_fee: Fee,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
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

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
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

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
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
    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
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

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
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
}

pub mod factory {
    use super::*;
    use crate::amm_pair::{AMMPair, StableParams};
    use crate::core::TokenPair;
    use crate::staking::StakingContractInstantiateInfo;
    use crate::{amm_pair::AMMSettings, Pagination};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use shade_protocol::Contract;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub pair_contract: ContractInstantiationInfo,
        pub amm_settings: AMMSettings,
        pub lp_token_contract: ContractInstantiationInfo,
        pub prng_seed: Binary,
        pub api_key: String,
        //Set the default authenticator for all permits on the contracts
        pub authenticator: Option<Contract>,
        pub admin_auth: Contract,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        SetConfig {
            pair_contract: Option<ContractInstantiationInfo>,
            lp_token_contract: Option<ContractInstantiationInfo>,
            amm_settings: Option<AMMSettings>,
            api_key: Option<String>,
            admin_auth: Option<Contract>,
            padding: Option<String>,
        },
        CreateAMMPair {
            pair: TokenPair,
            entropy: Binary,
            staking_contract: Option<StakingContractInstantiateInfo>,
            stable_params: Option<StableParams>,
            token0_oracle_key: Option<String>,
            token1_oracle_key: Option<String>,
            lp_token_decimals: u8,
            amm_pair_custom_label: Option<String>,
            lp_token_custom_label: Option<String>,
            padding: Option<String>,
        },
        AddAMMPairs {
            amm_pairs: Vec<AMMPair>,
            padding: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        ListAMMPairs {
            amm_pairs: Vec<AMMPair>,
        },
        GetConfig {
            pair_contract: ContractInstantiationInfo,
            amm_settings: AMMSettings,
            lp_token_contract: ContractInstantiationInfo,
            authenticator: Option<Contract>,
            admin_auth: Contract,
        },
        GetAMMPairAddress {
            address: String,
        },
        AuthorizeApiKey {
            authorized: bool,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        // GetCount returns the current count as a json-encoded number
        ListAMMPairs { pagination: Pagination },
        GetAMMPairAddress { pair: TokenPair },
        GetConfig {},
        AuthorizeApiKey { api_key: String },
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
}

pub mod lp_token {

    use shade_protocol::contract_interfaces::snip20::InitialBalance;

    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    pub struct InitConfig {
        /// Indicates whether the total supply is public or should be kept secret.
        /// default: False
        pub public_total_supply: Option<bool>,
        /// Indicates whether deposit functionality should be enabled
        /// default: False
        pub enable_deposit: Option<bool>,
        /// Indicates whether redeem functionality should be enabled
        /// default: False
        pub enable_redeem: Option<bool>,
        /// Indicates whether mint functionality should be enabled
        /// default: False
        pub enable_mint: Option<bool>,
        /// Indicates whether burn functionality should be enabled
        /// default: False
        pub enable_burn: Option<bool>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    pub struct InstantiateMsg {
        pub name: String,
        pub admin: Option<String>,
        pub symbol: String,
        pub decimals: u8,
        pub initial_balances: Option<Vec<InitialBalance>>,
        pub prng_seed: Binary,
        pub config: Option<InitConfig>,
        pub supported_denoms: Option<Vec<String>>,
    }

    impl InstantiateCallback for InstantiateMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }
}
