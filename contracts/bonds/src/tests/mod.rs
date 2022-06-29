pub mod handle;
pub mod query;

use cosmwasm_std::{
    HumanAddr, StdResult
};
use shade_protocol::contract_interfaces::{
    bonds, snip20::{self, InitialBalance}, query_auth,
};
use shade_protocol::utils::asset::Contract;
use fadroma::ensemble::{ContractEnsemble, MockEnv};
use fadroma_platform_scrt::ContractLink;
use contract_harness::harness::{
    bonds::Bonds, 
    snip20::Snip20, 
    query_auth::QueryAuth, 
    admin::ShadeAdmin
};
use shade_oracles_ensemble::harness::{
    ProxyBandOracle, MockBand, OracleRouter
};

use shade_oracles::{band::{proxy::InitMsg, HandleMsg::UpdateSymbolPrice, self}, router};
use cosmwasm_math_compat::Uint128;
use shade_admin::admin;

pub fn init_contracts() -> StdResult<(
    ContractEnsemble, 
    ContractLink<HumanAddr>, 
    ContractLink<HumanAddr>, 
    ContractLink<HumanAddr>, 
    ContractLink<HumanAddr>, 
    ContractLink<HumanAddr>,
    ContractLink<HumanAddr>,
    ContractLink<HumanAddr>,   
    ContractLink<HumanAddr> 
)> {
    let mut chain = ContractEnsemble::new(50);

    // Register snip20s
    let issu = chain.register(Box::new(Snip20));
    let issu = chain.instantiate(
        issu.id, 
        &snip20::InitMsg{
            name: "Issued".into(),
            admin: Some(HumanAddr::from("admin")),
            symbol: "ISSU".into(),
            decimals: 8,
            initial_balances: Some(vec![InitialBalance {
                address: HumanAddr::from("admin"),
                amount: Uint128::new(1_000_000_000_000_000),
            }]),
            prng_seed: Default::default(),
            config: None,
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "issu".into(), 
            code_hash: issu.code_hash })
    )?.instance;

    let msg = snip20::HandleMsg::SetViewingKey { key: "key".to_string(), padding: None };
    chain.execute(&msg, MockEnv::new("user", issu.clone())).unwrap();

    let coll = chain.register(Box::new(Snip20));
    let coll = chain.instantiate(
        coll.id, 
        &snip20::InitMsg{
            name: "Collateral".into(),
            admin: Some(HumanAddr::from("admin")),
            symbol: "COLL".into(),
            decimals: 8,
            initial_balances: Some(vec![InitialBalance {
                address: HumanAddr::from("user"),
                amount: Uint128::new(1_000_000_000_000_000),
            }]),
            prng_seed: Default::default(),
            config: None,
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "coll".into(), 
            code_hash: coll.code_hash })
    )?.instance;

    let msg = snip20::HandleMsg::SetViewingKey { key: "key".to_string(), padding: None };
    chain.execute(&msg, MockEnv::new("admin", coll.clone())).unwrap();

    let atom = chain.register(Box::new(Snip20));
    let atom = chain.instantiate(
        atom.id, 
        &snip20::InitMsg{
            name: "Atom".into(),
            admin: Some(HumanAddr::from("admin")),
            symbol: "ATOM".into(),
            decimals: 6,
            initial_balances: Some(vec![InitialBalance {
                address: HumanAddr::from("other_user"),
                amount: Uint128::new(1_000_000_000_000_000),
            }]),
            prng_seed: Default::default(),
            config: None,
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "atom".into(), 
            code_hash: atom.code_hash })
    )?.instance;

    let msg = snip20::HandleMsg::SetViewingKey { key: "key".to_string(), padding: None };
    chain.execute(&msg, MockEnv::new("admin", atom.clone())).unwrap();

    // Register mockband
    let band = chain.register(Box::new(MockBand));
    let band = chain.instantiate(
        band.id, 
        &band::InitMsg {}, 
        MockEnv::new("admin", ContractLink { 
            address: "band".into(), 
            code_hash: band.code_hash 
        })
    )?.instance;

    // Register oracles
    let issu_oracle = chain.register(Box::new(ProxyBandOracle));
    let issu_oracle = chain.instantiate(
        issu_oracle.id, 
        &InitMsg {
            owner: HumanAddr::from("admin"),
            band: shade_oracles::common::Contract { address: band.address.clone(), code_hash: band.code_hash.clone() },
            quote_symbol: "ISSU".to_string(),
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "issu_oracle".into(), 
            code_hash: issu_oracle.code_hash 
        })
    )?.instance;

    // Coll oracles
    let coll_oracle = chain.register(Box::new(ProxyBandOracle));
    let coll_oracle = chain.instantiate(
        coll_oracle.id, 
        &InitMsg {
            owner: HumanAddr::from("admin"),
            band: shade_oracles::common::Contract { address: band.address.clone(), code_hash: band.code_hash.clone() },
            quote_symbol: "COLL".to_string(),
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "coll_oracle".into(), 
            code_hash: coll_oracle.code_hash 
        })
    )?.instance;

    // Atom oracle
    let atom_oracle = chain.register(Box::new(ProxyBandOracle));
    let atom_oracle = chain.instantiate(
        atom_oracle.id, 
        &InitMsg {
            owner: HumanAddr::from("admin"),
            band: shade_oracles::common::Contract { address: band.address.clone(), code_hash: band.code_hash.clone() },
            quote_symbol: "ATOM".to_string(),
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "atom_oracle".into(), 
            code_hash: atom_oracle.code_hash 
        })
    )?.instance;

    // Oracle Router
    let router = chain.register(Box::new(OracleRouter));
    let router = chain.instantiate(
        router.id, 
        &router::InitMsg {
            owner: HumanAddr::from("admin"),
            default_oracle: shade_oracles::common::Contract {
                address: coll_oracle.address.clone(),
                code_hash: coll_oracle.code_hash.clone()
            }
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "router".into(), 
            code_hash: router.code_hash 
    })    
    )?.instance;

    let msg = router::HandleMsg::UpdateRegistry { operation: router::RegistryOperation::Add { 
        oracle: shade_oracles::common::Contract {
            address: issu_oracle.address.clone(),
            code_hash: issu_oracle.code_hash.clone()
        }, 
        key: "ISSU".to_string() 
    }};

    assert!(chain.execute(&msg, MockEnv::new("admin", router.clone())).is_ok());

    let msg = router::HandleMsg::UpdateRegistry { operation: router::RegistryOperation::Add { 
        oracle: shade_oracles::common::Contract {
            address: atom_oracle.address.clone(),
            code_hash: atom_oracle.code_hash.clone()
        }, 
        key: "ATOM".to_string() 
    }};

    assert!(chain.execute(&msg, MockEnv::new("admin", router.clone())).is_ok());

    // Register query_auth
    let query_auth = chain.register(Box::new(QueryAuth));
    let query_auth = chain.instantiate(
        query_auth.id, 
        &query_auth::InitMsg {
            admin: Some(HumanAddr::from("admin")),
            prng_seed: Default::default()
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "query_auth".into(), 
            code_hash: query_auth.code_hash 
        })
    )?.instance;

    // Register shade_admin
    let shade_admin = chain.register(Box::new(ShadeAdmin));
    let shade_admin = chain.instantiate(
        shade_admin.id, 
        &admin::InitMsg {
            
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "shade_admin".into(), 
            code_hash: shade_admin.code_hash 
        })
    )?.instance;

    // Register bonds
    let bonds = chain.register(Box::new(Bonds));
    let bonds = chain.instantiate(
        bonds.id, 
        &bonds::InitMsg{
            limit_admin: HumanAddr::from("limit_admin"),
            global_issuance_limit: Uint128::new(100_000_000_000_000_000),
            global_minimum_bonding_period: 0,
            global_maximum_discount: Uint128::new(10_000),
            oracle: Contract { address: router.address.clone(), code_hash: router.code_hash.clone() },
            treasury: HumanAddr::from("admin"),
            issued_asset: Contract { address: issu.address.clone(), code_hash: issu.code_hash.clone() },
            activated: true,
            bond_issuance_limit: Uint128::new(100_000_000_000_000),
            bonding_period: 0,
            discount: Uint128::new(10_000),
            global_min_accepted_issued_price: Uint128::new(10_000_000_000_000_000_000),
            global_err_issued_price: Uint128::new(5_000_000_000_000_000_000),
            allowance_key_entropy: "".into(),
            airdrop: None,
            shade_admins: Contract { address: shade_admin.address.clone(), code_hash: shade_admin.code_hash.clone() },
            query_auth: Contract { address: query_auth.address.clone(), code_hash: query_auth.code_hash.clone() },
        },
        MockEnv::new("admin", ContractLink { 
            address: "bonds".into(), 
            code_hash: bonds.code_hash })    
    )?.instance;

    Ok((chain, bonds, issu, coll, atom, band, router, query_auth, shade_admin))
}

