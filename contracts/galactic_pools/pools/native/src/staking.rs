// Assuming 'shade_protocol' correctly re-exports all necessary components from 'c_std'
use shade_protocol::c_std::{
    Addr,
    Coin,
    CosmosMsg,
    Deps,
    DistributionMsg,
    FullDelegation,
    StakingMsg,
    StdResult,
    Uint128,
};

use serde::{Deserialize, Serialize};

use crate::state::ConfigInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CustomValidatorRewards {
    pub validator_address: String,
    pub reward: Uint128,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Rewards {
    delegator: String,
}

/// Rewards response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RewardsResponse {
    pub rewards: Vec<ValidatorRewards>,
    pub total: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ValidatorRewards {
    pub validator_address: String,
    pub reward: Vec<Coin>,
}

pub fn get_rewards(
    deps: Deps,
    contract: &Addr,
    config: &ConfigInfo,
) -> StdResult<Vec<CustomValidatorRewards>> {
    // let req: QueryRequest<StakingQuery> = QueryRequest::Staking(StakingQuery::AllDelegations {
    //     delegator: contract.into(),
    // })
    // .into();

    // let request = to_vec(&req).map_err(|serialize_err| {
    //     StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    // })?;

    // let raw = deps.querier.raw_query(&request).unwrap().unwrap();
    // let dels: AllDelegationsResponse = from_binary(&raw).unwrap();
    // let delegations = dels.delegations;

    let mut result_vector: Vec<CustomValidatorRewards> = vec![];

    // //testing

    // if delegations.is_empty() {
    //     return Ok(result_vector);
    // }

    // let mut result_vector: Vec<CustomValidatorRewards> = vec![];

    // let validator_vector = delegations;
    // let mut collect_vals: Vec<String> = vec![];

    // for validator in validator_vector {
    //     collect_vals.push(validator.validator);
    // }

    let vals = &config.validators;

    for val in vals {
        //     let req: QueryRequest<StakingQuery> = QueryRequest::Staking(StakingQuery::Delegation {
        //         delegator: contract.into(),
        //         validator: val.address.clone(),
        //     })
        //     .into();

        //     let request = to_vec(&req).map_err(|serialize_err| {
        //         StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        //     })?;

        //     let raw = deps.querier.raw_query(&request).unwrap().unwrap();
        // let del: DelegationResponse = from_binary(&raw).unwrap();

        let contract_delegation: Option<FullDelegation> =
            deps.querier.query_delegation(contract, &val.address)?;

        if contract_delegation.is_none() {
            continue;
        }

        let coins = &contract_delegation.unwrap().accumulated_rewards;

        // let denom = &validator.reward[0].denom.as_str();
        let mut amount = Uint128::zero();

        // if config.denom.eq(coin.)

        for coin in coins {
            if config.denom.as_str().eq(coin.denom.as_str()) {
                amount += coin.amount;
            }
        }

        let validator_reward_obj = CustomValidatorRewards {
            validator_address: val.address.clone(),
            reward: amount,
        };

        result_vector.push(validator_reward_obj);
    }

    Ok(result_vector)
}

//TODO
// pub fn get_exp(deps: Deps, config: &ConfigInfo) -> StdResult<Uint128> {
//     let msg = experience_contract::msg::QueryMsg::Contract {
//         address: deps
//             .api
//             .addr_humanize(&config.contract_address)?
//             .to_string(),
//         key: config.exp_contract.clone().unwrap().vk,
//     };

//     let rewards = deps
//         .querier
//         .query_wasm_smart::<experience_contract::msg::QueryAnswer>(
//             config.exp_contract.clone().unwrap().contract.hash,
//             config.exp_contract.clone().unwrap().contract.address,
//             &(&msg),
//         )?;

//     let res = rewards;

//     if let experience_contract::msg::QueryAnswer::ContractResponse { unclaimed_exp, .. } = res {
//         return Ok(unclaimed_exp);
//     }

//     Ok(Uint128::zero())
// }

pub fn withdraw(validator: &String) -> CosmosMsg {
    CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
        validator: validator.clone(),
    })
}

pub fn stake(validator: &String, amount: Uint128, denom: &str) -> CosmosMsg {
    CosmosMsg::Staking(StakingMsg::Delegate {
        validator: (validator.clone()),
        amount: Coin {
            denom: denom.to_string(),
            amount,
        },
    })
}

pub fn undelegate(validator: &String, amount: Uint128, denom: &str) -> CosmosMsg {
    CosmosMsg::Staking(StakingMsg::Undelegate {
        validator: (validator.clone()),
        amount: Coin {
            denom: denom.to_string(),
            amount,
        },
    })
}

pub fn redelegate(
    src_validator: &String,
    dst_validator: &String,
    amount: Uint128,
    denom: &str,
) -> CosmosMsg {
    CosmosMsg::Staking(StakingMsg::Redelegate {
        // delegator is automatically set to address of the calling contract
        src_validator: (src_validator.clone()),
        dst_validator: (dst_validator.clone()),
        amount: Coin {
            denom: denom.to_string(),
            amount,
        },
    })
}
