use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, ContractInfo, MessageInfo, StdResult, Uint128, Uint256};
use libraries::tokens::TokenType;

#[cw_serde]
pub struct InstantiateMsg {
    pub factory: ContractInfo,
    pub admins: Option<Vec<Addr>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    #[serde(rename = "create_lb_pair")]
    CreateLBPair {
        token_x: TokenType,
        token_y: TokenType,
        active_id: u32,
        bin_step: u16,
    },
    Receive {
        from: String,
        msg: Option<Binary>,
        amount: Uint128,
    },
    SwapTokensForExact {
        /// The token type to swap from.
        offer: TokenAmount,
        expected_return: Option<Uint128>,
        path: Vec<Hop>,
        recipient: Option<String>,
    },
    RegisterSNIP20Token {
        token_addr: String,
        token_code_hash: String,
    },
    RecoverFunds {
        token: TokenType,
        amount: Uint128,
        to: String,
        msg: Option<Binary>,
    },
}

//LbPair Contract
#[cw_serde]
pub struct Hop {
    pub addr: String,
    pub code_hash: String,
}

#[cw_serde]
pub struct TokenAmount {
    pub token: TokenType,
    pub amount: Uint128,
}

impl TokenAmount {
    pub fn assert_sent_native_token_balance(&self, info: &MessageInfo) -> StdResult<()> {
        self.token
            .assert_sent_native_token_balance(info, self.amount)
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FactoryResponse)]
    GetFactory {},

    #[returns(IdFromPriceResponse)]
    GetIdFromPrice {
        lb_pair: ContractInfo,
        price: Uint256,
    },

    #[returns(PriceFromIdResponse)]
    GetPriceFromId { lb_pair: ContractInfo, id: u32 },

    #[returns(SwapInResponse)]
    GetSwapIn {
        lb_pair: ContractInfo,
        amount_out: Uint128,
        swap_for_y: bool,
    },

    #[returns(SwapOutResponse)]
    GetSwapOut {
        lb_pair: ContractInfo,
        amount_in: Uint128,
        swap_for_y: bool,
    },
}

// Add additional helper functions if needed for more complex queries

// We define a custom struct for each query response
#[cw_serde]
pub struct FactoryResponse {
    pub factory: Addr,
}

#[cw_serde]
pub struct IdFromPriceResponse {
    pub id: u32,
}

#[cw_serde]
pub struct PriceFromIdResponse {
    pub price: Uint256,
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
pub enum ExecuteMsgResponse {
    SwapResult {
        amount_in: Uint128,
        amount_out: Uint128,
    },
}
