use crate::contract::{handle, init, query};
use cosmwasm_std::{
    coins,
    from_binary,
    Binary,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    InitResponse,
    StdError,
    StdResult,
};
use fadroma::{
    ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv},
    ContractLink,
};

pub struct Oracle;

impl ContractHarness for Oracle {
    // Use the method from the default implementation
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        init(
            deps,
            env,
            from_binary(&msg)?,
            //mint::DefaultImpl,
        )
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        handle(
            deps,
            env,
            from_binary(&msg)?,
            //mint::DefaultImpl,
        )
    }

    // Override with some hardcoded value for the ease of testing
    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        query(
            deps,
            from_binary(&msg)?,
            //mint::DefaultImpl,
        )
    }
}
