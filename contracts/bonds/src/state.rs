use cosmwasm_std::{Storage, Api, Querier, Uint128, HumanAddr, Extern, StdResult};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::{contract_interfaces::{
    bonds::{Config, Account, BondOpportunity, AccountPermit, AddressProofPermit,
            errors::{permit_contract_mismatch, permit_key_revoked}},
    snip20::Snip20Asset},
    utils::asset::Contract
};

pub static CONFIG: &[u8] = b"config";
pub static GLOBAL_TOTAL_ISSUED: &[u8] = b"global_total_issued";
pub static GLOBAL_TOTAL_CLAIMED: &[u8] = b"global_total_claimed";
pub static COLLATERAL_ASSETS: &[u8] = b"collateral_assets";
pub static ISSUED_ASSET: &[u8] = b"issued_asset";
pub static ACCOUNTS_KEY: &[u8] = b"accounts";
pub static BOND_OPPORTUNITIES: &[u8] = b"bond_opportunities";
pub static ACCOUNT_VIEWING_KEY: &[u8] = b"account_viewing_key";
pub static ALLOCATED_ALLOWANCE: &[u8] = b"allocated_allowance";
pub static ALLOWANCE_VIEWING_KEY: &[u8] = b"allowance_viewing_key";
pub static ACCOUNT_PERMIT_KEY: &str = "account_permit_key";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG)
}

/* Global amount issued since last issuance reset */
pub fn global_total_issued_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, GLOBAL_TOTAL_ISSUED)
}

pub fn global_total_issued_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, GLOBAL_TOTAL_ISSUED)
}

/* Global amount claimed since last issuance reset */
pub fn global_total_claimed_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, GLOBAL_TOTAL_CLAIMED)
}

pub fn global_total_claimed_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, GLOBAL_TOTAL_CLAIMED)
}

/* List of assets that have bond opportunities stored */
pub fn collateral_assets_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<HumanAddr>> {
    singleton(storage, COLLATERAL_ASSETS)
}

pub fn collateral_assets_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<HumanAddr>> {
    singleton_read(storage, COLLATERAL_ASSETS)
}

/* Asset minted when user claims after bonding period */
pub fn issued_asset_w<S: Storage>(storage: &mut S) -> Singleton<S, Snip20Asset> {
    singleton(storage, ISSUED_ASSET)
}

pub fn issued_asset_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Snip20Asset> {
    singleton_read(storage, ISSUED_ASSET)
}

// Bond account 
pub fn account_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Account> {
    bucket_read(ACCOUNTS_KEY, storage)
}

pub fn account_w<S: Storage>(storage: &mut S) -> Bucket<S, Account> {
    bucket(ACCOUNTS_KEY, storage)
}

// Account viewing key
pub fn account_viewkey_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, [u8; 32]> {
    bucket_read(ACCOUNT_VIEWING_KEY, storage)
}

pub fn account_viewkey_w<S: Storage>(storage: &mut S) -> Bucket<S, [u8; 32]> {
    bucket(ACCOUNT_VIEWING_KEY, storage)
}

pub fn bond_opportunity_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, BondOpportunity> {
    bucket_read(BOND_OPPORTUNITIES, storage)
}

pub fn bond_opportunity_w<S: Storage>(storage: &mut S) -> Bucket<S, BondOpportunity> {
    bucket(BOND_OPPORTUNITIES, storage)
}

// The amount of allowance already allocated/unclaimed from opportunities
pub fn allocated_allowance_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, ALLOCATED_ALLOWANCE)
}

pub fn allocated_allowance_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, ALLOCATED_ALLOWANCE)
}

// Stores the bond contracts viewing key to see its own allowance
pub fn allowance_key_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, ALLOWANCE_VIEWING_KEY)
}

pub fn allowance_key_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, ALLOWANCE_VIEWING_KEY)
}

pub fn account_permit_key_r<S: Storage>(storage: &S, account: String) -> ReadonlyBucket<S, bool> {
    let key = ACCOUNT_PERMIT_KEY.to_string() + &account;
    bucket_read(key.as_bytes(), storage)
}

pub fn account_permit_key_w<S: Storage>(storage: &mut S, account: String) -> Bucket<S, bool> {
    let key = ACCOUNT_PERMIT_KEY.to_string() + &account;
    bucket(key.as_bytes(), storage)
}

pub fn revoke_permit<S: Storage>(storage: &mut S, account: String, permit_key: String) {
    account_permit_key_w(storage, account)
        .save(permit_key.as_bytes(), &false)
        .unwrap();
}

pub fn is_permit_revoked<S: Storage>(
    storage: &S,
    account: String,
    permit_key: String,
) -> StdResult<bool> {
    if account_permit_key_r(storage, account)
        .may_load(permit_key.as_bytes())?
        .is_some()
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn validate_account_permit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: &AccountPermit,
    contract: HumanAddr,
) -> StdResult<HumanAddr> {
    // Check that contract matches
    if permit.params.contract != contract {
        return Err(permit_contract_mismatch(
            permit.params.contract.as_str(),
            contract.as_str(),
        ));
    }

    // Authenticate permit
    let address = permit.validate(None)?.as_humanaddr(&deps.api)?;

    // Check that permit is not revoked
    if is_permit_revoked(
        &deps.storage,
        address.to_string(),
        permit.params.key.clone(),
    )? {
        return Err(permit_key_revoked(permit.params.key.as_str()));
    }

    return Ok(address);
}