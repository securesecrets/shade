use std::convert::TryFrom;

use cosmwasm_std::{
    debug_print, Coin, CosmosMsg, DistQuery, HumanAddr, Querier, RewardsResponse, StakingMsg,
    StdError, StdResult, Storage, Uint128,
};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

use crate::state::{get_address, get_frozen_exchange_rate};
use crate::tokens::query_total_supply;
use crate::types::config::read_config;
use crate::types::killswitch::KillSwitch;
use crate::types::validator_set::get_validator_set;

pub fn exchange_rate<S: Storage, Q: Querier>(store: &S, querier: &Q) -> StdResult<Decimal> {
    let contract_address = get_address(store)?;

    let config = read_config(store)?;

    if KillSwitch::try_from(config.kill_switch)? == KillSwitch::Closed {
        let total_on_chain = get_total_onchain_balance(querier, store, &contract_address)?;
        let tokens =
            query_total_supply(querier, &config.token_contract, &config.token_contract_hash)?
                .u128();
        debug_print(format!(
            "Queried exchange rate - on-chain: {} vs. tokens: {}",
            total_on_chain, tokens
        ));
        let exchange_rate = _calc_exchange_rate(total_on_chain, tokens)?;

        debug_print(format!(
            "calculated exchange rate: {}",
            exchange_rate.to_string()
        ));

        Ok(exchange_rate)
    } else {
        let frozen_x_rate = get_frozen_exchange_rate(store)?;
        debug_print(format!(
            "got frozen exchange rate: {}",
            frozen_x_rate.to_string()
        ));
        Ok(frozen_x_rate)
    }
}

fn _calc_exchange_rate(total_on_chain: u128, tokens: u128) -> Result<Decimal, StdError> {
    let scrt_balance = Decimal::from(total_on_chain as u64);
    let token_bal = Decimal::from(tokens as u64);

    let ratio = if total_on_chain == 0 || tokens == 0 {
        Decimal::one()
    } else {
        token_bal / scrt_balance
    };

    Ok(ratio.round_dp_with_strategy(12, RoundingStrategy::RoundUp))
}

#[allow(dead_code)]
pub fn get_locked_balance<Q: Querier>(
    querier: &Q,
    contract_address: &HumanAddr,
) -> StdResult<u128> {
    let staked_balance = get_bonded(querier, contract_address)?;

    Ok(staked_balance.u128())
}

pub fn get_total_onchain_balance<Q: Querier, S: Storage>(
    querier: &Q,
    storage: &S,
    contract_address: &HumanAddr,
) -> StdResult<u128> {
    let validator_set = get_validator_set(storage)?;
    let locked_balance = validator_set.total_staked();

    let rewards_balance = get_rewards(querier, contract_address)
        .unwrap_or_default()
        .u128();

    Ok(locked_balance + rewards_balance)
}

pub fn get_balance<Q: Querier>(querier: &Q, address: &HumanAddr) -> StdResult<Uint128> {
    let balance = querier.query_balance(address.clone(), &"uscrt".to_string())?;

    Ok(balance.amount)
}

pub fn get_rewards<Q: Querier>(querier: &Q, contract: &HumanAddr) -> StdResult<Uint128> {
    let query = DistQuery::Rewards {
        delegator: contract.clone(),
    };

    let query_rewards: RewardsResponse =
        querier
            .query(&query.into())
            .unwrap_or_else(|_| RewardsResponse {
                rewards: vec![],
                total: vec![],
            });

    if query_rewards.total.is_empty() {
        return Ok(Uint128(0));
    }
    let denom = query_rewards.total[0].denom.as_str();
    query_rewards.total.iter().fold(Ok(Uint128(0)), |racc, d| {
        let acc = racc?;
        if d.denom.as_str() != denom {
            Err(StdError::generic_err(format!(
                "different denoms in bonds: '{}' vs '{}'",
                denom, &d.denom
            )))
        } else {
            Ok(acc + d.amount)
        }
    })
}

