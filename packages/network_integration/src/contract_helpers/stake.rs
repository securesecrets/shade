use serde_json::Result;
use cosmwasm_std::{HumanAddr, Uint128};
use shade_protocol::{staking, snip20, asset::Contract};
use crate::{utils::{print_header, print_contract,
                    GAS, ACCOUNT_KEY, STAKING_FILE},
            contract_helpers::governance::{init_contract}};
use secretcli::{cli_types::NetContract,
                secretcli::{query_contract, test_contract_handle}};
use crate::contract_helpers::minter::get_balance;
use std::{thread, time};
use std::time::UNIX_EPOCH;

pub fn setup_staker(governance: &NetContract, shade: &Contract,
                    staking_account: String) -> Result<NetContract> {
    let staker = init_contract(&governance, "staking".to_string(),
                               STAKING_FILE,
                               staking::InitMsg{
                                   admin: Some(Contract{
                                       address: HumanAddr::from(governance.address.clone()),
                                       code_hash: governance.code_hash.clone() }),
                                   unbond_time: 180,
                                   staked_token: Contract {
                                       address: shade.address.clone(),
                                       code_hash: shade.code_hash.clone()
                                   }
                               })?;

    print_contract(&staker);

    let shade_net = NetContract{
        label: "-".to_string(),
        id: "-".to_string(),
        address: shade.address.to_string(),
        code_hash: shade.code_hash.clone()
    };

    print_header("Testing staking delegation");

    // Query current balance
    let original_balance = get_balance(&shade_net, staking_account.clone());
    let stake_amount = Uint128(7000000);
    let unbond_amount = Uint128(2000000);
    let balance_after_stake = (original_balance - stake_amount).unwrap();

    // Make a query key
    {
        let msg = staking::HandleMsg::SetViewingKey { key: "password".to_string() };

        test_contract_handle(&msg, &staker, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    // Stake some Shade on it
    {
        let msg = snip20::HandleMsg::Send {
            recipient: HumanAddr::from(staker.address.clone()),
            amount: stake_amount,
            msg: None,
            memo: None,
            padding: None
        };

        test_contract_handle(&msg, &shade_net, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    // Check total stake
    assert_eq!(get_total_staked(&staker), stake_amount);

    // Check user stake
    assert_eq!(get_user_stake(&staker, staking_account.clone(),
                              "password".to_string()).staked, stake_amount);

    // Query total Shade now
    assert_eq!(balance_after_stake, get_balance(&shade_net,
                                                 staking_account.clone()));

    print_header("Testing unbonding request");
    // User unbonds
    {
        let msg = staking::HandleMsg::Unbond { amount: unbond_amount };

        test_contract_handle(&msg, &staker, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    // Check if unstaking
    assert_eq!(get_total_staked(&staker), (stake_amount - unbond_amount).unwrap());

    // Check if user unstaking
    {
        let user_stake = get_user_stake(&staker, staking_account.clone(),
                                        "password".to_string());

        assert_eq!(user_stake.staked, (stake_amount - unbond_amount).unwrap());
        assert_eq!(user_stake.unbonding, unbond_amount);
    }

    print_header("Testing unbonding time");
    // User triggers but receives nothing
    {
        let msg = staking::HandleMsg::ClaimUnbond {};

        test_contract_handle(&msg, &staker, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    // Query total Shade now

    assert_eq!(balance_after_stake, get_balance(&shade_net,
                                                 staking_account.clone()));

    // Wait unbonding time
    thread::sleep(time::Duration::from_secs(180));

    // Check if unbonded
    assert_eq!(get_user_stake(&staker, staking_account.clone(),
                              "password".to_string()).unbonded, unbond_amount);

    print_header("Testing unbonding asset release");
    // User triggers and receives something
    {
        let msg = staking::HandleMsg::ClaimUnbond {};

        test_contract_handle(&msg, &staker, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    // Query total Shade now
    assert_eq!((balance_after_stake + unbond_amount), get_balance(&shade_net,
                                             staking_account.clone()));

    Ok(staker)
}

pub fn get_total_staked(staker: &NetContract) -> Uint128 {
    let msg = staking::QueryMsg::TotalStaked {};

    let total_stake: staking::QueryAnswer = query_contract(staker, &msg).unwrap();

    if let staking::QueryAnswer::TotalStaked { total } = total_stake {
        return total
    }

    Uint128::zero()
}

pub struct TestUserStake {
    pub staked: Uint128,
    pub pending_rewards: Uint128,
    pub unbonding: Uint128,
    pub unbonded: Uint128
}

pub fn get_user_stake(staker: &NetContract, address: String, key: String) -> TestUserStake {
    let msg = staking::QueryMsg::UserStake { address: HumanAddr::from(address), key,
        time: time::SystemTime::now().duration_since(UNIX_EPOCH).expect("").as_secs() };

    let query: staking::QueryAnswer = query_contract(staker, &msg).unwrap();

    if let staking::QueryAnswer::UserStake { staked, pending_rewards,
        unbonding, unbonded } = query {
        return TestUserStake {
            staked, pending_rewards, unbonding, unbonded
        }
    }


    TestUserStake {
        staked: Uint128::zero(),
        pending_rewards: Uint128::zero(),
        unbonding: Uint128::zero(),
        unbonded: Uint128::zero()
    }
}
