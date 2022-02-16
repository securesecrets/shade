use cosmwasm_std::{
    from_binary, Api, Binary, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::airdrop::account::AddressProofMsg;
use shade_protocol::airdrop::{
    account::{authenticate_ownership, Account, AccountPermit, AddressProofPermit},
    Config,
};
use shade_protocol::airdrop::errors::{permit_contract_mismatch, permit_key_revoked};

pub static CONFIG_KEY: &[u8] = b"config";
pub static DECAY_CLAIMED_KEY: &[u8] = b"decay_claimed";
pub static CLAIM_STATUS_KEY: &[u8] = b"claim_status_";
pub static REWARD_IN_ACCOUNT_KEY: &[u8] = b"reward_in_account";
pub static ACCOUNTS_KEY: &[u8] = b"accounts";
pub static TOTAL_CLAIMED_KEY: &[u8] = b"total_claimed";
pub static USER_TOTAL_CLAIMED_KEY: &[u8] = b"user_total_claimed";
pub static ACCOUNT_PERMIT_KEY: &str = "account_permit_key";
pub static ACCOUNT_VIEWING_KEY: &[u8] = b"account_viewing_key";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn decay_claimed_w<S: Storage>(storage: &mut S) -> Singleton<S, bool> {
    singleton(storage, DECAY_CLAIMED_KEY)
}

pub fn decay_claimed_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, bool> {
    singleton_read(storage, DECAY_CLAIMED_KEY)
}

// Is address added to an account
pub fn address_in_account_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, bool> {
    bucket_read(REWARD_IN_ACCOUNT_KEY, storage)
}

pub fn address_in_account_w<S: Storage>(storage: &mut S) -> Bucket<S, bool> {
    bucket(REWARD_IN_ACCOUNT_KEY, storage)
}

// airdrop account
pub fn account_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Account> {
    bucket_read(ACCOUNTS_KEY, storage)
}

pub fn account_w<S: Storage>(storage: &mut S) -> Bucket<S, Account> {
    bucket(ACCOUNTS_KEY, storage)
}

// If not found then its unrewarded; if true then claimed
pub fn claim_status_r<S: Storage>(storage: &S, index: usize) -> ReadonlyBucket<S, bool> {
    let mut key = CLAIM_STATUS_KEY.to_vec();
    key.push(index as u8);
    bucket_read(&key, storage)
}

pub fn claim_status_w<S: Storage>(storage: &mut S, index: usize) -> Bucket<S, bool> {
    let mut key = CLAIM_STATUS_KEY.to_vec();
    key.push(index as u8);
    bucket(&key, storage)
}

// Total claimed
pub fn total_claimed_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, TOTAL_CLAIMED_KEY)
}

pub fn total_claimed_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, TOTAL_CLAIMED_KEY)
}

// Total account claimed
pub fn account_total_claimed_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(USER_TOTAL_CLAIMED_KEY, storage)
}

pub fn account_total_claimed_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(USER_TOTAL_CLAIMED_KEY, storage)
}

// Account viewing key
pub fn account_viewkey_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, [u8; 32]> {
    bucket_read(ACCOUNT_VIEWING_KEY, storage)
}

pub fn account_viewkey_w<S: Storage>(storage: &mut S) -> Bucket<S, [u8; 32]> {
    bucket(ACCOUNT_VIEWING_KEY, storage)
}

// Account permit key
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

pub fn validate_address_permit<S: Storage>(
    storage: &S,
    permit: &AddressProofPermit,
    params: &AddressProofMsg,
    contract: HumanAddr,
) -> StdResult<()> {

    // Check that contract matches
    if params.contract != contract {
        return Err(permit_contract_mismatch(params.contract.as_str(), contract.as_str()));
    }

    // Check that permit is not revoked
    if is_permit_revoked(storage, params.address.to_string(), params.key.clone())? {
        return Err(permit_key_revoked(params.key.as_str()));
    }

    // Authenticate permit
    authenticate_ownership(permit, params.address.as_str())
}

pub fn validate_account_permit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: &AccountPermit,
    contract: HumanAddr,
) -> StdResult<HumanAddr> {
    // Check that contract matches
    if permit.params.contract != contract {
        return Err(permit_contract_mismatch(permit.params.contract.as_str(), contract.as_str()));
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
