pub mod handle;
pub mod query;

use shade_protocol::contract_interfaces::query_auth;
use contract_harness::harness::query_auth::QueryAuth;
use fadroma::ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma_platform_scrt::ContractLink;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    from_binary,
    to_binary,
    Binary,
    Env,
    HandleResponse,
    HumanAddr,
    InitResponse,
    StdError,
    StdResult,
};

pub fn init_contract() -> StdResult<(ContractEnsemble, ContractLink<HumanAddr>)> {
    let mut chain = ContractEnsemble::new(20);

    let auth = chain.register(Box::new(QueryAuth));
    let auth = chain.instantiate(
        auth.id,
        &query_auth::InitMsg {
            admin: None,
            prng_seed: Binary::from("random".as_bytes())
        },
        MockEnv::new("admin", ContractLink {
            address: "auth".into(),
            code_hash: auth.code_hash
        })
    )?;

    Ok((chain, auth))
}

