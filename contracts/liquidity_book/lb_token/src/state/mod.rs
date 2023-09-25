pub mod permissions;
mod save_load_functions;
pub mod txhistory;

use cosmwasm_std::{to_binary, Addr, BlockInfo, StdError, StdResult, Storage, Uint256};

use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, PrefixedStorage, ReadonlyBucket,
    ReadonlyPrefixedStorage, ReadonlySingleton, Singleton,
};

use shade_protocol::lb_libraries::lb_token::{
    permissions::Permission,
    state_structs::{ContractConfig, StoredTokenInfo},
};

pub const RESPONSE_BLOCK_SIZE: usize = 256;

// namespaces
pub const CONTR_CONF: &[u8] = b"contrconfig";
pub const TKN_TOTAL_SUPPLY: &[u8] = b"totalsupply";
pub const BALANCES: &[u8] = b"balances";
pub const TKN_INFO: &[u8] = b"tokeninfo";
/// storage key for the BlockInfo when the last handle was executed
pub const BLOCK_KEY: &[u8] = b"blockinfo";

/// prefix for storage of transactions
pub const PREFIX_TXS: &[u8] = b"preftxs";
/// prefix for storage of tx ids
pub const PREFIX_TX_IDS: &[u8] = b"txids";
/// prefix for NFT ownership history
pub const PREFIX_NFT_OWNER: &[u8] = b"nftowner";
/// prefix for storing permissions
pub const PREFIX_PERMISSIONS: &[u8] = b"permissions";
/// prefix for storing permission identifier (ID) for a given address
pub const PREFIX_PERMISSION_ID: &[u8] = b"permid";
pub const PREFIX_REVOKED_PERMITS: &str = "revokedperms";
pub const PREFIX_RECEIVERS: &[u8] = b"s1155receivers";

/////////////////////////////////////////////////////////////////////////////////
// Singletons
/////////////////////////////////////////////////////////////////////////////////

/// Contract configuration: stores information on this contract
pub fn contr_conf_w(storage: &mut dyn Storage) -> Singleton<ContractConfig> {
    singleton(storage, CONTR_CONF)
}
/// Contract configuration: reads information on this contract
pub fn contr_conf_r(storage: &dyn Storage) -> ReadonlySingleton<ContractConfig> {
    singleton_read(storage, CONTR_CONF)
}

/// Saves BlockInfo of latest tx. Should not be necessary after env becomes available to queries
pub fn blockinfo_w(storage: &mut dyn Storage) -> Singleton<BlockInfo> {
    singleton(storage, BLOCK_KEY)
}
/// Reads BlockInfo of latest tx. Should not be necessary after env becomes available to queries
pub fn blockinfo_r(storage: &dyn Storage) -> ReadonlySingleton<BlockInfo> {
    singleton_read(storage, BLOCK_KEY)
}

/////////////////////////////////////////////////////////////////////////////////
// Buckets
/////////////////////////////////////////////////////////////////////////////////

/// token_id configs. Key is `token_id.as_bytes()`
pub fn tkn_info_w(storage: &mut dyn Storage) -> Bucket<StoredTokenInfo> {
    bucket(storage, TKN_INFO)
}
/// token_id configs. Key is `token_id.as_bytes()`
pub fn tkn_info_r(storage: &dyn Storage) -> ReadonlyBucket<StoredTokenInfo> {
    bucket_read(storage, TKN_INFO)
}

/// total supply of a token_id. Key is `token_id.as_bytes()`
pub fn tkn_tot_supply_w(storage: &mut dyn Storage) -> Bucket<Uint256> {
    bucket(storage, TKN_TOTAL_SUPPLY)
}
/// total supply of a token_id. Key is `token_id.as_bytes()`
pub fn tkn_tot_supply_r(storage: &dyn Storage) -> ReadonlyBucket<Uint256> {
    bucket_read(storage, TKN_TOTAL_SUPPLY)
}

/////////////////////////////////////////////////////////////////////////////////
// Multi-level Buckets
/////////////////////////////////////////////////////////////////////////////////

/// Multilevel bucket to store balances for each token_id & addr combination. Key is to
/// be [`token_id`, `owner`: to_binary(&Addr)?.as_slice()]  
/// When using `balances_w` make sure to also check if need to change `current owner` of an nft and `total_supply`
pub fn balances_w<'a>(storage: &'a mut dyn Storage, token_id: &str) -> Bucket<'a, Uint256> {
    Bucket::multilevel(storage, &[BALANCES, token_id.as_bytes()])
}
/// Multilevel bucket to store balances for each token_id & addr combination. Key is to
/// be [`token_id`, `owner`: to_binary(&Addr)?.as_slice()]  
pub fn balances_r<'a>(storage: &'a dyn Storage, token_id: &str) -> ReadonlyBucket<'a, Uint256> {
    ReadonlyBucket::multilevel(storage, &[BALANCES, token_id.as_bytes()])
}

/// private functions.
/// To store permission. key is to be [`owner`, `token_id`, `allowed_addr`]
/// `allowed_addr` is `to_binary(&Addr)?.as_slice()`
fn permission_w<'a>(
    storage: &'a mut dyn Storage,
    owner: &'a Addr,
    token_id: &'a str,
) -> Bucket<'a, Permission> {
    let owner_bin = to_binary(owner).unwrap();
    Bucket::multilevel(
        storage,
        &[
            PREFIX_PERMISSIONS,
            owner_bin.as_slice(),
            token_id.as_bytes(),
        ],
    )
}
/// private functions.
/// To read permission. key is to be [`owner`, `token_id`, `allowed_addr`]
/// `allowed_addr` is `to_binary(&Addr)?.as_slice()`
fn permission_r<'a>(
    storage: &'a dyn Storage, // &'a (dyn Storage + 'a), // &'a S,
    owner: &'a Addr,
    token_id: &'a str,
) -> ReadonlyBucket<'a, Permission> {
    let owner_bin = to_binary(owner).unwrap();
    ReadonlyBucket::multilevel(
        storage,
        &[
            PREFIX_PERMISSIONS,
            owner_bin.as_slice(),
            token_id.as_bytes(),
        ],
    )
}
#[cfg(test)]
pub fn perm_r<'a>(
    storage: &'a dyn Storage,
    owner: &'a Addr,
    token_id: &'a str,
) -> ReadonlyBucket<'a, Permission> {
    let owner_bin = to_binary(owner).unwrap();
    ReadonlyBucket::multilevel(
        storage,
        &[
            PREFIX_PERMISSIONS,
            owner_bin.as_slice(),
            token_id.as_bytes(),
        ],
    )
}

/////////////////////////////////////////////////////////////////////////////////
// Receiver Interface
/////////////////////////////////////////////////////////////////////////////////

pub fn get_receiver_hash(store: &dyn Storage, account: &Addr) -> Option<StdResult<String>> {
    let store = ReadonlyPrefixedStorage::new(store, PREFIX_RECEIVERS);
    store.get(account.as_str().as_bytes()).map(|data| {
        String::from_utf8(data)
            .map_err(|_err| StdError::invalid_utf8("stored code hash was not a valid String"))
    })
}

pub fn set_receiver_hash(store: &mut dyn Storage, account: &Addr, code_hash: String) {
    let mut store = PrefixedStorage::new(store, PREFIX_RECEIVERS);
    store.set(account.as_str().as_bytes(), code_hash.as_bytes());
}
