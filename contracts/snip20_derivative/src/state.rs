use cosmwasm_std::{Addr, StdResult, Storage};
use secret_toolkit::storage::Item;
use secret_toolkit::{serialization::Json, storage::Keymap};

use crate::msg::{Config, ContractStatusLevel, InProcessUnbonding, PanicUnbond};
use crate::staking_interface::{Unbonding, Token};

pub const KEY_CONFIG: &[u8] = b"config";
pub const KEY_STAKING_INFO: &[u8] = b"staking_info";
pub const KEY_PANIC_UNBONDS: &[u8] = b"panic_unbonds_ids";
pub const KEY_PENDING_UNBONDING: &[u8] = b"last_unbonding_id";
pub const KEY_CONTRACT_STATUS: &[u8] = b"contract_status";
pub const PREFIX_UNBONDINGS_IDS: &[u8] = b"unbondings_ids";
pub const PREFIX_UNBONDINGS: &[u8] = b"unbondings";
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const UNBOND_REPLY_ID: u64 = 1_u64;
pub const PANIC_WITHDRAW_REPLY_ID: u64 = 2_u64;
pub const PANIC_UNBOND_REPLY_ID: u64 = 3_u64;
// Circle back rewards storage keys
pub const KEY_REWARDED_TOKENS_LIST: &[u8] = b"rewarded_tokens";
pub const PREFIX_CONTRACTS_VKS: &[u8] = b"contracts_vks";
pub static REWARDED_TOKENS_LIST: Item<Vec<Addr>> = Item::new(KEY_REWARDED_TOKENS_LIST);
pub static CONTRACTS_VKS: Keymap<Addr, Token> = Keymap::new(PREFIX_CONTRACTS_VKS);

pub static CONTRACT_STATUS: Item<ContractStatusLevel, Json> = Item::new(KEY_CONTRACT_STATUS);
pub static CONFIG: Item<Config> = Item::new(KEY_CONFIG);
pub static PANIC_UNBONDS: Item<Vec<PanicUnbond>> = Item::new(KEY_PANIC_UNBONDS);
pub static PENDING_UNBONDING: Item<InProcessUnbonding> = Item::new(KEY_PENDING_UNBONDING);
pub static UNBONDINGS_IDS: Item<Vec<u128>> = Item::new(PREFIX_UNBONDINGS_IDS);
pub static UNBONDING: Keymap<u128, Unbonding> = Keymap::new(PREFIX_UNBONDINGS);

pub struct ContractsVksStore {}
impl ContractsVksStore {
    pub fn may_load(store: &dyn Storage, contract_addr: &Addr) -> Option<Token> {
        CONTRACTS_VKS.get(store, &contract_addr)
    }

    pub fn save(store: &mut dyn Storage, contract_addr: &Addr, token: &Token) -> StdResult<()> {
        CONTRACTS_VKS.insert(store, &contract_addr, token)
    }
}

pub struct UnbondingIdsStore {}
impl UnbondingIdsStore {
    pub fn load(store: &dyn Storage, account: &Addr) -> Vec<u128> {
        let unbondings_ids = UNBONDINGS_IDS.add_suffix(account.as_str().as_bytes());
        unbondings_ids.load(store).unwrap_or_default()
    }

    pub fn save(store: &mut dyn Storage, account: &Addr, ids: Vec<u128>) -> StdResult<()> {
        let unbondings_ids = UNBONDINGS_IDS.add_suffix(account.as_str().as_bytes());
        unbondings_ids.save(store, &ids)
    }
}

pub struct UnbondingStore {}
impl UnbondingStore {
    pub fn may_load(store: &dyn Storage, id: u128) -> Option<Unbonding> {
        UNBONDING.get(store, &id)
    }

    pub fn save(store: &mut dyn Storage, id: u128, unbond: &Unbonding) -> StdResult<()> {
        UNBONDING.insert(store, &id, unbond)
    }

    pub fn remove(store: &mut dyn Storage, id: u128) -> StdResult<()> {
        UNBONDING.remove(store, &id)
    }
}
