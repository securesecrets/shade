use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;

use crate::{
    c_std::{Addr, BlockInfo, Coin, StdError, StdResult, Storage, Uint128},
    contract_interfaces::snip20::errors::{
        legacy_cannot_convert_from_tx,
        tx_code_invalid_conversion,
    },
};

#[cfg(feature = "snip20-impl")]
use crate::utils::storage::plus::{ItemStorage, MapStorage};
#[cfg(feature = "snip20-impl")]
use secret_storage_plus::{Item, Map};

// Note that id is a globally incrementing counter.
// Since it's 64 bits long, even at 50 tx/s it would take
// over 11 billion years for it to rollback. I'm pretty sure
// we'll have bigger issues by then.
#[cw_serde]
pub struct Tx {
    pub id: u64,
    pub from: Addr,
    pub sender: Addr,
    pub receiver: Addr,
    pub coins: Coin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    // The block time and block height are optional so that the JSON schema
    // reflects that some SNIP-20 contracts may not include this info.
    pub block_time: Option<Timestamp>,
    pub block_height: Option<u64>,
}

#[cfg(feature = "snip20-impl")]
impl Tx {
    // Inefficient but compliant, not recommended to use deprecated features
    pub fn get(
        storage: &dyn Storage,
        for_address: &Addr,
        page: u32,
        page_size: u32,
    ) -> StdResult<(Vec<Self>, u64)> {
        let id = UserTXTotal::load(storage, for_address.clone())?.0;
        let start_index = page as u64 * page_size as u64;

        // Since we dont know where the legacy txs are then we iterate over everything
        let mut total = 0u64;
        let mut txs = vec![];
        for i in 0..id {
            match StoredRichTx::load(storage, (for_address.clone(), i))?.into_legacy() {
                Ok(tx) => {
                    total += 1;
                    if total >= (start_index + page_size as u64) {
                        break;
                    } else if total >= start_index {
                        txs.push(tx);
                    }
                }
                Err(_) => {}
            }
        }

        let length = txs.len() as u64;
        Ok((txs, length))
    }
}

#[cw_serde]
pub enum TxAction {
    Transfer {
        from: Addr,
        sender: Addr,
        recipient: Addr,
    },
    Mint {
        minter: Addr,
        recipient: Addr,
    },
    Burn {
        burner: Addr,
        owner: Addr,
    },
    Deposit {},
    Redeem {},
}

// Note that id is a globally incrementing counter.
// Since it's 64 bits long, even at 50 tx/s it would take
// over 11 billion years for it to rollback. I'm pretty sure
// we'll have bigger issues by then.
#[cw_serde]
pub struct RichTx {
    pub id: u64,
    pub action: TxAction,
    pub coins: Coin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub block_time: Timestamp,
    pub block_height: u64,
}

#[cfg(feature = "snip20-impl")]
impl RichTx {
    pub fn get(
        storage: &dyn Storage,
        for_address: &Addr,
        page: u32,
        page_size: u32,
    ) -> StdResult<(Vec<Self>, u64)> {
        let id = UserTXTotal::load(storage, for_address.clone())?.0;
        let start_index = page as u64 * page_size as u64;
        let size: u64;
        if (start_index + page_size as u64) > id {
            size = id;
        } else {
            size = page_size as u64 + start_index;
        }

        let mut txs = vec![];
        for index in start_index..size {
            let stored_tx = StoredRichTx::load(storage, (for_address.clone(), index))?;
            txs.push(stored_tx.into_humanized()?);
        }

        let length = txs.len() as u64;
        Ok((txs, length))
    }
}

// Stored types:
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum TxCode {
    Transfer = 0,
    Mint = 1,
    Burn = 2,
    Deposit = 3,
    Redeem = 4,
}

impl TxCode {
    fn to_u8(self) -> u8 {
        self as u8
    }

    fn from_u8(n: u8) -> StdResult<Self> {
        use TxCode::*;
        match n {
            0 => Ok(Transfer),
            1 => Ok(Mint),
            2 => Ok(Burn),
            3 => Ok(Deposit),
            4 => Ok(Redeem),
            _ => Err(tx_code_invalid_conversion(n)),
        }
    }
}

