use crate::utils::asset::Contract;
use crate::c_std::{Deps, StdResult};
use crate::c_std::Uint128;

use crate::utils::{InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};

#[cw_serde]
pub struct InstantiateMsg {}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum BandQuery {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
}

impl Query for BandQuery {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct ReferenceData {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

pub fn reference_data(
    deps: &Deps,
    base_symbol: String,
    quote_symbol: String,
    band: Contract,
) -> StdResult<ReferenceData> {
    BandQuery::GetReferenceData {
        base_symbol,
        quote_symbol,
    }
    .query(&deps.querier, &band)
}

pub fn reference_data_bulk(
    deps: &Deps,
    base_symbols: Vec<String>,
    quote_symbols: Vec<String>,
    band: Contract,
) -> StdResult<Vec<ReferenceData>> {
    BandQuery::GetReferenceDataBulk {
        base_symbols,
        quote_symbols,
    }
    .query(&deps.querier, &band)
}
