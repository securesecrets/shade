use cosmwasm_schema::{cw_serde, QueryResponses};
use shade_protocol::{
    c_std::{Addr, Decimal, Uint128},
    contraact_interfaces::snip20::Snip20ReceiveMsg,
};

use lending_utils::{coin::Coin, interest::Interest, token::Token};

#[cw_serde]
pub struct InstantiateMsg {
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: String,
    /// The CodeId of the lending-market contract
    pub lending_market_id: u64,
    /// The CodeId of the lending-token contract
    pub lending_token_id: u64,
    /// Token which would be distributed as reward token to isotonic token holders.
    /// This is `distributed_token` in the market contract.
    pub reward_token: Token,
    /// Common Token (same for all markets)
    pub common_token: Token,
    /// Price for collateral in exchange for paying debt during liquidation
    pub liquidation_price: Decimal,
    /// Maximum percentage of credit_limit that can be borrowed.
    /// This is used to prevent borrowers from being liquidated (almost) immediately after borrowing,
    /// because they maxed out their credit limit.
    pub borrow_limit_ratio: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateMarket(MarketConfig),
    /// Tries to perform liquidation on passed account using collateral's denom. The native tokens
    /// sent along with this message define the debt market.
    Liquidate {
        account: String,
        collateral_denom: Token,
    },
    /// Ensures a given account has entered a market. Meant to be called by a specific
    /// market contract - so the sender of the msg would be the market. The store is treated as a set.
    EnterMarket {
        account: String,
    },
    /// Exits market if:
    /// * Sender have no debt in the market
    /// * Sender have no CTokens in the market, or collateral provided by owned CTokens
    ///   is not affecting liquidity of sender
    ExitMarket {
        /// Address of the `isotonic-market` sender want to exit from.
        market: String,
    },
    /// Repay a loan by using some indicated collateral.
    /// The collateral is traded on Wynd DEX.
    RepayWithCollateral {
        /// The maximum amount of collateral to use
        max_collateral: Coin,
        /// How much of the loan is trying to be repaid
        amount_to_repay: Coin,
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
    Liquidate {
        account: String,
        collateral_denom: Token,
    },
}

#[cw_serde]
pub struct MarketConfig {
    /// Name used to create the cToken name `Lent ${name}`.
    /// Forwarded to `isotonic-token`.
    pub name: String,
    /// Symbol used to create the cToken `C${symbol}`.
    /// Forwarded to `isotonic-token`.
    pub symbol: String,
    /// Decimals for cToken.
    /// Forwarded to `isotonic-token`.
    pub decimals: u8,
    /// Token for the market token
    pub market_token: Token,
    /// An optional cap on total number of tokens deposited into the market
    pub market_cap: Option<Uint128>,
    /// Interest rate curve
    pub interest_rate: Interest,
    /// Define interest's charged period (in seconds)
    pub interest_charge_period: u64,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    pub collateral_ratio: Decimal,
    /// Address of contract to query for price
    pub price_oracle: String,
    /// Defines the portion of borrower interest that is converted into reserves (0 <= x <= 1)
    pub reserve_factor: Decimal,
}
