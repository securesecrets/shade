use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub factory: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Swap { swap_for_y: bool, to: Addr },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FactoryResponse)]
    GetFactory {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct FactoryResponse {
    pub factory: Addr,
}
