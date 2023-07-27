use crate::{
    handle::decay_factor,
    state::{
        account_r,
        account_total_claimed_r,
        account_viewkey_r,
        claim_status_r,
        config_r,
        decay_claimed_r,
        total_claimed_r,
        validate_account_permit,
    },
};
use shade_protocol::{
    airdrop::{
        account::{AccountKey, AccountPermit},
        claim_info::RequiredTask,
        errors::invalid_viewing_key,
        QueryAnswer,
    },
    c_std::{Addr, Deps, StdResult, Uint128},
    query_authentication::viewing_keys::ViewingKey,
};

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(deps.storage).load()?,
    })
}

pub fn dates(deps: Deps, current_date: Option<u64>) -> StdResult<QueryAnswer> {
    let config = config_r(deps.storage).load()?;
    Ok(QueryAnswer::Dates {
        start: config.start_date,
        end: config.end_date,
        decay_start: config.decay_start,
        decay_factor: current_date.map(|date| Uint128::new(100u128) * decay_factor(date, &config)),
    })
}

pub fn total_claimed(deps: Deps) -> StdResult<QueryAnswer> {
    let claimed: Uint128;
    let total_claimed = total_claimed_r(deps.storage).load()?;
    if decay_claimed_r(deps.storage).load()? {
        claimed = total_claimed;
    } else {
        let config = config_r(deps.storage).load()?;
        claimed = total_claimed.checked_div(config.query_rounding)? * config.query_rounding;
    }
    Ok(QueryAnswer::TotalClaimed { claimed })
}

fn account_information(
    deps: Deps,
    account_address: Addr,
    current_date: Option<u64>,
) -> StdResult<QueryAnswer> {
    let account = account_r(deps.storage).load(account_address.to_string().as_bytes())?;

    // Calculate eligible tasks
    let config = config_r(deps.storage).load()?;
    let mut finished_tasks: Vec<RequiredTask> = vec![];
    let mut completed_percentage = Uint128::zero();
    let mut unclaimed_percentage = Uint128::zero();
    for (index, task) in config.task_claim.iter().enumerate() {
        // Check if task has been completed
        let state =
            claim_status_r(deps.storage, index).may_load(account_address.to_string().as_bytes())?;

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

    if unclaimed_percentage == Uint128::new(100u128) {
        unclaimed = account.total_claimable;
    } else {
        unclaimed =
            unclaimed_percentage.multiply_ratio(account.total_claimable, Uint128::new(100u128));
    }

    if let Some(time) = current_date {
        unclaimed = unclaimed * decay_factor(time, &config);
    }

    Ok(QueryAnswer::Account {
        total: account.total_claimable,
        claimed: account_total_claimed_r(deps.storage)
            .load(account_address.to_string().as_bytes())?,
        unclaimed,
        finished_tasks,
        addresses: account.addresses,
    })
}

pub fn account(
    deps: Deps,
    permit: AccountPermit,
    current_date: Option<u64>,
) -> StdResult<QueryAnswer> {
    let config = config_r(deps.storage).load()?;
    account_information(
        deps,
        validate_account_permit(deps, &permit, config.contract)?,
        current_date,
    )
}

pub fn account_with_key(
    deps: Deps,
    account: Addr,
    key: String,
    current_date: Option<u64>,
) -> StdResult<QueryAnswer> {
    // Validate address
    let stored_hash = account_viewkey_r(deps.storage).load(account.to_string().as_bytes())?;

    if !AccountKey(key).compare(&stored_hash) {
        return Err(invalid_viewing_key());
    }

    account_information(deps, account, current_date)
}
