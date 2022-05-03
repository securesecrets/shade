use crate::{
    contract::{handle, init, query}
};
use cosmwasm_std::{
    coins, from_binary,
    Extern, HumanAddr, StdError,
    Binary, StdResult, HandleResponse, Env,
    InitResponse,
};
use fadroma_ensemble::{
    MockDeps, ContractHarness, ContractEnsemble,
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
