use crate::c_std::{Env, StdError, StdResult, Timestamp, Uint128};
use chrono::prelude::*;

use cosmwasm_schema::cw_serde;
use std::convert::TryInto;

#[cw_serde]
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

pub fn utc_from_seconds(seconds: i64) -> DateTime<Utc> {
    DateTime::from_utc(NaiveDateTime::from_timestamp(seconds, 0), Utc)
}

pub fn utc_from_timestamp(timestamp: Timestamp) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDateTime::from_timestamp(timestamp.seconds() as i64, 0),
        Utc,
    )
}

pub fn parse_utc_datetime(rfc3339: &String) -> StdResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&rfc3339)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| StdError::generic_err(format!("Failed to parse rfc3339 datetime {}", rfc3339)))
}

pub fn utc_now(env: &Env) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDateTime::from_timestamp(env.block.time.seconds() as i64, 0),
        Utc,
    )
}

pub fn exceeds_cycle(now: &DateTime<Utc>, last_refresh: &DateTime<Utc>, cycle: Cycle) -> bool {
    match cycle {
        Cycle::Constant => true,
        Cycle::Once => false,
        //Cycle::Block { blocks } => {},
        Cycle::Seconds { seconds } => {
            seconds >= Uint128::new((now.timestamp() - last_refresh.timestamp()) as u128)
        }
        Cycle::Minutes { minutes } => {
            minutes
                >= Uint128::new(
                    ((now.timestamp() - last_refresh.timestamp()) / 60)
                        .try_into()
                        .unwrap(),
                )
        }
        Cycle::Hourly { hours } => {
            hours
                >= Uint128::new(
                    ((now.timestamp() - last_refresh.timestamp()) / 60 / 60)
                        .try_into()
                        .unwrap(),
                )
        }
        Cycle::Daily { days } => {
            now.num_days_from_ce() - last_refresh.num_days_from_ce() >= days.u128() as i32
        }
        Cycle::Monthly { months } => {
            let month_diff: u32;

            if now.year() > last_refresh.year() {
                month_diff = (12u32 - last_refresh.month()) + now.month();
            } else {
                month_diff = now.month() - last_refresh.month();
            }

            month_diff >= months.u128() as u32
        }
        Cycle::Yearly { years } => {
            now.year_ce().1 - last_refresh.year_ce().1 >= years.u128() as u32
        }
    }
}

#[cfg(test)]
mod test {

    fn test_exceeds_cycle(last_refresh: String, now: String, cycle: Cycle, exceeds: bool) {
        let last_refresh = parse_utc_datetime(&last_refresh);
        let now = parse_utc_datitem(&now);
        assert_eq!(exceeds_cycle(&now, &last_refresh, &cycle), exceeds);
    }

    macro_rules! exceeds_cycle_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (start, now, cycle, exceeds) = $value;
                    test_exceeds_cycle(start.to_string(), now.to_string(), cycle, exceeds);
                }
            )*
        }
    }

    exceeds_cycle_tests! {
        daily_cycle_1day: (
            "2019-10-12T07:20:50.52Z",
            "2019-10-13T07:20:50.52Z",
            Cycle::Daily,
            true,
        ),
        daily_cycle_1y: (
            "2019-10-12T07:20:50.51Z",
            "2020-10-12T07:20:50.51Z",
            Cycle::Daily,
            false,
        ),
        daily_cycle_23h: (
            "2019-10-12T07:20:50.52Z",
            "2019-10-13T06:20:50.52Z",
            Cycle::Daily,
            false,
        ),
        daily_cycle_1s_short: (
            "2019-10-12T07:20:50.51Z",
            "2019-10-13T07:20:49.51Z",
            Cycle::Daily,
            false,
        ),
    }
}
