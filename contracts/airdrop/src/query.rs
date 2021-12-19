use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage, HumanAddr, Uint128, StdError};
use shade_protocol::airdrop::{QueryAnswer, account::AddressProofPermit, claim_info::RequiredTask};
use crate::state::{config_r, claim_status_r,
                   total_claimed_r, validate_permit, account_r};

pub fn config<S: Storage, A: Api, Q: Querier>
(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
        total_claimed: total_claimed_r(&deps.storage).load()?,
    })
}

pub fn dates<S: Storage, A: Api, Q: Querier>
(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;
    Ok(QueryAnswer::Dates { start: config.start_date, end: config.end_date
    })
}

pub fn account<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, address: HumanAddr, permit: AddressProofPermit
) -> StdResult<QueryAnswer> {

    let config = config_r(&deps.storage).load()?;

    let account_address = validate_permit(&deps.storage, &permit, config.contract)?;

    if account_address != address {
        return Err(StdError::unauthorized())
    }

    let account = account_r(&deps.storage).load(address.to_string().as_bytes())?;

    // Calculate eligible tasks
    let config = config_r(&deps.storage).load()?;
    let mut finished_tasks: Vec<RequiredTask> = vec!();
    let mut completed_percentage = Uint128::zero();
    let mut unclaimed_percentage = Uint128::zero();
    for (index, task) in config.task_claim.iter().enumerate() {
        // Check if task has been completed
        let state = claim_status_r(&deps.storage, index).may_load(
            address.to_string().as_bytes())?;

        match state {
            // Ignore if none
            None => {}
            Some(claimed) => {
                finished_tasks.push(task.clone());
                if !claimed {
                    unclaimed_percentage += task.percent;
                }
                else {
                    completed_percentage += task.percent;
                }
            }
        }
    }

    let claimed: Uint128;
    let unclaimed: Uint128;

    if completed_percentage == Uint128(100) {
        claimed = account.total_claimable;
    }
    else {
        claimed = completed_percentage.multiply_ratio(account.total_claimable, Uint128(100));
    }

    if unclaimed_percentage == Uint128(100) {
        unclaimed = account.total_claimable;
    }
    else {
        unclaimed = unclaimed_percentage.multiply_ratio(account.total_claimable, Uint128(100));
    }

    Ok(QueryAnswer::Account {
        total: account.total_claimable,
        claimed,
        unclaimed,
        finished_tasks
    })
}
