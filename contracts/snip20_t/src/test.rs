use cosmwasm_std::{Api, Binary, Env, Extern, from_binary, HandleResponse, HandleResult, InitResponse, Querier, StdResult, Storage, to_binary};
use fadroma_ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma_platform_scrt::ContractLink;
use shade_protocol::contract_interfaces::snip20_test;
use shade_protocol::contract_interfaces::snip20_test::{Extended, HandleMsg};
use shade_protocol::utils::wrap::unwrap;


struct SNIP20_T;
impl ContractHarness for SNIP20_T {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        crate::contract::init(deps, env, from_binary(&msg)?)
    }

    fn handle(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary,
    ) -> StdResult<HandleResponse> {
        crate::contract::handle(deps, env, from_binary(&msg)?)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        crate::contract::query(deps, from_binary(&msg)?)
    }
}

#[test]
fn test() {
    let mut chain = ContractEnsemble::new(50);

    println!("{}", serde_json::to_string(&Extended::Snip20(HandleMsg::Run {})).unwrap());
    println!("{}", serde_json::to_string(&HandleMsg::Run {}).unwrap());

    // Register governance
    let t = chain.register(Box::new(SNIP20_T));
    let t = chain.instantiate(
        t.id,
        &snip20_test::HandleMsg::Run {},
        MockEnv::new("admin", ContractLink {
            address: "test".into(),
            code_hash: t.code_hash,
        }),
    ).unwrap();

    chain
        .execute(
            &snip20_test::HandleMsg::Run {
            },
            MockEnv::new(
                // Sender is self
                t.address.clone(),
                t.clone(),
            ),
        )
        .unwrap();
}