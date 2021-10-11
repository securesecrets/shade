use serde_json::Result;
use cosmwasm_std::{HumanAddr, Uint128, to_binary};
use shade_protocol::{staking, snip20, asset::Contract};
use crate::{utils::{print_header, print_contract, print_epoch_info, print_vec,
                    STORE_GAS, GAS, VIEW_KEY, ACCOUNT_KEY},
            contract_helpers::governance::{init_contract, get_contract, add_contract,
                                           create_proposal, trigger_latest_proposal}};
use secretcli::{cli_types::NetContract,
                secretcli::{query_contract, test_contract_handle, test_inst_init}};
use crate::contract_helpers::minter::get_balance;

pub fn setup_staker(governance: &NetContract, shade: &Contract, staking_account: String) -> Result<NetContract> {
    let staker = init_contract(&governance, "staking".to_string(),
                               "../../compiled/staking.wasm.gz",
                               staking::InitMsg{
                                   admin: Some(Contract{
                                       address: HumanAddr::from(governance.address.clone()),
                                       code_hash: governance.code_hash.clone() }),
                                   unbond_time: 0,
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

    /// Query current balance
    let original_balance = get_balance(&shade_net, staking_account.clone());
    let stake_amount = Uint128(100);

    /// Stake some Shade on it
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

    /// Check total stake
    assert_eq!(get_total_staked(&staker), stake_amount);

    /// Query total Shade now
    {
        let expected_total_shade = (original_balance - stake_amount).unwrap();

        assert_eq!(expected_total_shade, get_balance(&shade_net,
                                                     staking_account.clone()));
    }

    /// User unbonds
    {
        let msg = staking::HandleMsg::Unbond { amount: stake_amount };

        test_contract_handle(&msg, &staker, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    /// Check if unstaking
    assert_eq!(get_total_staked(&staker), Uint128(0));

    /// User triggers
    {
        let msg = staking::HandleMsg::TriggerUnbonds {};

        test_contract_handle(&msg, &staker, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    /// Query total Shade now
    {
        assert_eq!(original_balance, get_balance(&shade_net,
                                                     staking_account.clone()));
    }

    Ok(staker)
}

pub fn get_total_staked(staker: &NetContract) -> Uint128 {
    let msg = staking::QueryMsg::TotalStaked {};

    let total_stake: staking::QueryAnswer = query_contract(staker, &msg).unwrap();

    if let staking::QueryAnswer::TotalStaked { total } = total_stake {
        return total
    }

    Uint128(0)
}