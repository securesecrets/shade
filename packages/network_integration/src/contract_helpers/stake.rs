use crate::{
    contract_helpers::{governance::init_with_gov, minter::get_balance},
    utils::{print_contract, print_header, ACCOUNT_KEY, GAS, STAKING_FILE},
};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::HumanAddr;
use secretcli::secretcli::Report;
use secretcli::{
    cli_types::NetContract,
    secretcli::{handle, query},
};
use serde_json::Result;
use shade_protocol::utils::asset::Contract;
use shade_protocol::{snip20, staking};
use std::{thread, time, time::UNIX_EPOCH};

pub fn setup_staker(
    governance: &NetContract,
    shade: &Contract,
    staking_account: String,
    report: &mut Vec<Report>,
) -> Result<NetContract> {
    let staker = init_with_gov(
        governance,
        "staking".to_string(),
        STAKING_FILE,
        staking::InitMsg {
            admin: Some(Contract {
                address: HumanAddr::from(governance.address.clone()),
                code_hash: governance.code_hash.clone(),
            }),
            unbond_time: 180,
            staked_token: Contract {
                address: shade.address.clone(),
                code_hash: shade.code_hash.clone(),
            },
        },
        report,
    )?;

    print_contract(&staker);

    let shade_net = NetContract {
        label: "-".to_string(),
        id: "-".to_string(),
        address: shade.address.to_string(),
        code_hash: shade.code_hash.clone(),
    };

    print_header("Testing staking delegation");

    // Query current balance
    let original_balance = get_balance(&shade_net, staking_account.clone());
    let stake_amount = Uint128::new(7000000u128);
    let unbond_amount = Uint128::new(2000000u128);
    let balance_after_stake = original_balance - stake_amount;

    // Make a query key
    {
        let msg = staking::HandleMsg::SetViewingKey {
            key: "password".to_string(),
        };

        handle(
            &msg,
            &staker,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            report,
            None,
        )?;
    }

    // Stake some Shade on it
    {
        let msg = snip20::HandleMsg::Send {
            recipient: HumanAddr::from(staker.address.clone()),
            amount: stake_amount,
            msg: None,
            memo: None,
            padding: None,
        };

        handle(
            &msg,
            &shade_net,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            report,
            None,
        )?;
    }

    // Check total stake
    assert_eq!(get_total_staked(&staker), stake_amount);

    // Check user stake
    assert_eq!(
        get_user_stake(&staker, staking_account.clone(), "password".to_string()).staked,
        stake_amount
    );

    // Query total Shade now
    assert_eq!(
        balance_after_stake,
        get_balance(&shade_net, staking_account.clone())
    );

    print_header("Testing unbonding request");
    // User unbonds
    {
        let msg = staking::HandleMsg::Unbond {
            amount: unbond_amount,
        };

        handle(
            &msg,
            &staker,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            report,
            None,
        )?;
    }

    // Check if unstaking
    assert_eq!(get_total_staked(&staker), stake_amount - unbond_amount);

    // Check if user unstaking
    {
        let user_stake = get_user_stake(&staker, staking_account.clone(), "password".to_string());

        assert_eq!(user_stake.staked, stake_amount - unbond_amount);
        assert_eq!(user_stake.unbonding, unbond_amount);
    }

    print_header("Testing unbonding time");
    // User triggers but receives nothing
    {
        let msg = staking::HandleMsg::ClaimUnbond {};

        handle(
            &msg,
            &staker,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            report,
            None,
        )?;
    }

    // Query total Shade now

    assert_eq!(
        balance_after_stake,
        get_balance(&shade_net, staking_account.clone())
    );

    // Wait unbonding time
    thread::sleep(time::Duration::from_secs(180));

    // Check if unbonded
    assert_eq!(
        get_user_stake(&staker, staking_account.clone(), "password".to_string()).unbonded,
        unbond_amount
    );

    print_header("Testing unbonding asset release");
    // User triggers and receives something
    {
        let msg = staking::HandleMsg::ClaimUnbond {};

        handle(
            &msg,
            &staker,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            report,
            None,
        )?;
    }

    // Query total Shade now
    assert_eq!(
        (balance_after_stake + unbond_amount),
        get_balance(&shade_net, staking_account)
    );

    Ok(staker)
}

pub fn get_total_staked(staker: &NetContract) -> Uint128 {
    let msg = staking::QueryMsg::TotalStaked {};

    let total_stake: staking::QueryAnswer = query(staker, &msg, None).unwrap();

    if let staking::QueryAnswer::TotalStaked { total } = total_stake {
        return total;
    }

    Uint128::zero()
}

pub struct TestUserStake {
    pub staked: Uint128,
    pub pending_rewards: Uint128,
    pub unbonding: Uint128,
    pub unbonded: Uint128,
}

pub fn get_user_stake(staker: &NetContract, address: String, key: String) -> TestUserStake {
    let msg = staking::QueryMsg::UserStake {
        address: HumanAddr::from(address),
        key,
        time: time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("")
            .as_secs(),
    };

    let query: staking::QueryAnswer = query(staker, &msg, None).unwrap();

    if let staking::QueryAnswer::UserStake {
        staked,
        pending_rewards,
        unbonding,
        unbonded,
    } = query
    {
        return TestUserStake {
            staked,
            pending_rewards,
            unbonding,
            unbonded,
        };
    }

    TestUserStake {
        staked: Uint128::zero(),
        pending_rewards: Uint128::zero(),
        unbonding: Uint128::zero(),
        unbonded: Uint128::zero(),
    }
}
