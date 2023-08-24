use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{
    Addr,
};


#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    // does this need to be ContractInfo?
    pub lb_pair: Addr,
}
