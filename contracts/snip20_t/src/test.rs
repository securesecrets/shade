use cosmwasm_std::{Api, Binary, Env, Extern, from_binary, HandleResponse, HandleResult, InitResponse, Querier, StdResult, Storage, to_binary};
use fadroma_ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma_platform_scrt::ContractLink;
use shade_protocol::contract_interfaces::snip20_test;
use shade_protocol::contract_interfaces::snip20_test::{Extended, HandleMsg};
use shade_protocol::utils::wrap::unwrap;

#[test]
fn test() {
}