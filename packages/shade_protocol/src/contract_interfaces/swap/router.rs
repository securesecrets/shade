use crate::{
    c_std::{Addr, Binary, Uint128},
    cosmwasm_schema::cw_serde,
    liquidity_book::lb_pair::SwapResult,
    snip20::Snip20ReceiveMsg,
    swap::core::{ContractInstantiationInfo, TokenAmount, TokenType},
    utils::{ExecuteCallback, InstantiateCallback, Query},
    Contract, BLOCK_SIZE,
};

#[cw_serde]
pub enum ExecuteMsgResponse {
    SwapResult {
        amount_in: Uint128,
        amount_out: Uint128,
    },
}

#[cw_serde]
pub enum InvokeMsg {
    SwapTokensForExact {
        path: Vec<Hop>,
        expected_return: Option<Uint128>,
        recipient: Option<String>,
    },
}

#[cw_serde]
pub struct InitMsg {
    pub prng_seed: Binary,
    pub entropy: Binary,
    pub admin_auth: Contract,
    pub airdrop_address: Option<Contract>,
}

#[cw_serde]
pub struct Hop {
    pub addr: String,
    pub code_hash: String,
}

#[cw_serde]
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

#[cw_serde]
pub enum QueryMsg {
    SwapSimulation {
        offer: TokenAmount,
        path: Vec<Hop>,
        exclude_fee: Option<bool>,
    },
    GetConfig {},
    RegisteredTokens {},
}

#[cw_serde]
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
