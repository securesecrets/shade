use crate::{
    c_std::{Addr, Decimal, Uint128},
    lend_market,
    lending_utils::{self, coin::Coin, interest::Interest, token::Token, Authentication},
    snip20::Snip20ReceiveMsg,
    utils::asset::Contract,
    utils::{ExecuteCallback, InstantiateCallback, Query},
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /*
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: Contract,
    /// Address of query auth contract
    pub query_auth: Contract,
    /// The CodeId of the lending-market contract
    pub lend_market_id: u64,
    /// The code hash of the lend-market contract
    pub lend_market_code_hash: String,
    /// Market's viewing key used to query market state
    pub market_viewing_key: String,
    /// The CodeId of the lending-token contract
    pub ctoken_token_id: u64,
    /// The code hash of the lending-token contract
    pub ctoken_code_hash: String,
    /// Token which would be distributed as reward token to lend token holders.
    /// This is `distributed_token` in the market contract.
    pub reward_token: Token,
    /// Common Token (same for all markets)
    pub common_token: Token,
    /// Price for collateral in exchange for paying debt during liquidation
    pub liquidation_price: Decimal,
    /// LTV threshold that acts as a “cushion zone” so users can take a max LTV loan but still
    /// have e.g. 5% buffer before getting liquidated
    pub liquidation_threshold: Decimal,
    /// Maximum percentage of credit_limit that can be borrowed.
    /// This is used to prevent borrowers from being liquidated (almost) immediately after borrowing,
    /// because they maxed out their credit limit.
    pub borrow_limit_ratio: Decimal,
    /// How much more of collateral will be used in swap then the estimated amount during
    /// swap_withdraw_from
    pub default_estimate_multiplier: Decimal,
    */
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateMarket(lend_market::InstantiateMsg),
    /// Ensures a given account has entered a market. Meant to be called by a specific
    /// market contract - so the sender of the msg would be the market. The store is treated as a set.
    EnterMarket {
        account: String,
        market: Contract,
    },
    /// Exits market if:
    /// * Sender have no debt in the market
    /// * Sender have no CTokens in the market, or collateral provided by owned CTokens
    ///   is not affecting liquidity of sender
    ExitMarket {
        /// Address of the `lend-market` sender want to exit from.
        market: Contract,
    },
    /// Handles contract's logics that involves receiving Snip20 tokens.
    Receive(Snip20ReceiveMsg),
    /// Sender must be the Governance Contract
    AdjustMarketId {
        new_market_id: u64,
    },
    /// Sender must be the Governance Contract
    AdjustTokenId {
        new_token_id: u64,
    },
    /// Sets common_token parameter in configuration and sends AdjustCommonToken
    /// message to all affiliated markets
    ///
    /// Sender must be the Governance Contract
    AdjustCommonToken {
        new_common_token: Token,
    },
}

#[cw_serde]
pub enum ReceiveMsg {
    /// Tries to perform liquidation on passed account using collateral's denom. The snip20 tokens
    /// sent along with this message define the debt market.
    Liquidate {
        account: String,
        collateral_denom: Token,
    },
}

#[cw_serde]
pub struct MarketConfig {}

#[cw_serde]
// #[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns current configuration
    Configuration {},
    /// Queries a market address by market token
    Market { market_token: Token },
    /// List all base assets and the addresses of markets handling them.
    /// Pagination by base asset
    ListMarkets { limit: Option<u32> },
    /// Queries all markets for credit lines for particular account
    /// and returns sum of all of them.
    TotalCreditLine {
        account: String,
        authentication: Authentication,
    },
    /// Lists all markets which address entered. Pagination by market contract address. Mostly for
    /// verification purposes, but may be useful to verify if there are some obsolete markets to
    /// leave.
    ListEnteredMarkets { account: String },
    /// Checks if account is a member of particular market. Useful to ensure if the account is
    /// included in market before leaving it (to not waste tokens on obsolete call).
    IsOnMarket { account: String, market: Contract },
    /// Checks if the given account is liquidatable and returns the necessary information to do so.
    Liquidation { account: String },

    /// Querie that encapsulates all data for a given user
    UserData {
        account: String,
        authentication: Authentication,
        // Returns balances of entered markets
        tokens_balance: bool,
        withdrawable: bool,
        borrowable: bool,
        credit_line: bool,
    },
}

#[cw_serde]
pub struct UserDataResponse {
    pub token_balance: Vec<(Contract, lend_market::TokensBalanceResponse)>,
    pub withdrawable: Vec<(Contract, Coin)>,
    pub borrowable: Vec<(Contract, Coin)>,
    pub credit_line: Vec<(Contract, lending_utils::credit_line::CreditLineResponse)>,
}

#[cw_serde]
pub struct MarketResponse {
    pub market_token: Token,
    pub market: Contract,
}

#[cw_serde]
pub struct ListMarketsResponse {
    pub markets: Vec<MarketResponse>,
}

#[cw_serde]
pub struct ListEnteredMarketsResponse {
    pub markets: Vec<Contract>,
}

#[cw_serde]
pub struct IsOnMarketResponse {
    pub participating: bool,
}

#[cw_serde]
pub struct LiquidationResponse {
    pub can_liquidate: bool,
    pub debt: Vec<(Contract, Coin)>,
    pub collateral: Vec<(Contract, Coin)>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}
impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}
impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}
