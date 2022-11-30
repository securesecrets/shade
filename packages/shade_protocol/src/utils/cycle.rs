use crate::{
    c_std::{Env, StdError, StdResult, Timestamp, Uint128},
    chrono::prelude::*,
};

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
    if now.timestamp() < last_refresh.timestamp() && cycle != Cycle::Constant {
        return false;
    }
    match cycle {
        Cycle::Constant => true,
        Cycle::Once => false,
        Cycle::Seconds { seconds } => {
            seconds <= Uint128::new((now.timestamp() - last_refresh.timestamp()) as u128)
        }
        Cycle::Minutes { minutes } => {
            println!(
                "{} >= {} -> {} - {}",
                minutes,
                ((now.timestamp() - last_refresh.timestamp()) / 60),
                now.timestamp(),
                last_refresh.timestamp()
            );
            minutes
                <= Uint128::new(
                    ((now.timestamp() - last_refresh.timestamp()) / 60)
                        .try_into()
                        .unwrap(),
                )
        }
        Cycle::Hourly { hours } => {
            println!(
                "{} >= {} -> {} - {}",
                hours,
                ((now.timestamp() - last_refresh.timestamp()) / 60 / 60),
                now.timestamp(),
                last_refresh.timestamp()
            );
            hours
                <= Uint128::new(
                    ((now.timestamp() - last_refresh.timestamp()) / 60 / 60)
                        .try_into()
                        .unwrap(),
                )
        }
        Cycle::Daily { days } => {
            days.u128() as i32 <= now.num_days_from_ce() - last_refresh.num_days_from_ce()
        }
        Cycle::Monthly { months } => {
            let month_diff: u32;

            if now.year() > last_refresh.year() {
                month_diff = (12u32 - last_refresh.month()) + now.month();
            } else {
                month_diff = now.month() - last_refresh.month();
            }

            months.u128() as u32 <= month_diff
        }
        Cycle::Yearly { years } => {
            years.u128() as u32 <= now.year_ce().1 - last_refresh.year_ce().1
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn test_exceeds_cycle(last_refresh: String, now: String, cycle: Cycle, exceeds: bool) {
        let last_refresh = parse_utc_datetime(&last_refresh).unwrap();
        let now = parse_utc_datetime(&now).unwrap();
        assert_eq!(exceeds_cycle(&now, &last_refresh, cycle), exceeds);
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
        constant_cycle: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T23:59:59.59Z",
            Cycle::Constant,
            true,
        ),
        once_cycle: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T23:59:59.59Z",
            Cycle::Once,
            false,
        ),
        seconds_cycle_well_under: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:00:05.00Z", // 5 sec diff
            Cycle::Seconds { seconds: Uint128::new(10) },
            false,
        ),
        seconds_cycle_just_short: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:00:09.99Z", // 9.99 sec diff
            Cycle::Seconds { seconds: Uint128::new(10) },
            false,
        ),
        seconds_cycle_exact: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:00:10.00Z", // 10 sec diff
            Cycle::Seconds { seconds: Uint128::new(10) },
            true,
        ),
        seconds_cycle_well_over: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:20:00.00Z", // 20 min diff
            Cycle::Seconds { seconds: Uint128::new(10) },
            true,
        ),
        seconds_cycle_overflow: (
            "2019-10-12T00:20:00.00Z", // 20 min diff
            "2019-10-12T00:00:00.00Z",
            Cycle::Seconds { seconds: Uint128::new(10) },
            false,
        ),
        minutes_cycle_well_under: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:00:30.00Z", // 30 sec diff
            Cycle::Minutes { minutes: Uint128::new(1) },
            false,
        ),
        minutes_cycle_just_short: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:00:59.99Z", // 59.99 sec diff
            Cycle::Minutes { minutes: Uint128::new(1) },
            false,
        ),
        minutes_cycle_exact: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:01:00.00Z", // 1 min diff
            Cycle::Minutes { minutes: Uint128::new(1) },
            true,
        ),
        minutes_cycle_well_over: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:20:00.00Z", // 20 min diff
            Cycle::Minutes { minutes: Uint128::new(1) },
            true,
        ),
        minutes_cycle_overflow: (
            "2019-10-12T00:20:00.00Z", // 20 min diff
            "2019-10-12T00:00:00.00Z",
            Cycle::Minutes { minutes: Uint128::new(1) },
            false,
        ),
        hours_cycle_well_under: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:30:00.00Z", // 30 min diff
            Cycle::Hourly { hours: Uint128::new(1) },
            false,
        ),
        hours_cycle_just_short: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T00:59:59.99Z", // 59 mint 59.99 sec diff
            Cycle::Hourly { hours: Uint128::new(1) },
            false,
        ),
        hours_cycle_exact: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T01:00:00.00Z", // 1 hour diff
            Cycle::Hourly { hours: Uint128::new(1) },
            true,
        ),
        hours_cycle_well_over: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T01:30:00.00Z", // 1 hour 30 min diff
            Cycle::Hourly { hours: Uint128::new(1) },
            true,
        ),
        hours_cycle_overflow: (
            "2019-10-12T01:30:00.00Z", // 1 hour 30 min diff
            "2019-10-12T00:00:00.00Z",
            Cycle::Hourly { hours: Uint128::new(1) },
            false,
        ),
        daily_cycle_well_under: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T23:00:00.00Z", // 23 hour diff
            Cycle::Daily { days: Uint128::new(1)},
            false,
        ),
        daily_cycle_just_short: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-12T23:59:59.99Z", // 23 hours, 59 min, 59.99 sec diff
            Cycle::Daily { days: Uint128::new(1) },
            false,
        ),
        daily_cycle_exact: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-13T00:00:00.00Z", // 1 day diff
            Cycle::Daily { days: Uint128::new(1) },
            true,
        ),
        daily_cycle_well_over: (
            "2019-10-12T00:00:00.00Z",
            "2019-10-13T12:00:00.00Z", // 1 day 12 hr diff
            Cycle::Daily { days: Uint128::new(1) },
            true,
        ),
        daily_cycle_overflow: (
            "2019-10-13T12:00:00.00Z", // 1 day 12 hr diff
            "2019-10-12T00:00:00.00Z",
            Cycle::Daily { days: Uint128::new(1) },
            false,
        ),
        monthly_cycle_well_under: (
            "2019-10-01T00:00:00.00Z",
            "2019-10-15T00:00:00.00Z", // 14 day diff
            Cycle::Monthly { months: Uint128::new(1) },
            false,
        ),
        monthly_cycle_just_short: (
            "2019-10-01T00:00:00.00Z",
            "2019-10-31T23:59:59.99Z", // 30 day, 23 hr, 59 min, 59.99 sec diff
            Cycle::Monthly { months: Uint128::new(1) },
            false,
        ),
        monthly_cycle_exact: (
            "2019-10-01T00:00:00.00Z",
            "2019-11-01T00:00:00.00Z", // 1 mo diff
            Cycle::Monthly { months: Uint128::new(1) },
            true,
        ),
        monthly_cycle_well_over: (
            "2019-10-01T00:00:00.00Z",
            "2019-11-15T00:00:00.00Z", // 1 mo, 15 day diff
            Cycle::Monthly { months: Uint128::new(1) },
            true,
        ),
        monthly_cycle_overflow: (
            "2019-11-15T00:00:00.00Z", // 1 mo, 15 day diff
            "2019-10-01T00:00:00.00Z",
            Cycle::Monthly { months: Uint128::new(1) },
            false,
        ),
        yearly_cycle_well_under: (
            "2019-01-01T00:00:00.00Z",
            "2019-10-01T00:00:00.00Z", // 9 mo diff
            Cycle::Yearly { years: Uint128::new(1) },
            false,
        ),
        yearly_cycle_just_short: (
            "2019-01-01T00:00:00.00Z",
            "2019-12-31T23:59:59.99Z", // 11 mo, 31 days, 23 hrs, 59 mins 59.99 sec diff
            Cycle::Yearly { years: Uint128::new(1) },
            false,
        ),
        yearly_cycle_exact: (
            "2019-01-01T00:00:00.00Z",
            "2020-01-01T00:00:00.00Z", // 1 yr diff
            Cycle::Yearly { years: Uint128::new(1) },
            true,
        ),
        yearly_cycle_well_over: (
            "2019-01-01T00:00:00.00Z",
            "2020-06-01T00:00:00.00Z", // 1 yr, 6 mo diff
            Cycle::Yearly { years: Uint128::new(1) },
            true,
        ),
        yearly_cycle_overflow: (
            "2020-06-01T00:00:00.00Z", // 1 yr, 6 mo diff
            "2019-01-01T00:00:00.00Z",
            Cycle::Yearly { years: Uint128::new(1) },
            false,
        ),
    }
}
