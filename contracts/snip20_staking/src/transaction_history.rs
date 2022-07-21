
use shade_protocol::cosmwasm_schema::cw_serde;

use shade_protocol::c_std::{
    Api,
    CanonicalAddr,
    Coin,
    Addr,
    ReadonlyStorage,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::storage::{PrefixedStorage, ReadonlyPrefixedStorage};

use shade_protocol::c_std::Uint128;
use shade_protocol::secret_toolkit::storage::{AppendStore, AppendStoreMut};

use crate::state::Config;

const PREFIX_TXS: &[u8] = b"transactions";
const PREFIX_TRANSFERS: &[u8] = b"transfers";

// Note that id is a globally incrementing counter.
// Since it's 64 bits long, even at 50 tx/s it would take
// over 11 billion years for it to rollback. I'm pretty sure
// we'll have bigger issues by then.
#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub block_time: Option<u64>,
    pub block_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
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
    Stake {
        staker: Addr,
    },
    AddReward {
        funder: Addr,
    },
    FundUnbond {
        funder: Addr,
    },
    Unbond {
        staker: Addr,
    },
    ClaimUnbond {
        staker: Addr,
    },
    ClaimReward {
        staker: Addr,
    },
}

// Note that id is a globally incrementing counter.
// Since it's 64 bits long, even at 50 tx/s it would take
// over 11 billion years for it to rollback. I'm pretty sure
// we'll have bigger issues by then.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct RichTx {
    pub id: u64,
    pub action: TxAction,
    pub coins: Coin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub block_time: u64,
    pub block_height: u64,
}

// Stored types:

/// This type is the stored version of the legacy transfers
#[cw_serde]
struct StoredLegacyTransfer {
    id: u64,
    from: CanonicalAddr,
    sender: CanonicalAddr,
    receiver: CanonicalAddr,
    coins: Coin,
    memo: Option<String>,
    block_time: u64,
    block_height: u64,
}

impl StoredLegacyTransfer {
    pub fn into_humanized(self, api: &dyn Api) -> StdResult<Tx> {
        let tx = Tx {
            id: self.id,
            from: api.human_address(&self.from)?,
            sender: api.human_address(&self.sender)?,
            receiver: api.human_address(&self.receiver)?,
            coins: self.coins,
            memo: self.memo,
            block_time: Some(self.block_time),
            block_height: Some(self.block_height),
        };
        Ok(tx)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum TxCode {
    Transfer = 0,
    Mint = 1,
    Burn = 2,
    Deposit = 3,
    Redeem = 4,
    Stake = 5,
    AddReward = 6,
    FundUnbond = 7,
    Unbond = 8,
    ClaimUnbond = 9,
    ClaimReward = 10,
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
            5 => Ok(Stake),
            6 => Ok(AddReward),
            7 => Ok(FundUnbond),
            8 => Ok(Unbond),
            9 => Ok(ClaimUnbond),
            10 => Ok(ClaimReward),
            other => Err(StdError::generic_err(format!(
                "Unexpected Tx code in transaction history: {} Storage is corrupted.",
                other
            ))),
        }
    }
}

#[cw_serde]
struct StoredTxAction {
    tx_type: u8,
    address1: Option<CanonicalAddr>,
    address2: Option<CanonicalAddr>,
    address3: Option<CanonicalAddr>,
}

impl StoredTxAction {
    fn transfer(from: CanonicalAddr, sender: CanonicalAddr, recipient: CanonicalAddr) -> Self {
        Self {
            tx_type: TxCode::Transfer.to_u8(),
            address1: Some(from),
            address2: Some(sender),
            address3: Some(recipient),
        }
    }

    fn mint(minter: CanonicalAddr, recipient: CanonicalAddr) -> Self {
        Self {
            tx_type: TxCode::Mint.to_u8(),
            address1: Some(minter),
            address2: Some(recipient),
            address3: None,
        }
    }

