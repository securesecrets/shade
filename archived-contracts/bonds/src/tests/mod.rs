pub mod handle;
pub mod query;

use contract_harness::harness::{
    admin::Admin, bonds::Bonds, query_auth::QueryAuth, snip20::Snip20,
};
use shade_protocol::c_std::{Addr, StdResult};
use shade_protocol::fadroma::core::ContractLink;
use shade_protocol::fadroma::ensemble::{ContractEnsemble, MockEnv};
use shade_oracles_ensemble::harness::{MockBand, OracleRouter, ProxyBandOracle};
use shade_protocol::contract_interfaces::{
    bonds, query_auth,
    snip20::{self, InitialBalance},
};
use shade_protocol::utils::asset::Contract;

use shade_protocol::c_std::Uint128;
use shade_admin::admin;
use shade_oracles::{
    band::{self, proxy::InstantiateMsg, ExecuteMsg::UpdateSymbolPrice},
    router,
};

pub fn init_contracts() -> StdResult<(
    ContractEnsemble,
    ContractLink<Addr>,
    ContractLink<Addr>,
    ContractLink<Addr>,
    ContractLink<Addr>,
    ContractLink<Addr>,
    ContractLink<Addr>,
    ContractLink<Addr>,
    ContractLink<Addr>,
)> {
    let mut chain = ContractEnsemble::new(50);

    // Register shade_admin
    let shade_admin = chain.register(Box::new(Admin));
    let shade_admin = chain
        .instantiate(
            shade_admin.id,
            &admin::InstantiateMsg {},
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "shade_admin".into(),
                    code_hash: shade_admin.code_hash,
                },
            ),
        )?
        .instance;

    // Register snip20s
    let issu = chain.register(Box::new(Snip20));
    let issu = chain
        .instantiate(
            issu.id,
            &snip20::InstantiateMsg {
                name: "Issued".into(),
                admin: Some(Addr::unchecked("admin")),
                symbol: "ISSU".into(),
                decimals: 8,
                initial_balances: Some(vec![InitialBalance {
                    address: Addr::unchecked("admin"),
                    amount: Uint128::new(1_000_000_000_000_000),
                }]),
                prng_seed: Default::default(),
                config: None,
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "issu".into(),
                    code_hash: issu.code_hash,
                },
            ),
        )?
        .instance;

    let msg = snip20::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };
    chain
        .execute(
            &msg,
            MockEnv::new(
                "secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq",
                issu.clone(),
            ),
        )
        .unwrap();

    let depo = chain.register(Box::new(Snip20));
    let depo = chain
        .instantiate(
            depo.id,
            &snip20::InstantiateMsg {
                name: "Deposit".into(),
                admin: Some(Addr::unchecked("admin")),
                symbol: "DEPO".into(),
                decimals: 8,
                initial_balances: Some(vec![InitialBalance {
                    address: Addr::unchecked("secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq"),
                    amount: Uint128::new(1_000_000_000_000_000),
                }]),
                prng_seed: Default::default(),
                config: None,
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "depo".into(),
                    code_hash: depo.code_hash,
                },
            ),
        )?
        .instance;

    let msg = snip20::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };
    chain
        .execute(&msg, MockEnv::new("admin", depo.clone()))
        .unwrap();

    let atom = chain.register(Box::new(Snip20));
    let atom = chain
        .instantiate(
            atom.id,
            &snip20::InstantiateMsg {
                name: "Atom".into(),
                admin: Some(Addr::unchecked("admin")),
                symbol: "ATOM".into(),
                decimals: 6,
                initial_balances: Some(vec![InitialBalance {
                    address: Addr::unchecked("other_user"),
                    amount: Uint128::new(1_000_000_000_000_000),
                }]),
                prng_seed: Default::default(),
                config: None,
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "atom".into(),
                    code_hash: atom.code_hash,
                },
            ),
        )?
        .instance;

    let msg = snip20::ExecuteMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };
    chain
        .execute(&msg, MockEnv::new("admin", atom.clone()))
        .unwrap();

    // Register mockband
    let band = chain.register(Box::new(MockBand));
    let band = chain
        .instantiate(
            band.id,
            &band::InstantiateMsg {},
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "band".into(),
                    code_hash: band.code_hash,
                },
            ),
        )?
        .instance;

    // Register oracles
    let issu_oracle = chain.register(Box::new(ProxyBandOracle));
    let issu_oracle = chain
        .instantiate(
            issu_oracle.id,
            &InstantiateMsg {
                admin_auth: shade_oracles::common::Contract {
                    address: shade_admin.address.clone(),
                    code_hash: shade_admin.code_hash.clone(),
                },
                band: shade_oracles::common::Contract {
                    address: band.address.clone(),
                    code_hash: band.code_hash.clone(),
                },
                quote_symbol: "ISSU".to_string(),
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "issu_oracle".into(),
                    code_hash: issu_oracle.code_hash,
                },
            ),
        )?
        .instance;

    // Depo oracles
    let depo_oracle = chain.register(Box::new(ProxyBandOracle));
    let depo_oracle = chain
        .instantiate(
            depo_oracle.id,
            &InstantiateMsg {
                admin_auth: shade_oracles::common::Contract {
                    address: shade_admin.address.clone(),
                    code_hash: shade_admin.code_hash.clone(),
                },
                band: shade_oracles::common::Contract {
                    address: band.address.clone(),
                    code_hash: band.code_hash.clone(),
                },
                quote_symbol: "DEPO".to_string(),
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "depo_oracle".into(),
                    code_hash: depo_oracle.code_hash,
                },
            ),
        )?
        .instance;

    // Atom oracle
    let atom_oracle = chain.register(Box::new(ProxyBandOracle));
    let atom_oracle = chain
        .instantiate(
            atom_oracle.id,
            &InstantiateMsg {
                admin_auth: shade_oracles::common::Contract {
                    address: shade_admin.address.clone(),
                    code_hash: shade_admin.code_hash.clone(),
                },
                band: shade_oracles::common::Contract {
                    address: band.address.clone(),
                    code_hash: band.code_hash.clone(),
                },
                quote_symbol: "ATOM".to_string(),
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "atom_oracle".into(),
                    code_hash: atom_oracle.code_hash,
                },
            ),
        )?
        .instance;

    // Oracle Router
    let router = chain.register(Box::new(OracleRouter));
    let router = chain
        .instantiate(
            router.id,
            &router::InstantiateMsg {
                admin_auth: shade_oracles::common::Contract {
                    address: shade_admin.address.clone(),
                    code_hash: shade_admin.code_hash.clone(),
                },
                default_oracle: shade_oracles::common::Contract {
                    address: depo_oracle.address.clone(),
                    code_hash: depo_oracle.code_hash.clone(),
                },
                band: shade_oracles::common::Contract {
                    address: band.address.clone(),
                    code_hash: band.code_hash.clone(),
                },
                quote_symbol: "DEPO".to_string(),
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "router".into(),
                    code_hash: router.code_hash,
                },
            ),
        )?
        .instance;

    let msg = router::ExecuteMsg::UpdateRegistry {
        operation: router::RegistryOperation::Add {
            oracle: shade_oracles::common::Contract {
                address: issu_oracle.address.clone(),
                code_hash: issu_oracle.code_hash.clone(),
            },
            key: "ISSU".to_string(),
        },
    };

    assert!(chain
        .execute(&msg, MockEnv::new("admin", router.clone()))
        .is_ok());

    let msg = router::ExecuteMsg::UpdateRegistry {
        operation: router::RegistryOperation::Add {
            oracle: shade_oracles::common::Contract {
                address: atom_oracle.address.clone(),
                code_hash: atom_oracle.code_hash.clone(),
            },
            key: "ATOM".to_string(),
        },
    };

    assert!(chain
        .execute(&msg, MockEnv::new("admin", router.clone()))
        .is_ok());

    // Register query_auth
    let query_auth = chain.register(Box::new(QueryAuth));
    let query_auth = chain
        .instantiate(
            query_auth.id,
            &query_auth::InstantiateMsg {
                admin_auth: Contract {
                    address: shade_admin.address.clone(),
                    code_hash: shade_admin.code_hash.clone(),
                },
                prng_seed: Default::default(),
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "query_auth".into(),
                    code_hash: query_auth.code_hash,
                },
            ),
        )?
        .instance;

    // Register bonds
    let bonds = chain.register(Box::new(Bonds));
    let bonds = chain
        .instantiate(
            bonds.id,
            &bonds::InstantiateMsg {
                limit_admin: Addr::unchecked("limit_admin"),
                global_issuance_limit: Uint128::new(100_000_000_000_000_000),
                global_minimum_bonding_period: 0,
                global_maximum_discount: Uint128::new(10_000),
                oracle: Contract {
                    address: router.address.clone(),
                    code_hash: router.code_hash.clone(),
                },
                treasury: Addr::unchecked("admin"),
                issued_asset: Contract {
                    address: issu.address.clone(),
                    code_hash: issu.code_hash.clone(),
                },
                activated: true,
                bond_issuance_limit: Uint128::new(100_000_000_000_000),
                bonding_period: 0,
                discount: Uint128::new(10_000),
                global_min_accepted_issued_price: Uint128::new(10_000_000_000_000_000_000),
                global_err_issued_price: Uint128::new(5_000_000_000_000_000_000),
                allowance_key_entropy: "".into(),
                airdrop: None,
                shade_admin: Contract {
                    address: shade_admin.address.clone(),
                    code_hash: shade_admin.code_hash.clone(),
                },
                query_auth: Contract {
                    address: query_auth.address.clone(),
                    code_hash: query_auth.code_hash.clone(),
                },
            },
            MockEnv::new(
                "admin",
                ContractLink {
                    address: "bonds".into(),
                    code_hash: bonds.code_hash,
                },
            ),
        )?
        .instance;

    Ok((
        chain,
        bonds,
        issu,
        depo,
        atom,
        band,
        router,
        query_auth,
        shade_admin,
    ))
}

