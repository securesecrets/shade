use crate::{
    handle::decay_factor,
    state::{
        account_r, account_total_claimed_r, claim_status_r, config_r, decay_claimed_r,
        total_claimed_r, validate_account_permit,
    },
};
use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage, Uint128};
use shade_protocol::{
    airdrop::{account::AccountPermit, claim_info::RequiredTask, QueryAnswer},
    math::{div, mult},
};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn dates<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    current_date: Option<u64>,
) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;
    Ok(QueryAnswer::Dates {
        start: config.start_date,
        end: config.end_date,
        decay_start: config.decay_start,
        decay_factor: current_date.map(|date| Uint128(100) * decay_factor(date, &config)),
    })
}

pub fn total_claimed<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let claimed: Uint128;
    let total_claimed = total_claimed_r(&deps.storage).load()?;
    if decay_claimed_r(&deps.storage).load()? {
        claimed = total_claimed;
    } else {
        let config = config_r(&deps.storage).load()?;
        claimed = mult(
            div(total_claimed, config.query_rounding)?,
            config.query_rounding,
        );
    }
    Ok(QueryAnswer::TotalClaimed { claimed })
}

pub fn account<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: AccountPermit,
    current_date: Option<u64>,
) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;

    let account_address = validate_account_permit(deps, &permit, config.contract)?;

    let account = account_r(&deps.storage).load(account_address.to_string().as_bytes())?;

    // Calculate eligible tasks
    let config = config_r(&deps.storage).load()?;
    let mut finished_tasks: Vec<RequiredTask> = vec![];
    let mut completed_percentage = Uint128::zero();
    let mut unclaimed_percentage = Uint128::zero();
    for (index, task) in config.task_claim.iter().enumerate() {
        // Check if task has been completed
        let state = claim_status_r(&deps.storage, index)
            .may_load(account_address.to_string().as_bytes())?;

        match state {
            // Ignore if none
            None => {}
            Some(claimed) => {
                finished_tasks.push(task.clone());
                if !claimed {
                    unclaimed_percentage += task.percent;
                } else {
                    completed_percentage += task.percent;
                }
            }
        }
    }

    let mut unclaimed: Uint128;

    if unclaimed_percentage == Uint128(100) {
        unclaimed = account.total_claimable;
    } else {
        unclaimed = unclaimed_percentage.multiply_ratio(account.total_claimable, Uint128(100));
    }

    if let Some(time) = current_date {
        unclaimed = unclaimed * decay_factor(time, &config);
    }

    Ok(QueryAnswer::Account {
        total: account.total_claimable,
        claimed: account_total_claimed_r(&deps.storage)
            .load(account_address.to_string().as_bytes())?,
        unclaimed,
        finished_tasks,
    })
}
