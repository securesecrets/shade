use crate::utils::storage::plus::{ItemStorage, MapStorage};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Uint128, Uint256};
use shade_admin::asset::Contract;

// NOTE: we use string for the pools because of the possible case of a staking pool migrating

// TODO: implemet iter
#[cw_serde]
pub struct Profiles {
    pub profiles: Vec<String>,
}
impl ItemStorage for Profiles {
    const ITEM: Item<'static, Self> = Item::new("profiles-");
}

impl Profiles {
    fn get(storage: &dyn Storage, profile: String) -> StdResult<Profile> {
        Profile::load(storage, profile)
    }

    fn remove(storage: &mut dyn Storage, profile: String) -> StdResult<()> {
        // TODO: load profilees and remove one item from array
        // TODO: remove map key
        Ok(())
    }
}

#[cw_serde]
pub struct Profile {
    pub contract: Contract,
    pub unbond_period: u64,
}
impl MapStorage<String> for Profile {
    const MAP: Map<'static, String, Self> = Map::new("profile-");
}

// Derivative
#[cw_serde]
pub struct Derivative {
    pub derivative: Contract,
    // Where the stake gets deposited
    pub target: String,
    // Gets how much gets split for the derivative TODO: get how to calculate a percentage from the team
    pub split: Uint128,
}
impl ItemStorage for Derivative {
    const ITEM: Item<'static, Self> = Item::new("derivative-");
}

#[cw_serde]
pub struct DerivativeSplit {
    pub deriv: Uint256,
    pub normal: Uint256,
}
// TODO: calculate adding a normal and calculating its split based on a percentage, maybe this gets added in the derivative split itself
// TODO: calculate getting total normal from a set amount of deriv

// TODO: TotalDerivativePool

// User pools
#[cw_serde]
pub struct UserDerivativePool {
    pub shares: DerivativeSplit,
}
impl MapStorage<(Addr, String)> for UserDerivativePool {
    const MAP: Map<'static, (Addr, String), Self> = Map::new("user-derivative-pool-");
}
