use shade_protocol::{
    c_std::{
        coins, from_binary, to_binary,
        Addr, Coin, StdError,
        Binary, StdResult, Env,
        Uint128, QueryRequest, BankQuery,
        BalanceResponse, Decimal
    },
    contract_interfaces::dex::sienna::{
        Pair, PairInfo, TokenType,
    },
    utils::{
        asset::Contract,
        MultiTestable,
        InstantiateCallback,
        ExecuteCallback,
        Query,
    },
    snip20,
};
use shade_protocol::multi_test::App;
use shade_multi_test::multi::{
    mock_stkd::MockStkd,
    mock_sienna::MockSienna,
    snip20::Snip20,
};
use crate::contract as stkd;

use mock_sienna_pair::contract as mock_sienna;


#[test]
fn test() {
    let mut chain = App::default();

    let admin = Addr::unchecked("admin");
    let user = Addr::unchecked("user");
    let other = Addr::unchecked("other-user");

    let init_scrt = Coin {
        denom: "uscrt".to_string(),
        amount: Uint128::new(1000),
    };

    let some_scrt = Coin {
        denom: "uscrt".to_string(),
        amount: Uint128::new(100),
    };

    // Init balances
    chain.init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &user, vec![init_scrt.clone()]).unwrap();
        router.bank.init_balance(storage, &admin, vec![init_scrt.clone()]).unwrap(); 
    });
    
    let stkd = stkd::InstantiateMsg {
        name: "Staking Derivative".to_string(),
        symbol: "stkd-SCRT".to_string(),
        decimals: 6,
        price: Uint128::from(2_000_000u64),
        unbonding_time: 21,
        unbonding_batch_interval: 3,
        staking_commission: Decimal::permille(2),
        unbond_commission: Decimal::from_ratio(5u32, 10_000u32),
    }.test_init(MockStkd::default(), &mut chain, admin.clone(), "stkd", &[]).unwrap();

    // Test Staking
    stkd::ExecuteMsg::Stake {}
        .test_exec(&stkd, &mut chain, user.clone(), &[some_scrt]).unwrap();

    stkd::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }.test_exec(&stkd, &mut chain, user.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::StakingInfo {
            time: 0u64,
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::StakingInfo {
            validators: vec![],
            unbonding_time: 21u32,
            unbonding_batch_interval: 3u32,
            next_unbonding_batch_time: 3u64,
            unbond_amount_of_next_batch: Uint128::zero(),
            batch_unbond_in_progress: false,
            bonded_scrt: Uint128::zero(),
            reserved_scrt: Uint128::zero(),
            available_scrt: Uint128::zero(),
            rewards: Uint128::zero(),
            total_derivative_token_supply: Uint128::zero(),
            price: Uint128::from(2_000_000u64),
        },
    );

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: user.clone(),
            key: "password".to_string(),
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::new(50),
        },
    );

    assert_eq!(   // right amount of scrt left
        chain.wrap().query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: user.to_string(),
            denom: "uscrt".to_string(),
        })).unwrap(),
        BalanceResponse {
            amount: Coin {
                amount: Uint128::new(900),
                denom: "uscrt".to_string(),
            },
        },
    );
 
    // Test Unbonding
    stkd::ExecuteMsg::Unbond {
        redeem_amount: Uint128::new(25),
    }.test_exec(&stkd, &mut chain, user.clone(), &[]).unwrap();

    stkd::ExecuteMsg::MockFastForward {
        steps: 1,
    }.test_exec(&stkd, &mut chain, admin.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::Unbonding {
            address: user.clone(),
            key: "password".to_string(),
            page: None,
            page_size: None,
            time: None,
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Unbonding {
            count: 0,
            claimable_scrt: None,
            unbondings: vec![],
            unbond_amount_in_next_batch: Uint128::new(25),
            estimated_time_of_maturity_for_next_batch: None,
        },
    );

    stkd::ExecuteMsg::MockFastForward {
        steps: 2,
    }.test_exec(&stkd, &mut chain, admin.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::Unbonding {
            address: user.clone(),
            key: "password".to_string(),
            page: None,
            page_size: None,
            time: None,
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Unbonding {
            count: 1,
            claimable_scrt: None,
            unbondings: vec![stkd::Unbond {
                amount: Uint128::new(25),
                unbonds_at: 24u64,
                is_mature: None,
            }],
            unbond_amount_in_next_batch: Uint128::zero(),
            estimated_time_of_maturity_for_next_batch: None,
        },
    );

    stkd::ExecuteMsg::MockFastForward {
        steps: 1
    }.test_exec(&stkd, &mut chain, admin.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::Unbonding {
            address: user.clone(),
            key: "password".to_string(),
            page: None,
            page_size: None,
            time: None,
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Unbonding {
            count: 1,
            claimable_scrt: None,
            unbondings: vec![stkd::Unbond {
                amount: Uint128::new(25),
                unbonds_at: 24u64,
                is_mature: None,
            }],
            unbond_amount_in_next_batch: Uint128::zero(),
            estimated_time_of_maturity_for_next_batch: None,
        },
    );

    stkd::ExecuteMsg::MockFastForward {
        steps: 21
    }.test_exec(&stkd, &mut chain, admin.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::Unbonding {
            address: user.clone(),
            key: "password".to_string(),
            page: None,
            page_size: None,
            time: None,
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Unbonding {
            count: 1,
            claimable_scrt: None,
            unbondings: vec![stkd::Unbond {
                amount: Uint128::new(25),
                unbonds_at: 24u64,
                is_mature: None,
            }],
            unbond_amount_in_next_batch: Uint128::zero(),
            estimated_time_of_maturity_for_next_batch: None,
        },
    );

    // Test Claiming
    stkd::ExecuteMsg::Claim {}
        .test_exec(&stkd, &mut chain, user.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: user.clone(),
            key: "password".to_string(),
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::new(25),
        },
    );

    assert_eq!(   // right amount of scrt returned
        chain.wrap().query::<BalanceResponse>(&QueryRequest::Bank(BankQuery::Balance {
            address: user.to_string(),
            denom: "uscrt".to_string(),
        })).unwrap(),
        BalanceResponse {
            amount: Coin {
                amount: Uint128::new(950),
                denom: "uscrt".to_string(),
            },
        },
    );
    
    // Test wrong viewing key
    assert_eq!(
        stkd::QueryMsg::Balance {
            address: user.clone(),
            key: "not password".to_string(),
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain),
        Err(StdError::generic_err("Querier contract error: Generic error: unauthorized")),
    );

    assert_eq!(
        stkd::QueryMsg::Unbonding {
            address: other.clone(),
            key: "password".to_string(),
            page: None,
            page_size: None,
            time: None,
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain),
        Err(StdError::generic_err("Querier contract error: mock_stkd_derivative::contract::ViewingKey not found")),
    );

    // Test Sending
    stkd::ExecuteMsg::Send {
        recipient: other.clone(),
        recipient_code_hash: None,
        amount: Uint128::new(25),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&stkd, &mut chain, user.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: user.clone(),
            key: "password".to_string(),
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::new(0),
        },
    );

    stkd::ExecuteMsg::SetViewingKey {
        key: "other password".to_string(),
        padding: None,
    }.test_exec(&stkd, &mut chain, other.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: other.clone(),
            key: "other password".to_string(),
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::new(25),
        },
    );

    // Test swap
    let other_snip = snip20::InstantiateMsg {
        name: "other_token".into(),
        admin: None,
        symbol: "OTHER".into(),
        decimals: 6,
        initial_balances: Some(vec![
            snip20::InitialBalance {
                address: user.to_string(),
                amount: Uint128::new(1000),
            },
            snip20::InitialBalance {
                address: admin.to_string(),
                amount: Uint128::new(1000),
            },
        ]),
        prng_seed: Binary::from("random".as_bytes()),
        config: None,
        query_auth: None,
    }
    .test_init(
        Snip20::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "snip20",
        &[],
    ).unwrap();

    let sienna_pair = mock_sienna::InstantiateMsg {
        token_0: stkd.clone().into(),
        token_1: other_snip.clone().into(),
        viewing_key: "key".into(),
        commission: Decimal::permille(3),
    }.test_init(
        MockSienna::default(),
        &mut chain,
        Addr::unchecked("admin"),
        "stkd pair",
        &[],
    ).unwrap();

    stkd::ExecuteMsg::Stake {}  // get stkd to seed pair
        .test_exec(&stkd, &mut chain, admin.clone(), &[init_scrt]).unwrap();

    stkd::ExecuteMsg::Send {  // seed pair
        recipient: sienna_pair.address.clone(),
        recipient_code_hash: None,
        amount: Uint128::new(499),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&stkd, &mut chain, admin.clone(), &[]).unwrap();

    snip20::ExecuteMsg::Send {  // seed pair
        recipient: sienna_pair.address.to_string(),
        recipient_code_hash: None,
        amount: Uint128::new(998),
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&other_snip, &mut chain, admin.clone(), &[]).unwrap();

    mock_sienna::ExecuteMsg::MockPool {
        token_a: Contract {
            address: stkd.address.clone(),
            code_hash: stkd.code_hash.clone(),
        },
        token_b: Contract {
            address: other_snip.address.clone(),
            code_hash: other_snip.code_hash.clone(),
        },
    }.test_exec(&sienna_pair, &mut chain, user.clone(), &[]).unwrap();

    assert_eq!(
        mock_sienna::QueryMsg::PairInfo {}
            .test_query::<mock_sienna::PairInfoResponse>(&sienna_pair, &chain).unwrap(),
        mock_sienna::PairInfoResponse {
            pair_info: PairInfo {
                liquidity_token: Contract {
                    address: Addr::unchecked("lp_token"),
                    code_hash: "hash".to_string(),
                },
                factory: Contract {
                    address: Addr::unchecked("factory"),
                    code_hash: "hash".to_string(),
                },
                pair: Pair {
                    token_0: TokenType::CustomToken {
                        contract_addr: stkd.address.clone(),
                        token_code_hash: stkd.code_hash.clone(),
                    },
                    token_1: TokenType::CustomToken {
                        contract_addr: other_snip.address.clone(),
                        token_code_hash: other_snip.code_hash.clone(),
                    },
                },
                amount_0: Uint128::new(499),
                amount_1: Uint128::new(998),
                total_liquidity: Uint128::new(0),
                contract_version: 0,
            },
        },
    );

    snip20::ExecuteMsg::SetViewingKey {
        key: "password".to_string(),
        padding: None,
    }.test_exec(&other_snip, &mut chain, user.clone(), &[]).unwrap();

    assert_eq!(
        snip20::QueryMsg::Balance {
            address: user.to_string(),
            key: "password".to_string(),
        }.test_query::<snip20::QueryAnswer>(&other_snip, &chain).unwrap(),
        snip20::QueryAnswer::Balance {
            amount: Uint128::new(1000),
        },
    );

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: user.clone(),
            key: "password".to_string(),
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    );

    snip20::ExecuteMsg::Send {
        recipient: sienna_pair.address.to_string(),
        recipient_code_hash: Some(sienna_pair.clone().code_hash),
        amount: Uint128::new(10),
        msg: Some(to_binary(&mock_sienna::ReceiverCallbackMsg::Swap {
            expected_return: None,
            to: None,
        }).unwrap()),
        memo: None,
        padding: None,
    }.test_exec(&other_snip, &mut chain, user.clone(), &[]).unwrap();

    assert_eq!(
        snip20::QueryMsg::Balance {
            address: user.to_string(),
            key: "password".to_string(),
        }.test_query::<snip20::QueryAnswer>(&other_snip, &chain).unwrap(),
        snip20::QueryAnswer::Balance {
            amount: Uint128::new(990),
        },
    );

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: user.clone(),
            key: "password".to_string(),
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::new(5),
        },
    );

    assert_eq!(
        mock_sienna::QueryMsg::PairInfo {}
            .test_query::<mock_sienna::PairInfoResponse>(&sienna_pair, &chain).unwrap(),
        mock_sienna::PairInfoResponse {
            pair_info: PairInfo {
                liquidity_token: Contract {
                    address: Addr::unchecked("lp_token"),
                    code_hash: "hash".to_string(),
                },
                factory: Contract {
                    address: Addr::unchecked("factory"),
                    code_hash: "hash".to_string(),
                },
                pair: Pair {
                    token_0: TokenType::CustomToken {
                        contract_addr: stkd.address.clone(),
                        token_code_hash: stkd.code_hash.clone(),
                    },
                    token_1: TokenType::CustomToken {
                        contract_addr: other_snip.address.clone(),
                        token_code_hash: other_snip.code_hash.clone(),
                    },
                },
                amount_0: Uint128::new(494),
                amount_1: Uint128::new(1_008),
                total_liquidity: Uint128::new(0),
                contract_version: 0,
            },
        },
    );

    // test No balance
    stkd::ExecuteMsg::SetViewingKey {
        key: "key".into(),
        padding: None,
    }.test_exec(&stkd, &mut chain, Addr::unchecked("new"), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::Balance {
            address: Addr::unchecked("new"),
            key: "key".to_string(),
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::Balance {
            amount: Uint128::zero(),
        },
    );

}
