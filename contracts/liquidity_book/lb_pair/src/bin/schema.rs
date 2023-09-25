use cosmwasm_schema::write_api;

use shade_protocol::liquidity_book::lb_pair::{ExecuteMsg, InstantiateMsg, QueryMsg};
fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
