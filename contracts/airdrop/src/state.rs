use cosmwasm_std::{Storage, Uint128, StdResult, HumanAddr, StdError};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::airdrop::{Config, claim_info::Reward, account::Account};
use shade_protocol::airdrop::account::AddressProofPermit;

pub static CONFIG_KEY: &[u8] = b"config";
pub static CONTRACT_KEY: &[u8] = b"contract";
pub static REWARDS_KEY: &[u8] = b"rewards";
pub static REWARD_IN_ACCOUNT_KEY: &[u8] = b"reward_in_account";
pub static ACCOUNTS_KEY: &[u8] = b"accounts";
pub static TOTAL_CLAIMED_KEY: &[u8] = b"total_claimed";
pub static USER_TOTAL_CLAIMED_KEY: &[u8] = b"user_total_claimed";
pub static ACCOUNT_PERMIT_KEY: &str = "account_permit_key";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

// Airdrop eligible address
pub fn airdrop_address_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Reward> {
    bucket_read(REWARDS_KEY, storage)
}

pub fn airdrop_address_w<S: Storage>(storage: &mut S) -> Bucket<S, Reward> {
    bucket(REWARDS_KEY, storage)
}

// Is address added to an account
pub fn address_in_account_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, bool> {
    bucket_read(REWARD_IN_ACCOUNT_KEY, storage)
}

pub fn address_in_account_w<S: Storage>(storage: &mut S) -> Bucket<S, bool> {
    bucket(REWARD_IN_ACCOUNT_KEY, storage)
}

// airdrop account
pub fn account_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Account> {
    bucket_read(ACCOUNTS_KEY, storage)
}

pub fn account_w<S: Storage>(storage: &mut S) -> Bucket<S, Account> {
    bucket(ACCOUNTS_KEY, storage)
}

// If not found then its unrewarded; if true then claimed
pub fn claim_status_r<S: Storage>(storage: & S, index: usize) -> ReadonlyBucket<S, bool> {
    // TODO: add prefix thing
    bucket_read(&[index as u8], storage)
}

pub fn claim_status_w<S: Storage>(storage: &mut S, index: usize) -> Bucket<S, bool> {
    bucket(&[index as u8], storage)
}

// Total claimed
pub fn total_claimed_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, TOTAL_CLAIMED_KEY)
}

pub fn total_claimed_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, TOTAL_CLAIMED_KEY)
}

// Total account claimed
pub fn account_total_claimed_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(USER_TOTAL_CLAIMED_KEY, storage)
}

pub fn account_total_claimed_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(USER_TOTAL_CLAIMED_KEY, storage)
}

// Contract's address, used for permits when querying
pub fn contract_w<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, CONTRACT_KEY)
}

pub fn contract_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, CONTRACT_KEY)
}

// Account permit key
pub fn account_permit_key_r<S: Storage>(storage: & S, account: String) -> ReadonlyBucket<S, bool> {
    let key = ACCOUNT_PERMIT_KEY.to_string() + &*account;
    bucket_read(key.as_bytes(), storage)
}

pub fn account_permit_key_w<S: Storage>(storage: &mut S, account: String) -> Bucket<S, bool> {
    let key = ACCOUNT_PERMIT_KEY.to_string() + &*account;
    bucket(key.as_bytes(), storage)
}

pub fn revoke_permit<S: Storage>(storage: &mut S, account: String, permit_key: String) {
    account_permit_key_w(storage, account).save(permit_key.as_bytes(), &false).unwrap();
}

pub fn is_permit_revoked<S: Storage>(storage: &S, account: String, permit_key: String) -> StdResult<bool> {
    if account_permit_key_r(storage, account).may_load(permit_key.as_bytes())?.is_some() {
        Ok(true)
    }
    else {
        Ok(false)
    }
}

pub fn validate_permit<S: Storage>(storage: &S, permit: &AddressProofPermit) -> StdResult<HumanAddr> {
    // Check that contract matches
    if permit.params.contract != contract_r(storage).load()? {
        return Err(StdError::unauthorized())
    }

    // Check that permit is not revoked
    if is_permit_revoked(storage, permit.params.address.to_string(),
                         permit.params.key.clone())? {
        return Err(StdError::generic_err("permit key revoked"))
    }

    // Authenticate permit
    permit.authenticate()
}