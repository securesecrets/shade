use shade_protocol::{cosmwasm_schema::cw_serde, swap::core::TokenType};

pub use shade_protocol::liquidity_book::lb_pair::{LbPair, LbPairInformation};

#[cw_serde]
pub struct NextPairKey {
    pub token_a: TokenType,
    pub token_b: TokenType,
    pub bin_step: u16,
    pub code_hash: String,
    pub is_open: bool,
}
