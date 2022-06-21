pub mod handle;

use cosmwasm_std::{
    from_binary, to_binary, Binary, Env, HandleResponse, HumanAddr, InitResponse, StdError, StdResult
};
use secret_toolkit::utils::Query;
use shade_protocol::contract_interfaces::{
    bonds, snip20::{self, InitialBalance, InitConfig}, oracles::{band::{self, InitMsg}, oracle}, query_auth,
};
use shade_protocol::utils::asset::Contract;
use fadroma::ensemble::{ContractEnsemble, MockEnv};
use fadroma_platform_scrt::ContractLink;
use contract_harness::harness::{bonds::Bonds, snip20::Snip20, oracle::Oracle, mock_band::MockBand, query_auth::QueryAuth};
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
            config: Some(InitConfig {
                public_total_supply: Some(true),
                enable_deposit: Some(true),
                enable_redeem: Some(true),
                enable_mint: Some(true),
                enable_burn: Some(false),
                enable_transfer: Some(true),
            }),
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "issu".into(), 
            code_hash: issu.code_hash })
    )?;

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
            config: Some(InitConfig {
                public_total_supply: Some(true),
                enable_deposit: Some(true),
                enable_redeem: Some(true),
                enable_mint: Some(true),
                enable_burn: Some(false),
                enable_transfer: Some(true),
            }),
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "coll".into(), 
            code_hash: coll.code_hash })
    )?;

    // Register mockband
    let band = chain.register(Box::new(MockBand));
    let band = chain.instantiate(
        band.id, 
        &band::InitMsg {}, 
        MockEnv::new("admin", ContractLink { 
            address: "band".into(), 
            code_hash: band.code_hash 
        })
    )?;

    // Register oracle
    let oracle = chain.register(Box::new(Oracle));
    let oracle = chain.instantiate(
        oracle.id, 
        &oracle::InitMsg {
            admin: Some(HumanAddr::from("admin")),
            band: Contract { address: band.address.clone(), code_hash: band.code_hash.clone() },
            sscrt: Contract { address: HumanAddr::from(""), code_hash: "".into() },
        }, 
        MockEnv::new("admin", ContractLink { 
            address: "oracle".into(), 
            code_hash: oracle.code_hash 
        })
    )?;

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
    )?;

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
    )?;

    // Register bonds
    let bonds = chain.register(Box::new(Bonds));
    let bonds = chain.instantiate(
        bonds.id, 
        &bonds::InitMsg{
            limit_admin: HumanAddr::from("limit_admin"),
            global_issuance_limit: Uint128::new(100_000_000_000_000_000),
            global_minimum_bonding_period: 1,
            global_maximum_discount: Uint128::new(10_000),
            oracle: Contract { address: oracle.address.clone(), code_hash: oracle.code_hash.clone() },
            treasury: HumanAddr::from("admin"),
            issued_asset: Contract { address: issu.address.clone(), code_hash: issu.code_hash.clone() },
            activated: true,
            bond_issuance_limit: Uint128::new(100_000_000_000_000),
            bonding_period: 1,
            discount: Uint128::new(10_000),
            global_min_accepted_issued_price: Uint128::zero(),
            global_err_issued_price: Uint128::zero(),
            allowance_key_entropy: "".into(),
            airdrop: None,
            shade_admins: Contract { address: shade_admin.address.clone(), code_hash: shade_admin.code_hash.clone() },
            query_auth: Contract { address: query_auth.address.clone(), code_hash: query_auth.code_hash.clone() },
        },
        MockEnv::new("admin", ContractLink { 
            address: "bonds".into(), 
            code_hash: bonds.code_hash })    
    )?;

    Ok((chain, bonds, issu, coll, band, oracle, query_auth, shade_admins))
}