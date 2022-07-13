macro_rules! implement_harness {
    ($x:ident, $s:ident) => {
        use cosmwasm_std::{HumanAddr, from_binary, Binary, Env, HandleResponse, InitResponse, StdResult};
        use fadroma::ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
        use fadroma::core::ContractLink;
        use serde::Serialize;
        impl ContractHarness for $x {
            fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
                $s::contract::init(deps, env, from_binary(&msg)?)
            }

            fn handle(
                &self,
                deps: &mut MockDeps,
                env: Env,
                msg: Binary,
            ) -> StdResult<HandleResponse> {
                $s::contract::handle(deps, env, from_binary(&msg)?)
            }

            fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
                $s::contract::query(deps, from_binary(&msg)?)
            }
        }

        pub fn init<T: Serialize>(chain: &mut ContractEnsemble, msg: &T) -> StdResult<ContractLink<HumanAddr>> {
            let contract = chain.register(Box::new($x));
            let contract = chain.instantiate(
                contract.id,
                msg,
                MockEnv::new("admin", ContractLink {
                    address: stringify!($s).into(),
                    code_hash: contract.code_hash,
                }),
            )?.instance;

            Ok(contract)
        }
    };
}

pub(crate) use implement_harness;
