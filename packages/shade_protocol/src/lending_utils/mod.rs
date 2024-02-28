pub mod amount;
pub mod coin;
pub mod credit_line;
pub mod interest;
pub mod parse_reply;
pub mod price;
pub mod token;

#[cosmwasm_schema::cw_serde]
pub struct ViewingKey {
    pub key: String,
    pub address: String,
}

#[cosmwasm_schema::cw_serde]
pub enum Authentication {
    ViewingKey(ViewingKey),
    Permit(crate::query_auth::QueryPermit),
}
