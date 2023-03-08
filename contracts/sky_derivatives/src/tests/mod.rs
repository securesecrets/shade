mod integration;
mod query;
mod execute;

use shade_protocol::c_std::{
    to_binary, from_binary,
    Addr, StdError, Uint128, Coin,
    ContractInfo, Decimal,
};
use shade_protocol::contract_interfaces::{
    dex::{
        dex::Dex,
        sienna,
    },
    sky::{
        cycles::{
            ArbPair, Derivative,
            DerivativeType,
        },
        sky_derivatives::{
            Config,
            ExecuteMsg,
            InstantiateMsg,
            TradingFees,
        },
    },
    snip20,
};
use shade_protocol_temp::stkd;
use shade_protocol::utils::{
    asset::Contract,
    ExecuteCallback,
    InstantiateCallback,
    MultiTestable,
    Query,
};
use shade_protocol_temp::utils::{
    InstantiateCallback as OtherInstantiateCallback,
    MultiTestable as OtherMultiTestable,
    ExecuteCallback as OtherExecuteCallback,
    Query as OtherQuery,
};
use shade_protocol::multi_test::App;
use shade_multi_test::multi::{
    admin::init_admin_auth,
    snip20::Snip20,
    sky_derivatives::SkyDerivatives,
};
use shade_multi_test_temp::multi::{
    mock_sienna::MockSienna,
    mock_stkd::MockStkd,
};
use mock_sienna_temp::contract as mock_sienna;

fn init() -> (App, ContractInfo, ContractInfo, ContractInfo, ContractInfo, Config) {
    let mut chain = App::default();

    // Init balances
    let admin = Addr::unchecked("admin");
    chain.init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &admin, vec![Coin {
            amount: Uint128::new(2_000_000_000_000),
            denom: "uscrt".into(),
        }]).unwrap();
    });

    // Base snip20
    let base_snip20 = snip20::InstantiateMsg {
        name: "secret SCRT".into(),
        admin: Some("admin".into()),
        symbol: "SSCRT".into(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            address: "admin".into(),
            amount: Uint128::new(1_000_000_000_000),
        }]),
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
        query_auth: None,
    }.test_init(Snip20::default(), &mut chain, admin.clone(), "token", &[]).unwrap();

    // Stkd
    let deriv = stkd::MockInstantiateMsg {
        name: "derivative".to_string(),
        symbol: "stkd-SCRT".to_string(),
        decimals: 6,
        price: Uint128::new(2),
    }.test_init(MockStkd::default(), &mut chain, admin.clone(), "stkd-SCRT", &[]).unwrap();

    stkd::HandleMsg::Stake {
    }.test_exec(&deriv, &mut chain, Addr::unchecked("admin"), &[Coin {
        denom: "uscrt".into(),
        amount: Uint128::new(1_000_000_000_00),
    }]);

    // Sky Derivatives
    let shd_admin = init_admin_auth(&mut chain, &admin);
    let treasury = Addr::unchecked("treasury");
    let derivative = Derivative {
        contract: deriv.clone().into(),
        original_asset: base_snip20.clone().into(),
        staking_type: DerivativeType::StkdScrt,
    };

    let dex_fee = Decimal::permille(997);
    let stake_fee = Decimal::permille(998);
    let unbond_fee = Decimal::raw(999_500_000_000_000_000);
    let trading_fees = TradingFees { dex_fee, stake_fee, unbond_fee };
    let dex_pairs = fill_dex_pairs(2, base_snip20.clone().into(), deriv.clone().into());
    let config = Config {
        shade_admin_addr: shd_admin.clone().into(),
        treasury: treasury.clone(),
        derivative: derivative.clone(),
        trading_fees: trading_fees.clone(),
        max_arb_amount: Uint128::MAX,
        min_profit_amount: Uint128::one(),
        viewing_key: "key".into(),
    };
    let sky_arb = InstantiateMsg {
        shade_admin_addr: shd_admin.clone().into(),
        treasury,
        derivative,
        trading_fees,
        dex_pairs,
        max_arb_amount: Uint128::MAX,
        min_profit_amount: Uint128::one(),
        viewing_key: "key".into(),
    }.test_init(SkyDerivatives::default(), &mut chain, admin.clone(), "arb", &[]).unwrap();

    (chain, shd_admin, base_snip20, deriv, sky_arb, config)
}

