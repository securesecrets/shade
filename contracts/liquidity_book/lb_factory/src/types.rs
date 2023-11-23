use shade_protocol::{cosmwasm_schema::cw_serde, swap::core::TokenType};

pub use shade_protocol::lb_libraries::types::{LBPair, LBPairInformation};
use shade_protocol::lb_libraries::{tokens, types};
use tokens::TokenType;
pub use types::{LBPair, LBPairInformation};

#[cw_serde]
pub struct NextPairKey {
    pub token_a: TokenType,
    pub token_b: TokenType,
    pub bin_step: u16,
    pub code_hash: String,
    pub is_open: bool,
}
