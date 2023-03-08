use shade_protocol::c_std::{
    to_binary, from_binary,
    Addr, StdError, Uint128, Coin,
    Decimal,
};
use shade_protocol::contract_interfaces::{
    dex::dex::Dex,
    sky::{
        cycles::{
            ArbPair, Derivative,
            DerivativeType, 
        },
        sky_derivatives::{
            Config, Direction,
            ExecuteMsg,
            QueryAnswer, QueryMsg,
            SwapAmounts,
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
use shade_multi_test_temp::multi::mock_stkd::MockStkd;
use mock_sienna_temp::contract as mock_sienna;

use crate::tests::{init, init_with_pair, fill_dex_pairs, seeded_pair};

#[test]
fn get_config() {
    let (chain, _, base, deriv, arb, config) = init();

    assert_eq!(
        QueryMsg::Config { }
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::Config {
            config,
        },
    );

    // Make sure viewing keys work
    assert_eq!(
        snip20::QueryMsg::Balance {
            address: arb.address.to_string(),
            key: "key".into(),
        }.test_query::<snip20::QueryAnswer>(&base, &chain).unwrap(),
        snip20::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    )
}

#[test]
fn dex_pairs() {
    let (chain, admin, base, deriv, arb, config) = init();

    assert_eq!(
        QueryMsg::DexPairs {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::DexPairs {
            dex_pairs: vec![
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 0"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
                ArbPair {
                    pair_contract: Some(Contract {
                        address: Addr::unchecked("dex pair 1"),
                        code_hash: "hash".to_string(),
                    }),
                    mint_info: None,
                    token0: base.clone().into(),
                    token0_decimals: Uint128::new(6),
                    token0_amount: None,
                    token1: deriv.clone().into(),
                    token1_decimals: Uint128::new(6),
                    token1_amount: None,
                    dex: Dex::ShadeSwap,
                },
            ],
        },
    );
}
#[test]
fn is_profitable() {
    let (mut chain, base, deriv, arb, pair) = init_with_pair();

    // Unprofitable
    assert_eq!( // Pair: 1_000_000 deriv; 2_000_000 base
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: false,
            swap_amounts: None,
            direction: None,
        },
    );

    // Profitable staking direction
    snip20::ExecuteMsg::Send { // Pair: 1_000_000 deriv; 2_025_000 base
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(25_000),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: true,
            swap_amounts: Some(SwapAmounts {
                optimal_swap: Uint128::new(7444),
                swap1_result: Uint128::new(3714),
                swap2_result: Uint128::new(7472),
            }),
            direction: Some(Direction::Stake),
        },
    );

    // Profitable unbond direction
    snip20::ExecuteMsg::Send { // Pair: 1_000_000 deriv; 1_075_000 base
        recipient: "admin".to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(50_000),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, pair.clone().address, &[]).unwrap();

    assert_eq!(
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: true,
            swap_amounts: Some(SwapAmounts {
                optimal_swap: Uint128::new(8981),
                swap1_result: Uint128::new(4513),
                swap2_result: Uint128::new(9021),
            }),
            direction: Some(Direction::Unbond),
        },
    );

    // Unprofitable because of fees
    snip20::ExecuteMsg::Send { // Pair: 1_000_000 deriv; 2_010_038 base
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(35_038),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: false,
            swap_amounts: None,
            direction: None,
        },
    );

    // Profitable but barely do don't
    snip20::ExecuteMsg::Send { // Pair: 1_000_000 deriv; 2_010_039 base
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(1),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: false,
            swap_amounts: Some(SwapAmounts {
                optimal_swap: Uint128::zero(),
                swap1_result: Uint128::zero(),
                swap2_result: Uint128::zero(),
            }),
            direction: Some(Direction::Stake),
        },
    );

    // Min profit amount - was profitable but not accepted more
    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: None,
        min_profit_amount: Some(Uint128::new(10_000)),
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    snip20::ExecuteMsg::Send { // Pair: 1_000_000 deriv; 2_025_000 base
        recipient: pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(14_961),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&base, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: false,
            swap_amounts: Some(SwapAmounts {
                optimal_swap: Uint128::new(7444),
                swap1_result: Uint128::new(3714),
                swap2_result: Uint128::new(7472),
            }),
            direction: Some(Direction::Stake),
        },
    );

    // Dex pair swap reversed
    let pair = seeded_pair(
        &mut chain, 
        deriv.clone(), // swapped 
        base.clone(), // swapped
        Uint128::new(1_000_000), 
        Uint128::new(2_025_000)
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

    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: None,
        min_profit_amount: Some(Uint128::new(1)),
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: true,
            swap_amounts: Some(SwapAmounts {
                optimal_swap: Uint128::new(7444),
                swap1_result: Uint128::new(3714),
                swap2_result: Uint128::new(7472),
            }),
            direction: Some(Direction::Stake),
        },
    );

    // Low max swap
    println!("LOW MAX SWAP");
    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: Some(Uint128::new(1_000)),
        min_profit_amount: None,
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: true,
            swap_amounts: Some(SwapAmounts {
                optimal_swap: Uint128::new(1000),
                swap1_result: Uint128::new(499),
                swap2_result: Uint128::new(1006),
            }),
            direction: Some(Direction::Stake),
        },
    );

    // Profitable different number of decimals
    let pair = seeded_pair(
        &mut chain, 
        base.clone(),
        deriv.clone(),
        Uint128::new(2_025_000),
        Uint128::new(100_000_000),
    );

    ExecuteMsg::SetDexPairs {
        pairs: vec![ArbPair {
            pair_contract: Some(pair.clone().into()),
            mint_info: None,
            token0: base.clone().into(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: deriv.clone().into(),
            token1_decimals: Uint128::new(8),
            token1_amount: None,
            dex: Dex::SiennaSwap,
        }]
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    ExecuteMsg::UpdateConfig {
        shade_admin_addr: None,
        treasury: None,
        derivative: None,
        trading_fees: None,
        max_arb_amount: Some(Uint128::MAX),
        min_profit_amount: None,
        viewing_key: None,
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::IsProfitable {
            index: None,
        }.test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsProfitable {
            is_profitable: true,
            swap_amounts: Some(SwapAmounts {
                optimal_swap: Uint128::new(7444),
                swap1_result: Uint128::new(371488),
                swap2_result: Uint128::new(7472),
            }),
            direction: Some(Direction::Stake),
        },
    );
}

#[test]
fn is_any_pair_profitable() {
    let (mut chain, base, deriv, arb, pair) = init_with_pair();

    let pair_1 = seeded_pair(
        &mut chain, 
        base.clone(),
        deriv.clone(),
        Uint128::new(2_025_000),
        Uint128::new(1_000_000), 
    );

    let pair_2 = seeded_pair(
        &mut chain, 
        base.clone(),
        deriv.clone(),
        Uint128::new(1_975_000),
        Uint128::new(1_000_000_000), 
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
        },
        ArbPair {
            pair_contract: Some(pair_1.clone().into()),
            mint_info: None,
            token0: base.clone().into(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: deriv.clone().into(),
            token1_decimals: Uint128::new(6),
            token1_amount: None,
            dex: Dex::SiennaSwap,
        },
        ArbPair {
            pair_contract: Some(pair_2.clone().into()),
            mint_info: None,
            token0: base.clone().into(),
            token0_decimals: Uint128::new(6),
            token0_amount: None,
            token1: deriv.clone().into(),
            token1_decimals: Uint128::new(9),
            token1_amount: None,
            dex: Dex::SiennaSwap,
        }]
    }.test_exec(&arb, &mut chain, Addr::unchecked("admin"), &[]).unwrap();

    assert_eq!(
        QueryMsg::IsAnyPairProfitable {}
            .test_query::<QueryAnswer>(&arb, &chain).unwrap(),
        QueryAnswer::IsAnyPairProfitable { 
            is_profitable: vec![false, true, true], 
            swap_amounts: vec![
            None,
            Some(SwapAmounts {
                optimal_swap: Uint128::new(7444),
                swap1_result: Uint128::new(3714),
                swap2_result: Uint128::new(7472),
            }),
            Some(SwapAmounts {
                optimal_swap: Uint128::new(8981),
                swap1_result: Uint128::new(4513216),
                swap2_result: Uint128::new(9021),
            })], 
            direction: vec![None, Some(Direction::Stake), Some(Direction::Unbond)],
        }
    ); 
}

/*
// Adapter Tests
#[test]
fn adapter_balance() {
    assert!(false);
}

#[test]
fn adapter_claimable() {
    assert!(false);
}

#[test]
fn adapter_unbonding() {
    assert!(false);
}

#[test]
fn adapter_unbondable() {
    assert!(false);
}

#[test]
fn adapter_reserves() {
    assert!(false);
}
*/
