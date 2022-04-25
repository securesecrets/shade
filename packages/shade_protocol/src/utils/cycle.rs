use cosmwasm_std::{
    Uint128, StdResult, StdError,
};
use crate::{
    utils::{
        asset::Contract, 
        generic_response::ResponseStatus
    },
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cycle {
    Once,
    Constant,
    Yearly {
        years: Uint128,
    },
    Monthly {
        months: Uint128,
    },
    Daily {
        days: Uint128,
    },
    Hourly {
        hours: Uint128,
    },
    Minutes {
        minutes: Uint128,
    },
    Seconds {
        seconds: Uint128,
    },
}

pub fn parse_utc_datetime(
    last_refresh: &String,
) -> StdResult<DateTime<Utc>> {

    DateTime::parse_from_rfc3339(&last_refresh)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| 
            StdError::generic_err(
                format!("Failed to parse datetime {}", last_refresh)
            )
        )
}