#[cw_serde]
struct StoredTxAction {
    tx_type: u8,
    address1: Option<Addr>,
    address2: Option<Addr>,
    address3: Option<Addr>,
}

impl StoredTxAction {
    fn transfer(from: Addr, sender: Addr, recipient: Addr) -> Self {
        Self {
            tx_type: TxCode::Transfer.to_u8(),
            address1: Some(from),
            address2: Some(sender),
            address3: Some(recipient),
        }
    }

    fn mint(minter: Addr, recipient: Addr) -> Self {
        Self {
            tx_type: TxCode::Mint.to_u8(),
            address1: Some(minter),
            address2: Some(recipient),
            address3: None,
        }
    }

    fn burn(owner: Addr, burner: Addr) -> Self {
        Self {
            tx_type: TxCode::Burn.to_u8(),
            address1: Some(burner),
            address2: Some(owner),
            address3: None,
        }
    }

    fn deposit() -> Self {
        Self {
            tx_type: TxCode::Deposit.to_u8(),
            address1: None,
            address2: None,
            address3: None,
        }
    }

    fn redeem() -> Self {
        Self {
            tx_type: TxCode::Redeem.to_u8(),
            address1: None,
            address2: None,
            address3: None,
        }
    }

    fn into_humanized(self) -> StdResult<TxAction> {
        let transfer_addr_err = || {
            StdError::generic_err(
                "Missing address in stored Transfer transaction. Storage is corrupt",
            )
        };
        let mint_addr_err = || {
            StdError::generic_err("Missing address in stored Mint transaction. Storage is corrupt")
        };
        let burn_addr_err = || {
            StdError::generic_err("Missing address in stored Burn transaction. Storage is corrupt")
        };

        // In all of these, we ignore fields that we don't expect to find populated
        let action = match TxCode::from_u8(self.tx_type)? {
            TxCode::Transfer => {
                let from = self.address1.ok_or_else(transfer_addr_err)?;
                let sender = self.address2.ok_or_else(transfer_addr_err)?;
                let recipient = self.address3.ok_or_else(transfer_addr_err)?;
                TxAction::Transfer {
                    from,
                    sender,
                    recipient,
                }
            }
            TxCode::Mint => {
                let minter = self.address1.ok_or_else(mint_addr_err)?;
                let recipient = self.address2.ok_or_else(mint_addr_err)?;
                TxAction::Mint { minter, recipient }
            }
            TxCode::Burn => {
                let burner = self.address1.ok_or_else(burn_addr_err)?;
                let owner = self.address2.ok_or_else(burn_addr_err)?;
                TxAction::Burn { burner, owner }
            }
            TxCode::Deposit => TxAction::Deposit {},
            TxCode::Redeem => TxAction::Redeem {},
        };

        Ok(action)
    }
}

#[cw_serde]
struct StoredRichTx {
    id: u64,
    action: StoredTxAction,
    coins: Coin,
    memo: Option<String>,
    block_time: Timestamp,
    block_height: u64,
}

impl StoredRichTx {
    fn new(
        id: u64,
        action: StoredTxAction,
        coins: Coin,
        memo: Option<String>,
        block: &BlockInfo,
    ) -> Self {
        Self {
            id,
            action,
            coins,
            memo,
            block_time: block.time,
            block_height: block.height,
        }
    }

    fn into_humanized(self) -> StdResult<RichTx> {
        Ok(RichTx {
            id: self.id,
            action: self.action.into_humanized()?,
            coins: self.coins,
            memo: self.memo,
            block_time: self.block_time,
            block_height: self.block_height,
        })
    }

    fn into_legacy(self) -> StdResult<Tx> {
        if self.action.tx_type == 0 {
            Ok(Tx {
                id: self.id,
                from: self.action.address1.unwrap(),
                sender: self.action.address2.unwrap(),
                receiver: self.action.address3.unwrap(),
                coins: self.coins,
                memo: self.memo,
                block_time: Some(self.block_time),
                block_height: Some(self.block_height),
            })
        } else {
            Err(legacy_cannot_convert_from_tx())
        }
    }
}