    fn burn(owner: CanonicalAddr, burner: CanonicalAddr) -> Self {
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

    fn stake(staker: CanonicalAddr) -> Self {
        Self {
            tx_type: TxCode::Stake.to_u8(),
            address1: Some(staker),
            address2: None,
            address3: None,
        }
    }

    fn add_reward(funder: CanonicalAddr) -> Self {
        Self {
            tx_type: TxCode::AddReward.to_u8(),
            address1: Some(funder),
            address2: None,
            address3: None,
        }
    }

    fn fund_unbond(funder: CanonicalAddr) -> Self {
        Self {
            tx_type: TxCode::FundUnbond.to_u8(),
            address1: Some(funder),
            address2: None,
            address3: None,
        }
    }

    fn unbond(staker: CanonicalAddr) -> Self {
        Self {
            tx_type: TxCode::Unbond.to_u8(),
            address1: Some(staker),
            address2: None,
            address3: None,
        }
    }

    fn claim_unbond(staker: CanonicalAddr) -> Self {
        Self {
            tx_type: TxCode::ClaimUnbond.to_u8(),
            address1: Some(staker),
            address2: None,
            address3: None,
        }
    }

    fn claim_reward(staker: CanonicalAddr) -> Self {
        Self {
            tx_type: TxCode::ClaimReward.to_u8(),
            address1: Some(staker),
            address2: None,
            address3: None,
        }
    }

    fn into_humanized(self, api: &dyn Api) -> StdResult<TxAction> {
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
        let staker_addr_err = || {
            StdError::generic_err("Missing address in stored Stake transaction. Storage is corrupt")
        };

        // In all of these, we ignore fields that we don't expect to find populated
        let action = match TxCode::from_u8(self.tx_type)? {
            TxCode::Transfer => {
                let from = self.address1.ok_or_else(transfer_addr_err)?;
                let sender = self.address2.ok_or_else(transfer_addr_err)?;
                let recipient = self.address3.ok_or_else(transfer_addr_err)?;
                let from = api.human_address(&from)?;
                let sender = api.human_address(&sender)?;
                let recipient = api.human_address(&recipient)?;
                TxAction::Transfer {
                    from,
                    sender,
                    recipient,
                }
            }
            TxCode::Mint => {
                let minter = self.address1.ok_or_else(mint_addr_err)?;
                let recipient = self.address2.ok_or_else(mint_addr_err)?;
                let minter = api.human_address(&minter)?;
                let recipient = api.human_address(&recipient)?;
                TxAction::Mint { minter, recipient }
            }
            TxCode::Burn => {
                let burner = self.address1.ok_or_else(burn_addr_err)?;
                let owner = self.address2.ok_or_else(burn_addr_err)?;
                let burner = api.human_address(&burner)?;
                let owner = api.human_address(&owner)?;
                TxAction::Burn { burner, owner }
            }
            TxCode::Deposit => TxAction::Deposit {},
            TxCode::Redeem => TxAction::Redeem {},
            TxCode::Stake => {
                let staker = self.address1.ok_or_else(staker_addr_err)?;
                let staker = api.human_address(&staker)?;
                TxAction::Stake { staker }
            }
            TxCode::AddReward => {
                let funder = self.address1.ok_or_else(staker_addr_err)?;
                let funder = api.human_address(&funder)?;
                TxAction::AddReward { funder }
            }
            TxCode::FundUnbond => {
                let funder = self.address1.ok_or_else(staker_addr_err)?;
                let funder = api.human_address(&funder)?;
                TxAction::FundUnbond { funder }
            }
            TxCode::Unbond => {
                let staker = self.address1.ok_or_else(staker_addr_err)?;
                let staker = api.human_address(&staker)?;
                TxAction::Unbond { staker }
            }
            TxCode::ClaimUnbond => {
                let staker = self.address1.ok_or_else(staker_addr_err)?;
                let staker = api.human_address(&staker)?;
                TxAction::ClaimUnbond { staker }
            }
            TxCode::ClaimReward => {
                let staker = self.address1.ok_or_else(staker_addr_err)?;
                let staker = api.human_address(&staker)?;
                TxAction::ClaimReward { staker }
            }
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
    block_time: u64,
    block_height: u64,
}

impl StoredRichTx {
    fn new(
        id: u64,
        action: StoredTxAction,
        coins: Coin,
        memo: Option<String>,
        block: &shade_protocol::c_std::BlockInfo,
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

    fn into_humanized(self, api: &dyn Api) -> StdResult<RichTx> {
        Ok(RichTx {
            id: self.id,
            action: self.action.into_humanized(api)?,
            coins: self.coins,
            memo: self.memo,
            block_time: self.block_time,
            block_height: self.block_height,
        })
    }

    fn from_stored_legacy_transfer(transfer: StoredLegacyTransfer) -> Self {
        let action = StoredTxAction::transfer(transfer.from, transfer.sender, transfer.receiver);
        Self {
            id: transfer.id,
            action,
            coins: transfer.coins,
            memo: transfer.memo,
            block_time: transfer.block_time,
            block_height: transfer.block_height,
        }
    }
}

// Storage functions:

fn increment_tx_count(store: &mut dyn Storage) -> StdResult<u64> {
    let mut config = Config::from_storage(store);
    let id = config.tx_count() + 1;
    config.set_tx_count(id)?;
    Ok(id)
}

#[allow(clippy::too_many_arguments)] // We just need them
pub fn store_transfer(
    store: &mut dyn Storage,
    owner: &CanonicalAddr,
    sender: &CanonicalAddr,
    receiver: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let transfer = StoredLegacyTransfer {
        id,
        from: owner.clone(),
        sender: sender.clone(),
        receiver: receiver.clone(),
        coins,
        memo,
        block_time: block.time,
        block_height: block.height,
    };
    let tx = StoredRichTx::from_stored_legacy_transfer(transfer.clone());

    // Write to the owners history if it's different from the other two addresses
    if owner != sender && owner != receiver {
        // shade_protocol::c_std::debug_print("saving transaction history for owner");
        append_tx(store, &tx, owner)?;
        append_transfer(store, &transfer, owner)?;
    }
    // Write to the sender's history if it's different from the receiver
    if sender != receiver {
        // shade_protocol::c_std::debug_print("saving transaction history for sender");
        append_tx(store, &tx, sender)?;
        append_transfer(store, &transfer, sender)?;
    }
    // Always write to the recipient's history
    // shade_protocol::c_std::debug_print("saving transaction history for receiver");
    append_tx(store, &tx, receiver)?;
    append_transfer(store, &transfer, receiver)?;

    Ok(())
}

pub fn store_mint(
    store: &mut dyn Storage,
    minter: &CanonicalAddr,
    recipient: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::mint(minter.clone(), recipient.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    if minter != recipient {
        append_tx(store, &tx, recipient)?;
    }
    append_tx(store, &tx, minter)?;

    Ok(())
}

pub fn store_burn(
    store: &mut dyn Storage,
    owner: &CanonicalAddr,
    burner: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::burn(owner.clone(), burner.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    if burner != owner {
        append_tx(store, &tx, owner)?;
    }
    append_tx(store, &tx, burner)?;

    Ok(())
}

pub fn store_stake(
    store: &mut dyn Storage,
    staker: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::stake(staker.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    append_tx(store, &tx, staker)?;

    Ok(())
}

pub fn store_add_reward(
    store: &mut dyn Storage,
    staker: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::add_reward(staker.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    append_tx(store, &tx, staker)?;

    Ok(())
}

pub fn store_fund_unbond(
    store: &mut dyn Storage,
    staker: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::fund_unbond(staker.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    append_tx(store, &tx, staker)?;

    Ok(())
}

pub fn store_unbond(
    store: &mut dyn Storage,
    staker: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::unbond(staker.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    append_tx(store, &tx, staker)?;

    Ok(())
}

pub fn store_claim_unbond(
    store: &mut dyn Storage,
    staker: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::claim_unbond(staker.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    append_tx(store, &tx, staker)?;

    Ok(())
}

pub fn store_claim_reward(
    store: &mut dyn Storage,
    staker: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let id = increment_tx_count(store)?;
    let coins = Coin {
        denom,
        amount: amount.into(),
    };
    let action = StoredTxAction::claim_reward(staker.clone());
    let tx = StoredRichTx::new(id, action, coins, memo, block);

    append_tx(store, &tx, staker)?;

    Ok(())
}

fn append_tx(
    store: &mut dyn Storage,
    tx: &StoredRichTx,
    for_address: &CanonicalAddr,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_TXS, for_address.as_slice()], store);
    let mut store = AppendStoreMut::attach_or_create(&mut store)?;
    store.push(tx)
}

fn append_transfer(
    store: &mut dyn Storage,
    tx: &StoredLegacyTransfer,
    for_address: &CanonicalAddr,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_TRANSFERS, for_address.as_slice()], store);
    let mut store = AppendStoreMut::attach_or_create(&mut store)?;
    store.push(tx)
}

pub fn get_txs<A: Api, S: ReadonlyStorage>(
    api: &dyn Api,
    storage: &dyn Storage,
    for_address: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<(Vec<RichTx>, u64)> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_TXS, for_address.as_slice()], storage);

    // Try to access the storage of txs for the account.
    // If it doesn't exist yet, return an empty list of transfers.
    let store = AppendStore::<StoredRichTx, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok((vec![], 0));
    };

    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let tx_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);

    // The `and_then` here flattens the `StdResult<StdResult<RichTx>>` to an `StdResult<RichTx>`
    let txs: StdResult<Vec<RichTx>> = tx_iter
        .map(|tx| tx.map(|tx| tx.into_humanized(api)).and_then(|x| x))
        .collect();
    txs.map(|txs| (txs, store.len() as u64))
}

pub fn get_transfers<A: Api, S: ReadonlyStorage>(
    api: &dyn Api,
    storage: &dyn Storage,
    for_address: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<(Vec<Tx>, u64)> {
    let store =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_TRANSFERS, for_address.as_slice()], storage);

    // Try to access the storage of transfers for the account.
    // If it doesn't exist yet, return an empty list of transfers.
    let store = AppendStore::<StoredLegacyTransfer, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok((vec![], 0));
    };

    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let transfer_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);

    // The `and_then` here flattens the `StdResult<StdResult<RichTx>>` to an `StdResult<RichTx>`
    let transfers: StdResult<Vec<Tx>> = transfer_iter
        .map(|tx| tx.map(|tx| tx.into_humanized(api)).and_then(|x| x))
        .collect();
    transfers.map(|txs| (txs, store.len() as u64))
}
