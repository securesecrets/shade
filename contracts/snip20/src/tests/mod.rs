pub mod handle;
pub mod query;

use shade_protocol::{AnyResult, Contract};
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback, Query, MultiTestable};
use shade_protocol::multi_test::{App, AppResponse, Executor};
use shade_multi_test::multi::snip20::Snip20;
use shade_protocol::c_std::{Binary, Addr, StdResult, ContractInfo};
use shade_multi_test::multi::query_auth::QueryAuth;
use shade_multi_test::multi::admin::AdminAuth;
use shade_protocol::shade_admin::MultiTestable as AdminTestable;
use shade_protocol::contract_interfaces::{
    snip20,
    snip20::{InitConfig, InitialBalance},
    query_auth
};

pub fn init_snip20_with_auth(
    initial_balances: Option<Vec<InitialBalance>>,
    config: Option<InitConfig>,
    auth: bool
) -> AnyResult<(App, ContractInfo, Option<ContractInfo>)> {
    let mut chain = App::default();

    let query_auth_addr: Option<Contract>;
    let query_auth_contract: Option<ContractInfo>;

    if auth {
        let stored_code = chain.store_code(AdminAuth::default().contract());
        let admin = chain.instantiate_contract(
            stored_code,
            Addr::unchecked("admin"),
            &shade_admin::admin::InitMsg {},
            &[],
            "admin",
            None
        ).unwrap();

        let auth = query_auth::InstantiateMsg {
            admin_auth: Contract {
                address: admin.address.clone(),
                code_hash: admin.code_hash.clone(),
            },
            prng_seed: Binary::from("random".as_bytes()),
        }
            .test_init(
                QueryAuth::default(),
                &mut chain,
                Addr::unchecked("admin"),
                "query_auth",
                &[],
            )
            .unwrap();

        query_auth_contract = Some(auth.clone());

        query_auth_addr = Some(Contract {
            address: auth.address,
            code_hash: auth.code_hash
        })
    }
    else {
        query_auth_addr = None;
        query_auth_contract = None;
    }


    let snip = snip20::InstantiateMsg {
        name: "Token".into(),
        admin: None,
        symbol: "TKN".into(),
        decimals: 8,
        initial_balances: initial_balances.clone(),
        prng_seed: Binary::from("random".as_bytes()),
        config,
        query_auth: query_auth_addr
    }.test_init(Snip20::default(), &mut chain, Addr::unchecked("admin"), "snip20", &[])?;

    if let Some(balances) = initial_balances {
        for balance in balances.iter() {
            create_vk(&mut chain, &snip, balance.address.as_str(), None)?;
        }
    }

    Ok((chain, snip, query_auth_contract))
}

pub fn init_snip20_with_config(
    initial_balances: Option<Vec<InitialBalance>>,
    config: Option<InitConfig>,
) -> AnyResult<(App, ContractInfo)> {
    let (chain, snip20, _) = init_snip20_with_auth(initial_balances, config, false)?;

    Ok((chain, snip20))
}

pub fn create_vk(
    chain: &mut App,
    snip: &ContractInfo,
    addr: &str,
    key: Option<String>,
) -> AnyResult<AppResponse> {
    snip20::ExecuteMsg::SetViewingKey {
        key: key.unwrap_or("password".into()),
        padding: None,
    }.test_exec(snip, chain, Addr::unchecked(addr), &[])
}
