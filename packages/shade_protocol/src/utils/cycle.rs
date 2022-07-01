use chrono::prelude::*;
use cosmwasm_std::{Env, StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cycle {
    Once,
    Constant,
    /*
    Block {
        blocks: Uint128,
    },
    */
    Yearly { years: Uint128 },
    Monthly { months: Uint128 },
    Daily { days: Uint128 },
    Hourly { hours: Uint128 },
    Minutes { minutes: Uint128 },
    Seconds { seconds: Uint128 },
}

pub fn parse_utc_datetime(rfc3339: &String) -> StdResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&rfc3339)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| StdError::generic_err(format!("Failed to parse rfc3339 datetime {}", rfc3339)))
}

pub fn utc_now(env: &Env) -> DateTime<Utc> {
    DateTime::from_utc(NaiveDateTime::from_timestamp(env.block.time as i64, 0), Utc)
}

pub fn exceeds_cycle(now: &DateTime<Utc>, last_refresh: &DateTime<Utc>, cycle: Cycle) -> bool {
    match cycle {
        Cycle::Constant => true,
        Cycle::Once => false,
        //Cycle::Block { blocks } => {},
        Cycle::Seconds { seconds } => {
            seconds >= Uint128((now.timestamp() - last_refresh.timestamp()) as u128)
        }
        Cycle::Minutes { minutes } => {
            minutes
                >= Uint128(
                    ((now.timestamp() - last_refresh.timestamp()) / 60)
                        .try_into()
                        .unwrap(),
                )
        }
        Cycle::Hourly { hours } => {
            hours
                >= Uint128(
                    ((now.timestamp() - last_refresh.timestamp()) / 60 / 60)
                        .try_into()
                        .unwrap(),
                )
        }
        Cycle::Daily { days } => {
            now.num_days_from_ce() - last_refresh.num_days_from_ce() >= days.u128() as i32
        }
        Cycle::Monthly { months } => {
            let mut month_diff = 0u32;

            if now.year() > last_refresh.year() {
                month_diff = (12u32 - last_refresh.month()) + now.month();
            } else {
                month_diff = now.month() - last_refresh.month();
            }

            month_diff >= months.u128() as u32
        }
        Cycle::Yearly { years } => now.year_ce().1 - last_refresh.year_ce().1 >= years.u128() as u32,
    }
}