pub fn set_prices(
    chain: &mut ContractEnsemble,
    band: &ContractLink<HumanAddr>,
    issu_price: Uint128,
    coll_price: Uint128,
    atom_price: Uint128,
) -> StdResult<()> {
    let msg = UpdateSymbolPrice { 
        base_symbol: "ISSU".to_string(), 
        quote_symbol: "ISSU".to_string(),
        rate: issu_price.into(),
        last_updated: None,
    };
    chain.execute(&msg, MockEnv::new("admin", band.clone())).unwrap();

    let msg = UpdateSymbolPrice { 
        base_symbol: "COLL".to_string(), 
        rate: coll_price.into(),
        quote_symbol: "COLL".to_string(),    
        last_updated: None,
    };
    chain.execute(&msg, MockEnv::new("admin", band.clone())).unwrap();

    let msg = UpdateSymbolPrice { 
        base_symbol: "ATOM".to_string(), 
        rate: atom_price.into(),
        quote_symbol: "ATOM".to_string(),    
        last_updated: None,
    };
    chain.execute(&msg, MockEnv::new("admin", band.clone())).unwrap();
    
    Ok(())
}

pub fn check_balances(
    chain: &mut ContractEnsemble,
    issu: &ContractLink<HumanAddr>,
    coll: &ContractLink<HumanAddr>,
    user_expected_issu: Uint128,
    admin_expected_coll: Uint128,
) -> StdResult<()> {
    let msg = snip20::QueryMsg::Balance { 
        address: HumanAddr::from("admin".to_string()), 
        key: "key".to_string() 
    };

    let query: snip20::QueryAnswer = chain.query(
        coll.address.clone(), 
        &msg,
    ).unwrap();

    match query{
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, admin_expected_coll);
        }
        _ => assert!(false)
    }

    let msg = snip20::QueryMsg::Balance { 
        address: HumanAddr::from("user".to_string()), 
        key: "key".to_string() 
    };

    let query : snip20::QueryAnswer = chain.query(
        issu.address.clone(),
        &msg
    ).unwrap();

    match query{
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, user_expected_issu);
        }
        _ => assert!(false)
    };

    Ok(())
}

pub fn setup_admin (
    chain: &mut ContractEnsemble,
    shade_admins: &ContractLink<HumanAddr>,
    bonds: &ContractLink<HumanAddr>
) -> () {
    let msg = admin::HandleMsg::AddContract { contract_address: bonds.address.clone().to_string() };

    assert!(chain.execute(&msg, MockEnv::new("admin", shade_admins.clone())).is_ok());
}

pub fn increase_allowance (
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<HumanAddr>,
    issu: &ContractLink<HumanAddr>,
) -> () {
    let msg = snip20::HandleMsg::IncreaseAllowance { spender: bonds.address.clone(), amount: Uint128::new(9_999_999_999_999_999), expiration: None, padding: None };

    assert!(chain.execute(&msg, MockEnv::new("admin", issu.clone())).is_ok());
}