use shade_protocol::{
    c_std::{
        coins, from_binary, to_binary,
        Addr, Coin, StdError,
        Binary, StdResult, Env,
        Uint128, QueryRequest, BankQuery,
        BalanceResponse,
    },
    contract_interfaces::stkd,
    utils::{
        asset::Contract,
        MultiTestable,
        InstantiateCallback,
        ExecuteCallback,
        Query,
    },
};
use shade_protocol::multi_test::App;
use shade_multi_test::multi::mock_stkd::MockStkd;



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
        router.bank.init_balance(storage, &user, vec![init_scrt]).unwrap();
    });
    
    let stkd = stkd::MockInstantiateMsg {
        name: "Staking Derivative".to_string(),
        symbol: "stkd-SCRT".to_string(),
        decimals: 6,
        price: Uint128::from(2_000_000u64),
    }.test_init(MockStkd::default(), &mut chain, admin.clone(), "stkd", &[]).unwrap();

    // Test Staking
    stkd::HandleMsg::Stake {}
        .test_exec(&stkd, &mut chain, user.clone(), &[some_scrt]).unwrap();

    stkd::HandleMsg::SetViewingKey {
        key: "password".to_string(),
    }.test_exec(&stkd, &mut chain, user.clone(), &[]).unwrap();

    assert_eq!(
        stkd::QueryMsg::StakingInfo {
            time: 0u64,
        }.test_query::<stkd::QueryAnswer>(&stkd, &chain).unwrap(),
        stkd::QueryAnswer::StakingInfo {
            validators: vec![],
            unbonding_time: 2u32,
            unbonding_batch_interval: 1u32,
            next_unbonding_batch_time: 1u64,
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
    stkd::HandleMsg::Unbond {
        redeem_amount: Uint128::new(25),
    }.test_exec(&stkd, &mut chain, user.clone(), &[]).unwrap();

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

    stkd::HandleMsg::MockFastForward {
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
                unbonds_at: 2u64,
                is_mature: None,
            }],
            unbond_amount_in_next_batch: Uint128::zero(),
            estimated_time_of_maturity_for_next_batch: None,
        },
    );

    stkd::HandleMsg::MockFastForward {
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
                unbonds_at: 2u64,
                is_mature: None,
            }],
            unbond_amount_in_next_batch: Uint128::zero(),
            estimated_time_of_maturity_for_next_batch: None,
        },
    );

    // Test Claiming
    stkd::HandleMsg::Claim {}
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

    let new_scrt = 
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
    stkd::HandleMsg::Send {
        recipient: other.to_string(),
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

    stkd::HandleMsg::SetViewingKey {
        key: "other password".to_string(),
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

}
