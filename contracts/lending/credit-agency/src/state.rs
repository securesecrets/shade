use cosmwasm_schema::{cw_serde, QueryResponses};
use shade_protocol::{
    c_std::{Addr, Decimal},
    secret_storage_plus::{Item, Map},
    utils::asset::Contract,
};

use lending_utils::{token::Token, ViewingKey};

use std::collections::BTreeSet;

#[cw_serde]
pub struct Config {
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: Contract,
    /// The CodeId of the lend-market contract
    pub lend_market_id: u64,
    /// The code hash of the lend-market contract
    pub lend_market_code_hash: String,
    /// The CodeId of the lend-token contract
    pub ctoken_token_id: u64,
    /// The code hash of the lend-token contract
    pub ctoken_code_hash: String,
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
    /// Address of auth query contract
    pub query_auth: Contract,
    /// Market's viewing key used to query market state
    pub market_viewing_key: String,
}

#[cw_serde]
/// Possible states of a market
pub enum MarketState {
    /// Represents a maket that is being created.
    Instantiating,
    /// Represents a market that has already been created.
    Ready(Contract),
}

impl MarketState {
    pub fn to_contract(self) -> Option<Contract> {
        match self {
            MarketState::Instantiating => None,
            MarketState::Ready(contract) => Some(contract),
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
pub const MARKETS: Item<Vec<(Token, MarketState)>> = Item::new("market");
/// A set of "entered markets" for each account, as in markets in which the account is
/// actively participating.
pub const ENTERED_MARKETS: Item<Vec<(Addr, BTreeSet<Addr>)>> = Item::new("entered_martkets");

/// Key generated during CA instantiation and send in configuration with each subsequent market.
/// Necesary for contract to access storage data about users without leaking
pub const MARKET_VIEWING_KEY: Item<ViewingKey> = Item::new("market_viewing_key");

pub fn insert_or_update<K, V>(vec: &mut Vec<(K, V)>, key: K, value: V)
where
    K: PartialEq,
{
    match vec.iter_mut().find(|(k, _)| *k == key) {
        Some((_, v)) => *v = value,
        None => vec.push((key, value)),
    }
}

pub fn find_value<'a, K, V>(vec: &'a Vec<(K, V)>, key: &K) -> Option<&'a V>
where
    K: PartialEq,
{
    vec.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}