#[cfg(feature = "snip20-impl")]
impl MapStorage<'static, (Addr, u64)> for StoredRichTx {
    const MAP: Map<'static, (Addr, u64), Self> = Map::new("stored-rich-tx-");
}

// Storage functions:
#[cw_serde]
struct TXCount(pub u64);

#[cfg(feature = "snip20-impl")]
impl ItemStorage for TXCount {
    const ITEM: Item<'static, Self> = Item::new("tx-count-");
}

#[cfg(feature = "snip20-impl")]
fn increment_tx_count(storage: &mut dyn Storage) -> StdResult<u64> {
    let id = TXCount::may_load(storage)?.unwrap_or(TXCount(0)).0 + 1;
    TXCount(id).save(storage)?;
    Ok(id)
}

// User tx index
#[cw_serde]
struct UserTXTotal(pub u64);

#[cfg(feature = "snip20-impl")]
impl UserTXTotal {
    pub fn append(
        storage: &mut dyn Storage,
        for_address: &Addr,
        tx: &StoredRichTx,
    ) -> StdResult<()> {
        let id = UserTXTotal::may_load(storage, for_address.clone())?
            .unwrap_or(UserTXTotal(0))
            .0;
        UserTXTotal(id + 1).save(storage, for_address.clone())?;
        tx.save(storage, (for_address.clone(), id))?;

        Ok(())
    }
}

#[cfg(feature = "snip20-impl")]
impl MapStorage<'static, Addr> for UserTXTotal {
    const MAP: Map<'static, Addr, Self> = Map::new("user-tx-total-");
}

#[cfg(feature = "snip20-impl")]
#[allow(clippy::too_many_arguments)] // We just need them
pub fn store_transfer(
    storage: &mut dyn Storage,
    owner: &Addr,
    sender: &Addr,
    receiver: &Addr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(storage)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let tx = StoredRichTx::new(
        id,
        StoredTxAction::transfer(owner.clone(), sender.clone(), receiver.clone()),
        coins,
        memo,
        block,
    );

    // Write to the owners history if it's different from the other two addresses
    if owner != sender && owner != receiver {
        // crate::c_std::debug_print("saving transaction history for owner");
        UserTXTotal::append(storage, owner, &tx)?;
    }
    // Write to the sender's history if it's different from the receiver
    if sender != receiver {
        // crate::c_std::debug_print("saving transaction history for sender");
        UserTXTotal::append(storage, sender, &tx)?;
    }
    // Always write to the recipient's history
    // crate::c_std::debug_print("saving transaction history for receiver");
    UserTXTotal::append(storage, receiver, &tx)?;

    Ok(())
}

#[cfg(feature = "snip20-impl")]
pub fn store_mint(
    storage: &mut dyn Storage,
    minter: &Addr,
    recipient: &Addr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(storage)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::mint(minter.clone(), recipient.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    if minter != recipient {
        UserTXTotal::append(storage, recipient, &tx)?;
    }
    UserTXTotal::append(storage, minter, &tx)?;

    Ok(())
}

#[cfg(feature = "snip20-impl")]
pub fn store_burn(
    storage: &mut dyn Storage,
    owner: &Addr,
    burner: &Addr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(storage)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::burn(owner.clone(), burner.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    if burner != owner {
        UserTXTotal::append(storage, owner, &tx)?;
    }
    UserTXTotal::append(storage, burner, &tx)?;

    Ok(())
}

#[cfg(feature = "snip20-impl")]
pub fn store_deposit(
    storage: &mut dyn Storage,
    recipient: &Addr,
    amount: Uint128,
    denom: String,
    block: &BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(storage)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::deposit();
    let tx = StoredRichTx::new(id, action, coins, None, block);

    UserTXTotal::append(storage, recipient, &tx)?;

    Ok(())
}

#[cfg(feature = "snip20-impl")]
pub fn store_redeem(
    storage: &mut dyn Storage,
    redeemer: &Addr,
    amount: Uint128,
    denom: String,
    block: &BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(storage)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::redeem();
    let tx = StoredRichTx::new(id, action, coins, None, block);

    UserTXTotal::append(storage, redeemer, &tx)?;

    Ok(())
}
