use cosmwasm_schema::{cw_serde, QueryResponses};
use shade_protocol::c_std::{Decimal, Timestamp, Uint128, ContractInfo};

use lending_utils::interest::Interest;
use lending_utils::{coin::Coin, token::Token};

#[cw_serde]
pub struct InstantiateMsg {
    /// Name used to create the cToken name `Lent ${name}`
    pub name: String,
    /// Symbol used to create the cToken `C${symbol}`
    pub symbol: String,
    /// Decimals for cToken
    pub decimals: u8,
    /// CodeId used to create cToken
    pub token_id: u64,
    /// Market token
    pub market_token: Token,
    /// An optional cap on total number of tokens deposited into the market
    pub market_cap: Option<Uint128>,
    /// Interest rate curve
    pub interest_rate: Interest,
    /// Token which would be distributed via created lend contracts
    pub distributed_token: Token,
    /// Define interest's charged period (in seconds)
    pub interest_charge_period: u64,
    /// Common Token denom that comes from Credit Agency (same for all markets)
    pub common_token: Token,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    pub collateral_ratio: Decimal,
    /// Address of contract to query for price
    pub price_oracle: String,
    /// Defines the portion of borrower interest that is converted into reserves (0 <= x <= 1)
    pub reserve_factor: Decimal,
    /// Maximum percentage of credit_limit that can be borrowed.
    /// This is used to prevent borrowers from being liquidated (almost) immediately after borrowing,
    /// because they maxed out their credit limit.
    pub borrow_limit_ratio: Decimal,
    /// Address of the governance contract that controls this market
    pub gov_contract: String,
    /// Key used for reading data in queries
    pub viewing_key: String,
    // I have no idea what to do with it
    pub ctoken_code_hash: String,
}

#[cw_serde]
pub enum ExecuteMsg {
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
}
