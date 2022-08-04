use shade_protocol::c_std::{
    coins, from_binary, to_binary,
    Addr, StdError,
    Binary, StdResult, Env,
    Uint128,
};

//use shade_protocol::secret_toolkit::snip20;

use shade_protocol::{
    snip20,
    dao::{
        treasury_manager,
        adapter,
        manager,
    },
    utils::{
        asset::Contract,
        MultiTestable,
        InstantiateCallback,
        ExecuteCallback,
        Query,
    },
};
use shade_protocol::multi_test::{ App };
use shade_multi_test::multi::{
    treasury_manager::TreasuryManager,
    snip20::Snip20,
};


/* No adapters configured
 * All assets will sit on manager unused as "reserves"
 * No need to "claim" as "unbond" will send up to "reserves"
 */
fn single_asset_holder_no_adapters(
    initial: Uint128, 
    deposit: Uint128,
) {

    let mut app = App::default();

    let viewing_key = "unguessable".to_string();

    let admin = Addr::unchecked("admin");
    let holder = Addr::unchecked("holder");
    let treasury = Addr::unchecked("treasury");

    let token = snip20::InstantiateMsg {
        name: "token".into(),
        admin: Some("admin".into()),
        symbol: "TKN".into(),
        decimals: 6,
        initial_balances: Some(vec![
            snip20::InitialBalance {
                address: holder.to_string().clone(),
                amount: initial,
            },
        ]),
        prng_seed: to_binary("").ok().unwrap(),
        config: None,
    }.test_init(Snip20::default(), &mut app, admin.clone(), "token", &[]).unwrap();

    let manager = treasury_manager::InstantiateMsg {
        admin: Some(admin.clone()),
        treasury: treasury.clone(),
        viewing_key: viewing_key.clone(),
    }.test_init(TreasuryManager::default(), &mut app, admin.clone(), "manager", &[]).unwrap();

    // set holder viewing key
    snip20::ExecuteMsg::SetViewingKey{
        key: viewing_key.clone(),
        padding: None,
    }.test_exec(&token, &mut app, admin.clone(), &[]);

    // Register manager assets
    treasury_manager::ExecuteMsg::RegisterAsset {
        contract: Contract {
            address: token.address.clone(),
            code_hash: token.code_hash.clone(),
        },
    }.test_exec(&manager, &mut app, admin.clone(), &[]);

    // Add 'holder' as holder
    treasury_manager::ExecuteMsg::AddHolder {
        holder: holder.clone(),
    }.test_exec(&manager, &mut app, admin.clone(), &[]);

    // Deposit funds into manager
    snip20::ExecuteMsg::Send {
        recipient: manager.address.to_string().clone(),
        recipient_code_hash: None,
        amount: deposit,
        msg: None,
        memo: None,
        padding: None,
    }.test_exec(&token, &mut app, admin.clone(), &[]);
    
    // Balance Checks

    // manager reported holder balance
    match (manager::QueryMsg::Manager(
        manager::SubQueryMsg::Balance {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap()) {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Holder Balance");
        },
        _ => panic!("Query failed"),
    };

    // manager reported treasury balance
    match (manager::QueryMsg::Manager(
        manager::SubQueryMsg::Balance {
            asset: token.address.clone(),
            holder: treasury.clone(),
        }
    ).test_query(&manager, &app).unwrap()) {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, Uint128::zero(), "Pre-unbond Manager Treasury Balance");
        },
        _ => panic!("Query failed"),
    };

    // Manager reported total asset balance
    match (manager::QueryMsg::Manager(
        manager::SubQueryMsg::Balance {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap()) {
        manager::QueryAnswer::Balance { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond Manager Total Balance");
        }
        _ => panic!("Query failed"),
    };

    // holder snip20 bal
    match (snip20::QueryMsg::Balance {
        address: holder.to_string().clone(),
        key: viewing_key.clone(),
    }.test_query(&token, &app).unwrap()) {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount.u128(), initial.u128() - deposit.u128(), "Pre-unbond Holder Snip20 balance");
        },
        _ => {
            panic!("Query failed");
        }
    };

    // Unbondable
    match manager::QueryMsg::Manager(
        manager::SubQueryMsg::Unbondable {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap() {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond unbondable");
        }
        _ => panic!("Query failed"),
    };

    // Reserves
    match manager::QueryMsg::Manager(
        manager::SubQueryMsg::Reserves {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap() {
        manager::QueryAnswer::Reserves { amount } => {
            assert_eq!(amount, deposit, "Pre-unbond reserves");
        }
        _ => panic!("Query failed"),
    };

    let unbond_amount = Uint128::new(deposit.u128() / 2);

    // unbond from manager
    manager::ExecuteMsg::Manager(
        manager::SubExecuteMsg::Unbond {
            asset: token.address.clone(),
            amount: unbond_amount,
        }
    ).test_exec(&manager, &mut app, admin.clone(), &[]);

    // Unbondable
    match manager::QueryMsg::Manager(
        manager::SubQueryMsg::Unbondable {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap() {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, Uint128::new(deposit.u128() - unbond_amount.u128()), "Post-unbond total unbondable");
        }
        _ => panic!("Query failed"),
    };

    match manager::QueryMsg::Manager(
        manager::SubQueryMsg::Unbondable {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap() {
        manager::QueryAnswer::Unbondable { amount } => {
            assert_eq!(amount, Uint128::new(deposit.u128() - unbond_amount.u128()), "Post-unbond holder unbondable");
        }
        _ => panic!("Query failed"),
    };

    // Unbonding
    match manager::QueryMsg::Manager(
        manager::SubQueryMsg::Unbonding {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap() {
        manager::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond total unbonding");
        }
        _ => panic!("Query failed"),
    };

    match manager::QueryMsg::Manager(
        manager::SubQueryMsg::Unbonding {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap() {
        manager::QueryAnswer::Unbonding { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond Holder Unbonding");
        }
        _ => panic!("Query failed"),
    };

    // Claimable (zero as its immediately claimed)
    match manager::QueryMsg::Manager(
        manager::SubQueryMsg::Claimable {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap() {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond total claimable");
        }
        _ => panic!("Query failed"),
    };

    match manager::QueryMsg::Manager(
        manager::SubQueryMsg::Claimable {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap() {
        manager::QueryAnswer::Claimable { amount } => {
            assert_eq!(amount, Uint128::zero(), "Post-unbond holder claimable"); 
        }
        _ => panic!("Query failed"),
    };

    // Manager reflects unbonded
    match (manager::QueryMsg::Manager(
        manager::SubQueryMsg::Balance {
            asset: token.address.clone(),
            holder: holder.clone(),
        }
    ).test_query(&manager, &app).unwrap()) {
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
    }.test_query(&token, &app).unwrap()) {
        snip20::QueryAnswer::Balance { amount } => {
            assert_eq!(amount.u128(), (initial.u128() - deposit.u128()) + unbond_amount.u128(), "Post-claim holder snip20 balance");
        },
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
/*
single_asset_holder_no_adapters_tests! {
    single_asset_holder_no_adapters_0: (
        Uint128::new(100_000_000),
        Uint128::new(50_000_000),
    ),
}
*/
