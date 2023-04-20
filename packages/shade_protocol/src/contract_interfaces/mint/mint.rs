use crate::{
    contract_interfaces::snip20::helpers::Snip20Asset,
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use crate::c_std::Uint128;
use crate::c_std::{Binary, Addr};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};


#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub oracle: Contract,
    // Both treasury & Commission must be set to function
    pub treasury: Addr,
    pub secondary_burn: Option<Addr>,
    pub activated: bool,
    pub limit: Option<Limit>,
}

/// Used to store the assets allowed to be burned
#[cw_serde]
pub struct SupportedAsset {
    pub asset: Snip20Asset,
    // Capture a percentage of burned assets
    pub capture: Uint128,
    // Fee taken off the top of a given burned asset
    pub fee: Uint128,
    pub unlimited: bool,
}

#[cw_serde]
pub enum Limit {
    Daily {
        supply_portion: Uint128,
        days: Uint128,
    },
    Monthly {
        supply_portion: Uint128,
        months: Uint128,
    },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub oracle: Contract,

    // Asset that is minted
    pub native_asset: Contract,

    //Symbol to peg to, default to snip20 symbol
    pub peg: Option<String>,

    // Both treasury & asset capture must be set to function properly
    pub treasury: Addr,

    // This is where the non-burnable assets will go, if not defined they will stay in this contract
    pub secondary_burn: Option<Addr>,

    pub limit: Option<Limit>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        contract: Contract,
        // Commission * 100 e.g. 5 == .05 == 5%
        capture: Option<Uint128>,
        fee: Option<Uint128>,
        unlimited: Option<bool>,
    },
    RemoveAsset {
        address: Addr,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct SnipMsgHook {
    pub minimum_expected_amount: Uint128,
    pub to_mint: Addr,
}

#[cw_serde]
pub struct MintMsgHook {
    pub minimum_expected_amount: Uint128,
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: ResponseStatus,
        address: Addr,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    RegisterAsset {
        status: ResponseStatus,
    },
    RemoveAsset {
        status: ResponseStatus,
    },
    Mint {
        status: ResponseStatus,
        amount: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    NativeAsset {},
    SupportedAssets {},
    Asset {
        contract: String,
    },
    Config {},
    Limit {},
    Mint {
        offer_asset: Addr,
        amount: Uint128,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    NativeAsset {
        asset: Snip20Asset,
        peg: String,
    },
    SupportedAssets {
        assets: Vec<Contract>,
    },
    Asset {
        asset: SupportedAsset,
        burned: Uint128,
    },
    Config {
        config: Config,
    },
    Limit {
        minted: Uint128,
        limit: Uint128,
        last_refresh: String,
    },
    Mint {
        asset: Contract,
        amount: Uint128,
    },
}