pub fn set_prices(
    chain: &mut ContractEnsemble,
    band: &ContractLink<Addr>,
    issu_price: Uint128,
    depo_price: Uint128,
    atom_price: Uint128,
) -> StdResult<()> {
    let msg = UpdateSymbolPrice {
        base_symbol: "ISSU".to_string(),
        quote_symbol: "ISSU".to_string(),
        rate: issu_price.u128().into(),
        last_updated: None,
    };
    chain
        .execute(&msg, MockEnv::new("admin", band.clone()))
        .unwrap();

    let msg = UpdateSymbolPrice {
        base_symbol: "DEPO".to_string(),
        rate: depo_price.u128().into(),
        quote_symbol: "DEPO".to_string(),
        last_updated: None,
    };
    chain
        .execute(&msg, MockEnv::new("admin", band.clone()))
        .unwrap();

    let msg = UpdateSymbolPrice {
        base_symbol: "ATOM".to_string(),
        rate: atom_price.u128().into(),
        quote_symbol: "ATOM".to_string(),
        last_updated: None,
    };
    chain
        .execute(&msg, MockEnv::new("admin", band.clone()))
        .unwrap();

    Ok(())
}

pub fn check_balances(
    chain: &mut ContractEnsemble,
    issu: &ContractLink<Addr>,
    depo: &ContractLink<Addr>,
    user_expected_issu: Uint128,
    admin_expected_depo: Uint128,
) -> StdResult<()> {
    let msg = snip20::QueryMsg::Balance {
        address: Addr::unchecked("admin".to_string()),
        key: "key".to_string(),
    };

    let query: snip20::QueryAnswer = chain.query(depo.address.clone(), &msg).unwrap();

    match query {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, admin_expected_depo);
        }
        _ => assert!(false),
    }

    let msg = snip20::QueryMsg::Balance {
        address: Addr::unchecked("secret19rla95xfp22je7hyxv7h0nhm6cwtwahu69zraq".to_string()),
        key: "key".to_string(),
    };

    let query: snip20::QueryAnswer = chain.query(issu.address.clone(), &msg).unwrap();

    match query {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, user_expected_issu);
        }
        _ => assert!(false),
    };

    Ok(())
}

pub fn setup_admin(
    chain: &mut ContractEnsemble,
    shade_admins: &ContractLink<Addr>,
    bonds: &ContractLink<Addr>,
) -> () {
    let msg = admin::ExecuteMsg::AddContract {
        contract_address: bonds.address.clone().to_string(),
    };

    assert!(chain
        .execute(&msg, MockEnv::new("admin", shade_admins.clone()))
        .is_ok());
}

pub fn increase_allowance(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<Addr>,
    issu: &ContractLink<Addr>,
) -> () {
    let msg = snip20::ExecuteMsg::IncreaseAllowance {
        spender: bonds.address.clone(),
        amount: Uint128::new(9_999_999_999_999_999),
        expiration: None,
        padding: None,
    };

    assert!(chain
        .execute(&msg, MockEnv::new("admin", issu.clone()))
        .is_ok());
}
