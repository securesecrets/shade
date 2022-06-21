pub mod handle;
pub mod query;

use contract_harness::harness::query_auth::QueryAuth;
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
use fadroma::ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma_platform_scrt::ContractLink;
use query_authentication::transaction::{PermitSignature, PubKey};
use shade_protocol::contract_interfaces::{
    query_auth,
    query_auth::{PermitData, QueryPermit},
};

pub fn init_contract() -> StdResult<(ContractEnsemble, ContractLink<HumanAddr>)> {
    let mut chain = ContractEnsemble::new(20);

    let auth = chain.register(Box::new(QueryAuth));
    let auth = chain
        .instantiate(
            auth.id,
            &query_auth::InitMsg {
                admin: None,
                prng_seed: Binary::from("random".as_bytes()),
            },
            MockEnv::new("admin", ContractLink {
                address: "auth".into(),
                code_hash: auth.code_hash,
            }),
        )?
        .instance;

    Ok((chain, auth))
}

pub fn get_permit() -> QueryPermit {
    QueryPermit {
        params: PermitData {
            key: "key".to_string(),
            data: Binary::from_base64("c29tZSBzdHJpbmc=").unwrap()
        },
        signature: PermitSignature {
            pub_key: PubKey::new(
                Binary::from_base64(
                    "A9NjbriiP7OXCpoTov9ox/35+h5k0y1K0qCY/B09YzAP"
                ).unwrap()
            ),
            signature: Binary::from_base64(
                "XRzykrPmMs0ZhksNXX+eU0TM21fYBZXZogr5wYZGGy11t2ntfySuQNQJEw6D4QKvPsiU9gYMsQ259dOzMZNAEg=="
            ).unwrap()
        },
        account_number: None,
        chain_id: Some(String::from("chain")),
        sequence: None,
        memo: None
    }
}
