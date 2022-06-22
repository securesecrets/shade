macro_rules! implement_harness {
    ($x:ident, $s:ident) => {
        use cosmwasm_std::{from_binary, Binary, Env, HandleResponse, InitResponse, StdResult};
        use fadroma::ensemble::{ContractHarness, MockDeps};
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
    };
}

pub(crate) use implement_harness;

macro_rules! implement_harness_fadroma {
    ($x:ident, $s:ident) => {
        use cosmwasm_std::{InitResponse};
        use fadroma::ensemble::{ContractHarness, MockDeps};
        use fadroma::scrt::cosmwasm_std::{from_binary, Binary, Env, HandleResponse, StdResult};
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
    };
}

pub(crate) use implement_harness_fadroma;
