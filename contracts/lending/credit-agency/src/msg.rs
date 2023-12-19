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
pub enum ReceiveMsg {
    Liquidate {
        account: String,
        collateral_denom: Token,
    },
}
