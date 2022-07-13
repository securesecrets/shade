use crate::c_std::{Uint128, Addr};
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct RequiredTask {
    pub address: Addr,
    pub percent: Uint128,
}
