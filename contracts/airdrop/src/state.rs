use shade_protocol::c_std::{Deps, Uint128};
use shade_protocol::c_std::{
    Api,
    Addr,
    StdResult,
    Storage,
};
use shade_protocol::storage::{
    bucket,
    bucket_read,
    singleton,
    singleton_read,
    Bucket,
    ReadonlyBucket,
    ReadonlySingleton,
    Singleton,
};
use shade_protocol::contract_interfaces::airdrop::{
    account::{
        authenticate_ownership,
        Account,
        AccountPermit,
        AddressProofMsg,
        AddressProofPermit,
    },
    errors::{permit_contract_mismatch, permit_key_revoked},
    Config,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static DECAY_CLAIMED_KEY: &[u8] = b"decay_claimed";
pub static CLAIM_STATUS_KEY: &[u8] = b"claim_status_";
pub static REWARD_IN_ACCOUNT_KEY: &[u8] = b"reward_in_account";
pub static ACCOUNTS_KEY: &[u8] = b"accounts";
pub static TOTAL_CLAIMED_KEY: &[u8] = b"total_claimed";
pub static USER_TOTAL_CLAIMED_KEY: &[u8] = b"user_total_claimed";
pub static ACCOUNT_PERMIT_KEY: &str = "account_permit_key";
pub static ACCOUNT_VIEWING_KEY: &[u8] = b"account_viewing_key";

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn decay_claimed_w(storage: &mut dyn Storage) -> Singleton<bool> {
    singleton(storage, DECAY_CLAIMED_KEY)
}

pub fn decay_claimed_r(storage: &dyn Storage) -> ReadonlySingleton<bool> {
    singleton_read(storage, DECAY_CLAIMED_KEY)
}

// Is address added to an account
pub fn address_in_account_r(storage: &dyn Storage) -> ReadonlyBucket<bool> {
    bucket_read(storage, REWARD_IN_ACCOUNT_KEY)
}

pub fn address_in_account_w(storage: &mut dyn Storage) -> Bucket<bool> {
    bucket(storage, REWARD_IN_ACCOUNT_KEY)
}

// airdrop account
pub fn account_r(storage: &dyn Storage) -> ReadonlyBucket<Account> {
    bucket_read(storage, ACCOUNTS_KEY)
}

pub fn account_w(storage: &mut dyn Storage) -> Bucket<Account> {
    bucket(storage, ACCOUNTS_KEY)
}

// If not found then its unrewarded; if true then claimed
pub fn claim_status_r(storage: &dyn Storage, index: usize) -> ReadonlyBucket<bool> {
    let mut key = CLAIM_STATUS_KEY.to_vec();
    key.push(index as u8);
    bucket_read(storage, &key)
}

pub fn claim_status_w(storage: &mut dyn Storage, index: usize) -> Bucket<bool> {
    let mut key = CLAIM_STATUS_KEY.to_vec();
    key.push(index as u8);
    bucket(storage, &key)
}

// Total claimed
pub fn total_claimed_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, TOTAL_CLAIMED_KEY)
}

pub fn total_claimed_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, TOTAL_CLAIMED_KEY)
}

// Total account claimed
pub fn account_total_claimed_r(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, USER_TOTAL_CLAIMED_KEY)
}

pub fn account_total_claimed_w(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, USER_TOTAL_CLAIMED_KEY)
}

// Account viewing key
pub fn account_viewkey_r(storage: &dyn Storage) -> ReadonlyBucket<[u8; 32]> {
    bucket_read(storage, ACCOUNT_VIEWING_KEY)
}

pub fn account_viewkey_w(storage: &mut dyn Storage) -> Bucket<[u8; 32]> {
    bucket(storage, ACCOUNT_VIEWING_KEY)
}

// Account permit key
pub fn account_permit_key_r(storage: &dyn Storage, account: String) -> ReadonlyBucket<bool> {
    let key = ACCOUNT_PERMIT_KEY.to_string() + &account;
    bucket_read(storage, key.as_bytes())
}

pub fn account_permit_key_w(storage: &mut dyn Storage, account: String) -> Bucket<bool> {
    let key = ACCOUNT_PERMIT_KEY.to_string() + &account;
    bucket(storage, key.as_bytes())
}

pub fn revoke_permit(storage: &mut dyn Storage, account: String, permit_key: String) {
    account_permit_key_w(storage, account)
        .save(permit_key.as_bytes(), &false)
        .unwrap();
}

pub fn is_permit_revoked(
    storage: &dyn Storage,
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

pub fn validate_address_permit(
    storage: &dyn Storage,
    api: &dyn Api,
    permit: &AddressProofPermit,
    params: &AddressProofMsg,
    contract: Addr,
) -> StdResult<()> {
    // Check that contract matches
    if params.contract != contract {
        return Err(permit_contract_mismatch(
            params.contract.as_str(),
            contract.as_str(),
        ));
    }

    // Check that permit is not revoked
    if is_permit_revoked(storage, params.address.to_string(), params.key.clone())? {
        return Err(permit_key_revoked(params.key.as_str()));
    }

    // Authenticate permit
    authenticate_ownership(api, permit, params.address.as_str())
}

pub fn validate_account_permit(
    deps: Deps,
    permit: &AccountPermit,
    contract: Addr,
) -> StdResult<Addr> {
    // Check that contract matches
    if permit.params.contract != contract {
        return Err(permit_contract_mismatch(
            permit.params.contract.as_str(),
            contract.as_str(),
        ));
    }

    // Authenticate permit
    let address = permit.validate(deps.api, None)?.as_addr(None)?;

    // Check that permit is not revoked
    if is_permit_revoked(
        deps.storage,
        address.to_string(),
        permit.params.key.clone(),
    )? {
        return Err(permit_key_revoked(permit.params.key.as_str()));
    }

    return Ok(address);
}
