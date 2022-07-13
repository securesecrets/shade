use crate::utils::asset::Contract;
use crate::c_std::{Deps, StdResult};
use crate::c_std::Uint128;

use crate::utils::{InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitMsg {}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct ReferenceData {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

pub fn reference_data(
    deps: Deps,
    base_symbol: String,
    quote_symbol: String,
    band: Contract,
) -> StdResult<ReferenceData> {
    BandQuery::GetReferenceData {
        base_symbol,
        quote_symbol,
    }
    .query(&deps.querier, band.code_hash, band.address)
}

pub fn reference_data_bulk(
    deps: Deps,
    base_symbols: Vec<String>,
    quote_symbols: Vec<String>,
    band: Contract,
) -> StdResult<Vec<ReferenceData>> {
    BandQuery::GetReferenceDataBulk {
        base_symbols,
        quote_symbols,
    }
    .query(&deps.querier, band.code_hash, band.address)
}
