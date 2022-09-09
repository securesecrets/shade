use shade_protocol::c_std::{
    coins,
    from_binary,
    to_binary,
    Binary,
    Env,
    DepsMut,
    Response,
    Addr,
    Response,
    StdError,
    StdResult,
};

use shade_protocol::c_std::Uint128;
use shade_protocol::{
    contract_interfaces::{
        snip20,
        mint::mint::{ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg},
        oracles::band::{BandQuery, ReferenceData},
    },
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};

use snip20_reference_impl;
use mock_band;
use oracle;

use mint::{
    contract::{handle, instantiate, query},
    handle::{calculate_mint, calculate_portion, try_burn},
};

use contract_harness::harness::{
    mint::Mint, 
    mock_band::MockBand, 
    oracle::Oracle, 
    snip20_reference_impl::Snip20ReferenceImpl as Snip20
};

use shade_protocol::fadroma::{
    ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv},
};
use shade_protocol::fadroma::core::ContractLink;

fn test_ensemble(
    offer_price: Uint128,
    offer_amount: Uint128,
    mint_price: Uint128,
    expected_amount: Uint128,
) {
    let mut ensemble = ContractEnsemble::new(50);

    let reg_oracle = ensemble.register(Box::new(Oracle));
    let reg_mint = ensemble.register(Box::new(Mint));
    let reg_snip20 = ensemble.register(Box::new(Snip20));
    let reg_band = ensemble.register(Box::new(MockBand));

    let sscrt = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20::InstantiateMsg {
                name: "secretSCRT".into(),
                admin: Some("admin".into()),
                symbol: "SSCRT".into(),
                decimals: 6,
                initial_balances: None,
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: Addr::unchecked("sscrt".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap().instance;

    let shade = ensemble
        .instantiate(
            reg_snip20.id,
            &snip20::InstantiateMsg {
                name: "Shade".into(),
                admin: Some("admin".into()),
                symbol: "SHD".into(),
                decimals: 8,
                initial_balances: None,
                prng_seed: to_binary("").ok().unwrap(),
                config: None,
            },
            MockEnv::new("admin", ContractLink {
                address: Addr::unchecked("shade".into()),
                code_hash: reg_snip20.code_hash.clone(),
            }),
        )
        .unwrap().instance;

    let band = ensemble
        .instantiate(
            reg_band.id,
            &shade_protocol::contract_interfaces::oracles::band::InstantiateMsg {},
            MockEnv::new("admin", ContractLink {
                address: Addr::unchecked("band".into()),
                code_hash: reg_band.code_hash.clone(),
            }),
        )
        .unwrap().instance;

    let oracle = ensemble
        .instantiate(
            reg_oracle.id,
            &shade_protocol::contract_interfaces::oracles::oracle::InstantiateMsg {
                admin: Some(Addr::unchecked("admin".into())),
                band: Contract {
                    address: band.address.clone(),
                    code_hash: band.code_hash.clone(),
                },
                sscrt: Contract {
                    address: sscrt.address.clone(),
                    code_hash: sscrt.code_hash.clone(),
                },
            },
            MockEnv::new("admin", ContractLink {
                address: Addr::unchecked("oracle".into()),
                code_hash: reg_oracle.code_hash.clone(),
            }),
        )
        .unwrap().instance;

    let mint = ensemble
        .instantiate(
            reg_mint.id,
            &shade_protocol::contract_interfaces::mint::mint::InstantiateMsg {
                admin: Some(Addr::unchecked("admin".into())),
                oracle: Contract {
                    address: oracle.address.clone(),
                    code_hash: oracle.code_hash.clone(),
                },
                native_asset: Contract {
                    address: shade.address.clone(),
                    code_hash: shade.code_hash.clone(),
                },
                peg: None,
                treasury: Addr::unchecked("admin".into()),
                secondary_burn: None,
                limit: None,
            },
            MockEnv::new("admin", ContractLink {
                address: Addr::unchecked("mint".into()),
                code_hash: reg_mint.code_hash,
            }),
        )
        .unwrap().instance;

    // Setup price feeds
    ensemble
        .execute(
            &mock_band::contract::ExecuteMsg::MockPrice {
                symbol: "SCRT".into(),
                price: offer_price,
            },
            MockEnv::new("admin", band.clone()),
        )
        .unwrap();
    ensemble
        .execute(
            &mock_band::contract::ExecuteMsg::MockPrice {
                symbol: "SHD".into(),
                price: mint_price,
            },
            MockEnv::new("admin", band.clone()),
        )
        .unwrap();

    // Register sSCRT burn
    ensemble
        .execute(
            &shade_protocol::contract_interfaces::mint::mint::ExecuteMsg::RegisterAsset {
                contract: Contract {
                    address: sscrt.address.clone(),
                    code_hash: sscrt.code_hash.clone(),
                },
                capture: None,
                fee: None,
                unlimited: None,
            },
            MockEnv::new("admin", mint.clone()),
        )
        .unwrap();

    // Check mint query
    let (asset, amount) = match ensemble
        .query(
            mint.address.clone(),
            &shade_protocol::contract_interfaces::mint::mint::QueryMsg::Mint {
                offer_asset: sscrt.address.clone(),
                amount: Uint128::new(offer_amount.u128()),
            },
        )
        .unwrap()
    {
        shade_protocol::contract_interfaces::mint::mint::QueryAnswer::Mint { asset, amount } => {
            (asset, amount)
        }
        _ => (
            Contract {
                address: Addr::unchecked("".into()),
                code_hash: "".into(),
            },
            Uint128::new(0),
        ),
    };

    assert_eq!(asset, Contract {
        address: shade.address.clone(),
        code_hash: shade.code_hash.clone(),
    });

    assert_eq!(amount, Uint128::new(expected_amount.u128()));
}

macro_rules! mint_int_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (offer_price, offer_amount, mint_price, expected_amount) = $value;
                test_ensemble(offer_price, offer_amount, mint_price, expected_amount);
            }
        )*
    }
}
mint_int_tests! {
    mint_int_0: (
        Uint128::new(10u128.pow(18)), // $1
        Uint128::new(10u128.pow(6)), // 1sscrt
        Uint128::new(10u128.pow(18)), // $1
        Uint128::new(10u128.pow(8)), // 1 SHD
    ),
    mint_int_1: (
        Uint128::new(2 * 10u128.pow(18)), // $2
        Uint128::new(10u128.pow(6)), // 1 sscrt
        Uint128::new(10u128.pow(18)), // $1
        Uint128::new(2 * 10u128.pow(8)), // 2 SHD
    ),
    mint_int_2: (
        Uint128::new(1 * 10u128.pow(18)), // $1
        Uint128::new(4 * 10u128.pow(6)), // 4 sscrt
        Uint128::new(10u128.pow(18)), // $1
        Uint128::new(4 * 10u128.pow(8)), // 4 SHD
    ),
    mint_int_3: (
        Uint128::new(10 * 10u128.pow(18)), // $10
        Uint128::new(30 * 10u128.pow(6)), // 30 sscrt
        Uint128::new(5 * 10u128.pow(18)), // $5
        Uint128::new(60 * 10u128.pow(8)), // 60 SHD
    ),
}
