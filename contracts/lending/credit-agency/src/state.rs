use cosmwasm_schema::{cw_serde, QueryResponses};
use shade_protocol::{
    c_std::{Addr, Decimal},
    secret_storage_plus::{Item, Map},
    utils::asset::Contract,
};

use lending_utils::token::Token;

use std::collections::BTreeSet;

#[cw_serde]
pub struct Config {
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: Contract,
    /// The CodeId of the lend-market contract
    pub lend_market_id: u64,
    /// The CodeId of the lend-token contract
    pub lend_token_id: u64,
    /// Token which would be distributed as reward token to wynd_lend token holders.
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
    /// How much more of collateral will be used in swap then the estimated amount during
    /// swap_withdraw_from
    pub default_estimate_multiplier: Decimal,
}

#[cw_serde]
/// Possible states of a market
pub enum MarketState {
    /// Represents a maket that is being created.
    Instantiating,
    /// Represents a market that has already been created.
    Ready(Addr),
}

impl MarketState {
    pub fn to_addr(self) -> Option<Addr> {
        match self {
            MarketState::Instantiating => None,
            MarketState::Ready(addr) => Some(addr),
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
/// A map of reply_id -> market_token, used to tell which base asset
/// a given instantiating contract will handle
pub const REPLY_IDS: Map<u64, Token> = Map::new("reply_ids");
/// The next unused reply ID
pub const NEXT_REPLY_ID: Item<u64> = Item::new("next_reply_id");
/// A map of market asset -> market contract address
pub const MARKETS: Map<&Token, MarketState> = Map::new("market");
/// A set of "entered markets" for each account, as in markets in which the account is
/// actively participating.
pub const ENTERED_MARKETS: Map<&Addr, BTreeSet<Addr>> = Map::new("entered_martkets");