pub fn get_rewards_limited<Q: Querier>(
    querier: &Q,
    contract: &HumanAddr,
    amount: u32,
) -> StdResult<RewardsResponse> {
    let query = DistQuery::Rewards {
        delegator: contract.clone(),
    };

    let mut query_rewards: RewardsResponse =
        querier
            .query(&query.into())
            .unwrap_or_else(|_| RewardsResponse {
                rewards: vec![],
                total: vec![],
            });

    if query_rewards.rewards.len() < amount as usize {
        return Ok(query_rewards);
    }

    query_rewards.rewards.sort_by(|a, b| {
        let a_amt = a
            .reward
            .first()
            .unwrap_or(&Coin {
                denom: "".to_string(),
                amount: Default::default(),
            })
            .amount;
        let b_amt = b
            .reward
            .first()
            .unwrap_or(&Coin {
                denom: "".to_string(),
                amount: Default::default(),
            })
            .amount;

        b_amt.cmp(&a_amt)
    });

    query_rewards.rewards.truncate(amount as usize);
    let total_amount: Uint128 = Uint128(
        query_rewards
            .rewards
            .iter()
            .map(|a| {
                a.reward
                    .first()
                    .unwrap_or(&Coin {
                        denom: "".to_string(),
                        amount: Default::default(),
                    })
                    .amount
                    .u128()
            })
            .sum(),
    );
    query_rewards.total = vec![Coin {
        denom: "uscrt".to_string(),
        amount: total_amount,
    }];

    Ok(query_rewards)
    // let denom = query_rewards.total[0].denom.as_str();
    // query_rewards.total.iter().fold(Ok(Uint128(0)), |racc, d| {
    //     let acc = racc?;
    //     if d.denom.as_str() != denom {
    //         Err(StdError::generic_err(format!(
    //             "different denoms in bonds: '{}' vs '{}'",
    //             denom, &d.denom
    //         )))
    //     } else {
    //         Ok(acc + d.amount)
    //     }
    // })
}

// get_bonded returns the total amount of delegations from contract
// it ensures they are all the same denom
// Simon I'm trusting you that this works don't let me down bro
#[allow(dead_code)]
pub fn get_bonded<Q: Querier>(querier: &Q, contract: &HumanAddr) -> StdResult<Uint128> {
    let bonds = querier.query_all_delegations(contract)?;
    if bonds.is_empty() {
        return Ok(Uint128(0));
    }
    let denom = bonds[0].amount.denom.as_str();
    bonds.iter().fold(Ok(Uint128(0)), |racc, d| {
        let acc = racc?;
        if d.amount.denom.as_str() != denom {
            Err(StdError::generic_err(format!(
                "different denoms in bonds: '{}' vs '{}'",
                denom, &d.amount.denom
            )))
        } else {
            Ok(acc + d.amount.amount)
        }
    })
}

// get_bonded returns the total amount of delegations from contract
// it ensures they are all the same denom
// Simon I'm trusting you that this works don't let me down bro
// pub fn get_unbonding<Q: Querier>(querier: &Q, contract: &HumanAddr) -> StdResult<Uint128> {
//     let query = StakingQuery::UnbondingDelegations {
//         delegator: contract.clone(),
//     };
//
//     let query_rewards: UnbondingDelegationsResponse = querier.query(&query.into())?;
//
//     let bonds = query_rewards.delegations;
//     if bonds.is_empty() {
//         return Ok(Uint128(0));
//     }
//     let denom = bonds[0].amount.denom.as_str();
//     bonds.iter().fold(Ok(Uint128(0)), |racc, d| {
//         let acc = racc?;
//         if d.amount.denom.as_str() != denom {
//             Err(StdError::generic_err(format!(
//                 "different denoms in bonds: '{}' vs '{}'",
//                 denom, &d.amount.denom
//             )))
//         } else {
//             Ok(acc + d.amount.amount)
//         }
//     })
// }

pub fn withdraw_to_self(validator: &str) -> CosmosMsg {
    CosmosMsg::Staking(StakingMsg::Withdraw {
        validator: HumanAddr(validator.to_string()),
        recipient: None,
    })
}

// pub fn restake(validator: &String, amount: u128) -> Vec<CosmosMsg> {
//     vec![
//         CosmosMsg::Staking(StakingMsg::Withdraw {
//             validator: HumanAddr(validator.clone()),
//             recipient: None,
//         }),
//         CosmosMsg::Staking(StakingMsg::Delegate {
//             validator: HumanAddr(validator.clone()),
//             amount: Coin {
//                 denom: "uscrt".to_string(),
//                 amount: Uint128(amount),
//             },
//         }),
//     ]
// }

pub fn stake_msg(validator: &str, amount: u128) -> CosmosMsg {
    CosmosMsg::Staking(StakingMsg::Delegate {
        validator: HumanAddr(validator.to_string()),
        amount: Coin {
            denom: "uscrt".to_string(),
            amount: Uint128(amount),
        },
    })
}

pub fn undelegate_msg(validator: &str, amount: u128) -> CosmosMsg {
    CosmosMsg::Staking(StakingMsg::Undelegate {
        validator: HumanAddr(validator.to_string()),
        amount: Coin {
            denom: "uscrt".to_string(),
            amount: Uint128(amount),
        },
    })
}

pub fn redelegate_msg(from: &str, to: &str, amount: u128) -> CosmosMsg {
    CosmosMsg::Staking(StakingMsg::Redelegate {
        src_validator: HumanAddr(from.to_string()),
        amount: Coin {
            denom: "uscrt".to_string(),
            amount: Uint128(amount),
        },
        dst_validator: HumanAddr(to.to_string()),
    })
}
