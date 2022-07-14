use anyhow::Result as AnyResult;
use cosmwasm_std::{ContractInfo, Coin};
use shade_protocol::{multi_test::{App, AppResponse, Contract, ContractWrapper, Executor}, serde::{Serialize, de::DeserializeOwned}, c_std::{Addr, Empty, StdResult}};

/// Trait for making integration with multi-test easier.
pub trait MultiTestable {
    fn get_info(&self) -> &ContractInfo;
    fn contract() -> Box<dyn Contract<Empty>>;
    fn new(info: ContractInfo) -> Self;
    fn init<T: Serialize>(
        router: &mut App,
        sender: Addr,
        label: &str,
        send_funds: &[Coin],
        msg: &T,
    ) -> ContractInfo;
    fn query<T: DeserializeOwned>(&self, router: &App, msg: &impl Serialize) -> StdResult<T> {
        let info = self.get_info();
        router
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: info.address.clone(),
                msg,
                code_hash: info.code_hash.clone()
            }))
    }
    fn execute<T: Serialize + std::fmt::Debug>(
        &self,
        router: &mut App,
        sender: Addr,
        msg: &T,
        send_funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        router.execute_contract(sender, (*self.get_info()).clone(), msg, send_funds)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod multi;
#[cfg(not(target_arch = "wasm32"))]
pub mod multi_macro;