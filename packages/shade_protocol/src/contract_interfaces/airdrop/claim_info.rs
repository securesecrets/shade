use crate::c_std::{Uint128, Addr};
use cosmwasm_schema::{cw_serde};

#[cw_serde]
pub struct RequiredTask {
    pub address: Addr,
    pub percent: Uint128,
}
