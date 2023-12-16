use super::*;

use cosmwasm_std::{Addr, Api, BlockInfo, CanonicalAddr, StdResult, Storage, Uint256};

use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

use shade_protocol::{
    lb_libraries::lb_token::txhistory::{StoredTx, StoredTxAction, Tx},
    s_toolkit::storage::AppendStore,
};

use crate::state::save_load_functions::{json_load, json_save};

pub static TX_ID_STORE: AppendStore<u64> = AppendStore::new(PREFIX_TX_IDS);
pub static NFT_OWNER_STORE: AppendStore<Addr> = AppendStore::new(PREFIX_NFT_OWNER);

/////////////////////////////////////////////////////////////////////////////////
// Transaction history
/////////////////////////////////////////////////////////////////////////////////

/// Returns StdResult<(Vec<Tx>, u64)> of the txs to display and the total count of txs
///
/// # Arguments
///
/// * `api` - a reference to the Api used to convert human and canonical addresses
/// * `storage` - a reference to the contract's storage
/// * `address` - a reference to the address whose txs to display
/// * `page` - page to start displaying
/// * `page_size` - number of txs per page
pub fn get_txs(
    api: &dyn Api,
    storage: &dyn Storage,
    address: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<(Vec<Tx>, u64)> {
    let addr_store = TX_ID_STORE.add_suffix(address.as_slice());

    let count = addr_store.get_len(storage)? as u64;
    // access tx storage
    let tx_store = ReadonlyPrefixedStorage::new(storage, PREFIX_TXS);
    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let txs: StdResult<Vec<Tx>> = addr_store
        .iter(storage)?
        .rev()
        .skip((page * page_size) as usize)
        .take(page_size as usize)
        .map(|id| {
            id.map(|id| {
                json_load(&tx_store, &id.to_le_bytes())
                    .and_then(|tx: StoredTx| tx.into_humanized(api))
            })
            .and_then(|x| x)
        })
        .collect();

    txs.map(|t| (t, count))
}

#[allow(clippy::too_many_arguments)]
pub fn store_transfer(
    storage: &mut dyn Storage,
    config: &mut ContractConfig,
    block: &BlockInfo,
    token_id: &str,
    from: CanonicalAddr,
    sender: Option<CanonicalAddr>,
    recipient: CanonicalAddr,
    amount: Uint256,
    memo: Option<String>,
) -> StdResult<()> {
    let action = StoredTxAction::Transfer {
        from,
        sender,
        recipient,
        amount,
    };
    let tx = StoredTx {
        tx_id: config.tx_cnt,
        block_height: block.height,
        block_time: block.time.seconds(),
        token_id: token_id.to_string(),
        action,
        memo,
    };
    let mut tx_store = PrefixedStorage::new(storage, PREFIX_TXS);
    json_save(&mut tx_store, &config.tx_cnt.to_le_bytes(), &tx)?;
    if let StoredTxAction::Transfer {
        from,
        sender,
        recipient,
        amount: _,
    } = tx.action
    {
        append_tx_for_addr(storage, config.tx_cnt, &from)?;
        append_tx_for_addr(storage, config.tx_cnt, &recipient)?;
        if let Some(sndr) = sender.as_ref() {
            if *sndr != recipient {
                append_tx_for_addr(storage, config.tx_cnt, sndr)?;
            }
        }
    }
    config.tx_cnt += 1;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn store_mint(
    storage: &mut dyn Storage,
    config: &mut ContractConfig,
    block: &BlockInfo,
    token_id: &str,
    minter: CanonicalAddr,
    recipient: CanonicalAddr,
    amount: Uint256,
    memo: Option<String>,
) -> StdResult<()> {
    let action = StoredTxAction::Mint {
        minter,
        recipient,
        amount,
    };
    let tx = StoredTx {
        tx_id: config.tx_cnt,
        block_height: block.height,
        block_time: block.time.seconds(),
        token_id: token_id.to_string(),
        action,
        memo,
    };
    let mut tx_store = PrefixedStorage::new(storage, PREFIX_TXS);
    json_save(&mut tx_store, &config.tx_cnt.to_le_bytes(), &tx)?;
    if let StoredTxAction::Mint {
        minter,
        recipient,
        amount: _,
    } = tx.action
    {
        append_tx_for_addr(storage, config.tx_cnt, &recipient)?;
        if recipient != minter {
            append_tx_for_addr(storage, config.tx_cnt, &minter)?;
        }
    }
    config.tx_cnt += 1;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn store_burn(
    storage: &mut dyn Storage,
    config: &mut ContractConfig,
    block: &BlockInfo,
    token_id: &str,
    burner: Option<CanonicalAddr>,
    owner: CanonicalAddr,
    amount: Uint256,
    memo: Option<String>,
) -> StdResult<()> {
    let action = StoredTxAction::Burn {
        burner,
        owner,
        amount,
    };
    let tx = StoredTx {
        tx_id: config.tx_cnt,
        block_height: block.height,
        block_time: block.time.seconds(),
        token_id: token_id.to_string(),
        action,
        memo,
    };
    let mut tx_store = PrefixedStorage::new(storage, PREFIX_TXS);
    json_save(&mut tx_store, &config.tx_cnt.to_le_bytes(), &tx)?;
    if let StoredTxAction::Burn {
        burner,
        owner,
        amount: _,
    } = tx.action
    {
        append_tx_for_addr(storage, config.tx_cnt, &owner)?;
        if let Some(bnr) = burner.as_ref() {
            if bnr != &owner {
                append_tx_for_addr(storage, config.tx_cnt, bnr)?;
            }
        }
    }
    config.tx_cnt += 1;
    Ok(())
}

/// Returns StdResult<()> after saving tx id
///
/// # Arguments
///
/// * `storage` - a mutable reference to the storage this item should go to
/// * `tx_id` - the tx id to store
/// * `address` - a reference to the address for which to store this tx id
fn append_tx_for_addr(
    storage: &mut dyn Storage,
    tx_id: u64,
    address: &CanonicalAddr,
) -> StdResult<()> {
    let addr_store = TX_ID_STORE.add_suffix(address.as_slice());
    addr_store.push(storage, &tx_id)
}

/////////////////////////////////////////////////////////////////////////////////
// Token transfer history (for NFTs only)
/////////////////////////////////////////////////////////////////////////////////

/// stores ownership history for a given token_id. Meant to be used for nfts.
/// In base specification, only the latest (ie: current) owner is relevant. But  
/// this design pattern is used to allow viewing a token_id's ownership history,
/// which is allowed in the additional specifications
pub fn append_new_owner(
    storage: &mut dyn Storage,
    token_id: &str,
    address: &Addr,
) -> StdResult<()> {
    let token_id_store = NFT_OWNER_STORE.add_suffix(token_id.as_bytes());
    token_id_store.push(storage, address)
}

pub fn may_get_current_owner(storage: &dyn Storage, token_id: &str) -> StdResult<Option<Addr>> {
    let token_id_store = NFT_OWNER_STORE.add_suffix(token_id.as_bytes());

    let len = token_id_store.get_len(storage)?;
    match len {
        0 => Ok(None),
        x if x > 0 => {
            let pos = token_id_store.get_len(storage)?.saturating_sub(1);
            let current_owner = token_id_store.get_at(storage, pos)?;
            Ok(Some(current_owner))
        }
        _ => unreachable!(),
    }
}
