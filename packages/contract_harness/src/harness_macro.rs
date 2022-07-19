#[macro_export]
macro_rules! implement_harness {
    ($x:ident, $s:ident) => {
        use shade_protocol::c_std::{from_binary, Binary, Env, Response, StdResult};
        use shade_protocol::fadroma::ensemble::{ContractHarness, MockDeps};
        impl ContractHarness for $x {
            fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<Response> {
                $s::contract::init(deps, env, info, from_binary(&msg)?)
            }

            fn handle(
                &self,
                deps: &mut MockDeps,
                env: Env,
                msg: Binary,
            ) -> StdResult<Response> {
                $s::contract::handle(deps, env, info, from_binary(&msg)?)
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