fn init_with_pair() -> (App, ContractInfo, ContractInfo, ContractInfo, ContractInfo) {
    let (mut chain, admin, base, deriv, arb, config) = init();

    let pair = seeded_pair(
        &mut chain, 
        base.clone(), 
        deriv.clone(), 
        Uint128::new(2_000_000), 
        Uint128::new(1_000_000)
    );

    ExecuteMsg::SetDexPairs {
        pairs: vec![ArbPair {
            pair_contract: Some(pair.clone().into()),
            mint_info: None,
            token0: base.clone().into(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: deriv.clone().into(),
            token1_decimals: Uint128::new(6),
            token1_amount: None,
            dex: Dex::SiennaSwap,
        }]
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    (chain, base, deriv, arb, pair)
}

fn fill_dex_pairs(num: usize, token0: Contract, token1: Contract) -> Vec<ArbPair> {
    let mut dex_pairs = vec![];
    for i in 0..num {
        dex_pairs.push(ArbPair {
            pair_contract: Some(Contract {
                address: Addr::unchecked(format!("dex pair {}", i)),
                code_hash: "hash".to_string(),
            }),
            mint_info: None,
            token0: token0.clone(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: token1.clone(),
            token1_decimals: Uint128::new(6),
            token1_amount: None,
            dex: Dex::ShadeSwap,
        });
    }

    dex_pairs
}

fn seeded_pair(
    chain: &mut App, 
    token0: ContractInfo, 
    token1: ContractInfo, 
    amt_0: Uint128, 
    amt_1: Uint128
) -> ContractInfo {
    let pair = mock_sienna::InstantiateMsg {
        token_0: token0.clone().into(),
        token_1: token1.clone().into(),
        viewing_key: "key".into(),
    }.test_init(
        MockSienna::default(),
        chain,
        Addr::unchecked("admin"),
        "pair",
        &[],
    ).unwrap();

    snip20::ExecuteMsg::Send {
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: amt_0,
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&token0, chain, Addr::unchecked("admin"), &[]).unwrap();
    
    snip20::ExecuteMsg::Send {
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: amt_1,
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&token1, chain, Addr::unchecked("admin"), &[]).unwrap();
 
    pair
}

#[test]
fn instantiate() {
    let mut chain = App::default();

    let admin = Addr::unchecked("admin");

    let base_snip20 = snip20::InstantiateMsg {
        name: "secret SCRT".into(),
        admin: Some("admin".into()),
        symbol: "SSCRT".into(),
        decimals: 6,
        initial_balances: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
        query_auth: None,
    }.test_init(Snip20::default(), &mut chain, admin.clone(), "token", &[]).unwrap();

    let deriv = stkd::MockInstantiateMsg {
        name: "derivative".to_string(),
        symbol: "stkd-SCRT".to_string(),
        decimals: 6,
        price: Uint128::new(2),
    }.test_init(MockStkd::default(), &mut chain, admin.clone(), "stkd-SCRT", &[]).unwrap();

    let shd_admin = init_admin_auth(&mut chain, &admin);
    let treasury = Addr::unchecked("treasury");
    let derivative = Derivative {
        contract: deriv.clone().into(),
        original_asset: base_snip20.clone().into(),
        staking_type: DerivativeType::StkdScrt,
    };

    let dex_fee = Decimal::raw(999_500_000_000_000_000);
    let stake_fee = Decimal::raw(998);
    let unbond_fee = Decimal::raw(997);
    let trading_fees = TradingFees { dex_fee, stake_fee, unbond_fee };

    let dex_pairs: Vec<ArbPair> = vec![];

    let sky_arb = InstantiateMsg {
        shade_admin_addr: shd_admin.clone().into(),
        treasury,
        derivative: derivative.clone(),
        trading_fees: trading_fees.clone(),
        dex_pairs: dex_pairs.clone(),
        max_arb_amount: Uint128::MAX,
        min_profit_amount: Uint128::one(),
        viewing_key: "key".into(),
    }.test_init(SkyDerivatives::default(), &mut chain, admin.clone(), "arb", &[]).unwrap();

    // Test invalid instantiations
    // dex pairs
    assert!(
        InstantiateMsg { 
            shade_admin_addr: shd_admin.clone().into(),
            treasury: Addr::unchecked("treasury"),
            derivative: derivative.clone(),
            trading_fees: trading_fees.clone(),
            // invalid
            dex_pairs: fill_dex_pairs(2, deriv.clone().into(), base_snip20.clone().into()),
            max_arb_amount: Uint128::MAX,
            min_profit_amount: Uint128::one(),
            viewing_key: "key".into(),
        }.test_init(SkyDerivatives::default(), &mut chain, admin.clone(), "arb2", &[]).is_err(),
    );

    // admin
    assert!(
        InstantiateMsg {
            // invalid
            shade_admin_addr: Contract {
                address: Addr::unchecked("fake admin"), 
                code_hash: "hash".to_string(),
            },
            treasury: Addr::unchecked("treasury"),
            derivative: derivative.clone(),
            trading_fees,
            dex_pairs: dex_pairs.clone(),
            max_arb_amount: Uint128::MAX,
            min_profit_amount: Uint128::one(),
            viewing_key: "key".into(),
        }.test_init(SkyDerivatives::default(), &mut chain, admin.clone(), "arb3", &[]).is_err(),
    );
    
    // trading fees
    assert!(
        InstantiateMsg {
            shade_admin_addr: shd_admin.clone().into(),
            treasury: Addr::unchecked("treasury"),
            derivative,
            // invalid
            trading_fees: TradingFees {
                dex_fee: Decimal::raw(1_100_000_000_000_000_000),
                stake_fee: Decimal::one(),
                unbond_fee: Decimal::one(),
            },
            dex_pairs,
            max_arb_amount: Uint128::MAX,
            min_profit_amount: Uint128::one(),
            viewing_key: "key".into(),
        }.test_init(SkyDerivatives::default(), &mut chain, admin.clone(), "arb4", &[]).is_err(),
    );
}
