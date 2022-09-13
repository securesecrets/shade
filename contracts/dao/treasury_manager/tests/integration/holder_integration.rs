use shade_multi_test::multi::admin::init_admin_auth;
use shade_protocol::c_std::{
    to_binary,
    Addr,
    Uint128,
};

//use shade_protocol::secret_toolkit::snip20;

use shade_multi_test::multi::{snip20::Snip20, treasury_manager::TreasuryManager};
use shade_protocol::{
    dao::{manager, treasury_manager},
    multi_test::App,
    snip20,
    utils::{ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

/* No adapters configured
 * All assets will sit on manager unused as "reserves"
 * No need to "claim" as "unbond" will send up to "reserves"
 */
fn single_asset_holder_no_adapters(initial: Uint128, deposit: Uint128) {
    let mut app = App::default();

    let viewing_key = "unguessable".to_string();

    let admin = Addr::unchecked("admin");
    let holder = Addr::unchecked("holder");
    let treasury = Addr::unchecked("treasury");
    let admin_auth = init_admin_auth(&mut app, &admin);

    let token = snip20::InstantiateMsg {
        name: "token".into(),
        admin: Some("admin".into()),
        symbol: "TKN".into(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            address: holder.to_string().clone(),
            amount: initial,
        }]),
        prng_seed: to_binary("").ok().unwrap(),
        config: None,
        query_auth: None,
    }
    .test_init(Snip20::default(), &mut app, admin.clone(), "token", &[])
    .unwrap();

    let manager = treasury_manager::InstantiateMsg {
        admin_auth: admin_auth.into(),
        treasury: treasury.clone().into(),
        viewing_key: viewing_key.clone(),
    }
    .test_init(
        TreasuryManager::default(),
        &mut app,
        admin.clone(),
        "manager",
        &[],
    )
    .unwrap();

    // set holder viewing key
    snip20::ExecuteMsg::SetViewingKey {
        key: viewing_key.clone(),
        padding: None,
    }
    .test_exec(&token, &mut app, holder.clone(), &[])
    .unwrap();

    // Register manager assets
    treasury_manager::ExecuteMsg::RegisterAsset {
        contract: token.clone().into(),
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Add 'holder' as holder
    treasury_manager::ExecuteMsg::AddHolder {
        holder: holder.to_string().clone(),
    }
    .test_exec(&manager, &mut app, admin.clone(), &[])
    .unwrap();

    // Deposit funds into manager
    snip20::ExecuteMsg::Send {
        recipient: manager.address.to_string().clone(),
        recipient_code_hash: None,
        amount: deposit,
        msg: None,
        memo: None,
        padding: None,
    }
    .test_exec(&token, &mut app, holder.clone(), &[])
    .unwrap();

    // Balance Checks

    // manager reported holder balance
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Holder Balance");
        }
        _ => panic!("Query failed"),
    };

    // manager reported treasury balance
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: treasury.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(
                amount,
                Uint128::zero(),
                "Pre-unbond Manager Treasury Balance"
            );
        }
        _ => panic!("Query failed"),
    };

    // Manager reported total asset balance
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Total Balance");
        }
        _ => panic!("Query failed"),
    };

    // holder snip20 bal
    match (snip20::QueryMsg::Balance {
        address: holder.to_string().clone(),
        key: viewing_key.clone(),
    }
    .test_query(&token, &app)
    .unwrap())
    {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(
                amount.u128(),
                initial.u128() - deposit.u128(),
                "Pre-unbond Holder Snip20 balance"
            );
        }
        _ => {
            panic!("Query failed");
        }
    };

    // Unbondable
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond unbondable");
        }
        _ => panic!("Query failed"),
    };

    // Reserves
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Reserves {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond reserves");
        }
        _ => panic!("Query failed"),
    };

    let unbond_amount = Uint128::new(deposit.u128() / 2);

    // unbond from manager
    manager::ExecuteMsg::Manager(manager::SubExecuteMsg::Unbond {
        asset: token.address.to_string().clone().to_string(),
        amount: unbond_amount,
    })
    .test_exec(&manager, &mut app, holder.clone(), &[])
    .unwrap();

    // Unbondable
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(
                amount,
                Uint128::new(deposit.u128() - unbond_amount.u128()),
                "Post-unbond total unbondable"
            );
        }
        _ => panic!("Query failed"),
    };

    match manager::QueryMsg::Manager(manager::SubQueryMsg::Unbondable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(
                amount,
                Uint128::new(deposit.u128() - unbond_amount.u128()),
                "Post-unbond holder unbondable"
            );
        }
        _ => panic!("Query failed"),
    };

    // Unbonding
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond total unbonding");
        }
        _ => panic!("Query failed"),
    };

    match manager::QueryMsg::Manager(manager::SubQueryMsg::Unbonding {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond Holder Unbonding");
        }
        _ => panic!("Query failed"),
    };

    // Claimable (zero as its immediately claimed)
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond total claimable");
        }
        _ => panic!("Query failed"),
    };

    match manager::QueryMsg::Manager(manager::SubQueryMsg::Claimable {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond holder claimable");
        }
        _ => panic!("Query failed"),
    };

    // Manager reflects unbonded
    match manager::QueryMsg::Manager(manager::SubQueryMsg::Balance {
        asset: token.address.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .test_query(&manager, &app)
    .unwrap()
    {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount.u128(), deposit.u128() - unbond_amount.u128());
        }
        _ => {
            panic!("Query failed");
        }
    };

    // user received unbonded
    match (snip20::QueryMsg::Balance {
        address: holder.to_string().clone(),
        key: viewing_key.clone(),
    }
    .test_query(&token, &app)
    .unwrap())
    {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(
                amount.u128(),
                (initial.u128() - deposit.u128()) + unbond_amount.u128(),
                "Post-claim holder snip20 balance"
            );
        }
        _ => {
            panic!("Query failed");
        }
    };
}

macro_rules! single_asset_holder_no_adapters_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (initial, deposit) = $value;
                single_asset_holder_no_adapters(initial, deposit);
            }
        )*
    }
}

single_asset_holder_no_adapters_tests! {
    single_asset_holder_no_adapters_0: (
        Uint128::new(100_000_000),
        Uint128::new(50_000_000),
    ),
}
