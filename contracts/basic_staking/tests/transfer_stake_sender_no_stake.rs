use shade_protocol::c_std::{to_binary, Addr, BlockInfo, Timestamp, Uint128};

use shade_protocol::{
    contract_interfaces::{basic_staking, query_auth, snip20},
    multi_test::App,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

use shade_multi_test::multi::{
    admin::{init_admin_auth, Admin},
    basic_staking::BasicStaking,
    query_auth::QueryAuth,
    snip20::Snip20,
};

// Add other adapters here as they come
fn transfer_stake(stake_amount: Uint128, transfer_amount: Uint128) {
    let mut app = App::default();

    // init block time for predictable behavior
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(0),
        chain_id: "chain_id".to_string(),
    });

    let viewing_key = "unguessable".to_string();
    let admin_user = Addr::unchecked("admin");
    let receiving_user = Addr::unchecked("staker");
    let sending_user = Addr::unchecked("transfer");

    let token = snip20::InstantiateMsg {
        name: "stake_token".into(),
        admin: Some(admin_user.to_string().clone()),
        symbol: "STKN".into(),
        decimals: 6,
        initial_balances: Some(vec![
            snip20::InitialBalance {
                amount: stake_amount,
                address: receiving_user.to_string(),
            },
            snip20::InitialBalance {
                amount: transfer_amount,
                address: sending_user.to_string(),
            },
        ]),
        query_auth: None,
        prng_seed: to_binary("").ok().unwrap(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(false),
            enable_burn: Some(false),
            enable_transfer: Some(true),
        }),
    }
    .test_init(
        Snip20::default(),
        &mut app,
        admin_user.clone(),
        "stake_token",
        &[],
    )
    .unwrap();

    // set receiving_user viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, receiving_user.clone(), &[])
    .unwrap();

    // set sending_user viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, sending_user.clone(), &[])
    .unwrap();

    let admin_contract = init_admin_auth(&mut app, &admin_user);

    let query_contract = query_auth::InstantiateMsg {
        admin_auth: admin_contract.clone().into(),
        prng_seed: to_binary("").ok().unwrap(),
    }
    .test_init(
        QueryAuth::default(),
        &mut app,
        admin_user.clone(),
        "query_auth",
        &[],
    )
    .unwrap();

    // set staking user VK
    query_auth::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&query_contract, &mut app, receiving_user.clone(), &[])
    .unwrap();

    // set transfer user VK
    query_auth::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&query_contract, &mut app, sending_user.clone(), &[])
    .unwrap();

    let basic_staking = basic_staking::InstantiateMsg {
        admin_auth: admin_contract.into(),
        query_auth: query_contract.into(),
        stake_token: token.clone().into(),
        airdrop: None,
        unbond_period: Uint128::zero(),
        max_user_pools: Uint128::one(),
        viewing_key: viewing_key.clone(),
    }
    .test_init(
        BasicStaking::default(),
        &mut app,
        admin_user.clone(),
        "basic_staking",
        &[],
    )
    .unwrap();
    println!("BASIC STAKING {}", basic_staking.address);

    // Stake staking user
    snip20::ExecuteMsg::Send {
        recipient: basic_staking.address.to_string().clone(),
        recipient_code_hash: None,
        amount: stake_amount,
        msg: Some(
            to_binary(&basic_staking::Action::Stake {
                compound: None,
                airdrop_task: None,
            })
            .unwrap(),
        ),
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, receiving_user.clone(), &[])
    .unwrap();

    // Transfer should fail, not on whitelist
    match (basic_staking::ExecuteMsg::TransferStake {
        amount: transfer_amount,
        recipient: receiving_user.clone().into(),
        compound: Some(true),
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, sending_user.clone(), &[]))
    {
        Err(_) => {}
        Ok(_) => {
            panic!("Shouldn't allow transfer if source not on whitelist");
        }
    }

    // Add transfer user to whitelist
    basic_staking::ExecuteMsg::AddTransferWhitelist {
        user: sending_user.clone().into(),
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, admin_user.clone(), &[])
    .unwrap();

    match (basic_staking::ExecuteMsg::TransferStake {
        amount: transfer_amount,
        recipient: receiving_user.clone().into(),
        compound: Some(true),
        padding: None,
    }
    .test_exec(&basic_staking, &mut app, sending_user.clone(), &[]))
    {
        Ok(_) => {
            panic!("Transfer success with no balance!");
        }
        Err(e) => {}
    }
}

macro_rules! transfer_stake {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    stake_amount,
                    transfer_amount,
                ) = $value;
                transfer_stake(
                    stake_amount,
                    transfer_amount,
                )
            }
        )*
    }
}

transfer_stake! {
    transfer_stake_0: (
        Uint128::new(1), //   stake_amount
        Uint128::new(1), //   transfer_amount
    ),
    transfer_stake_1: (
        Uint128::new(100),
        Uint128::new(100),
    ),
    transfer_stake_2: (
        Uint128::new(1000),
        Uint128::new(1000),
    ),
    transfer_stake_3: (
        Uint128::new(10),
        Uint128::new(10),
    ),
    transfer_stake_4: (
        Uint128::new(1234567),
        Uint128::new(1234567),
    ),
    transfer_stake_5: (
        Uint128::new(99999999999),
        Uint128::new(99999999999),
    ),
}
