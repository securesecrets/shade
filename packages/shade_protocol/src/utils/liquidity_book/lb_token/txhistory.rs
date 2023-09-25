use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Api, CanonicalAddr, StdResult, Uint256};
/// tx type and specifics for storage
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StoredTxAction {
    Mint {
        minter: CanonicalAddr,
        recipient: CanonicalAddr,
        amount: Uint256,
    },
    Burn {
        /// in the base specification, the burner MUST be the owner. In the additional
        /// specifications, it is OPTIONAL to allow other addresses to burn tokens.
        burner: Option<CanonicalAddr>,
        owner: CanonicalAddr,
        amount: Uint256,
    },
    /// `transfer` or `send` txs
    Transfer {
        /// previous owner
        from: CanonicalAddr,
        /// optional sender if not owner
        sender: Option<CanonicalAddr>,
        /// new owner
        recipient: CanonicalAddr,
        /// amount of tokens transferred
        amount: Uint256,
    },
}

/// tx in storage
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StoredTx {
    /// tx id
    pub tx_id: u64,
    /// the block containing this tx
    pub block_height: u64,
    /// the time (in seconds since 01/01/1970) of the block containing this tx
    pub block_time: u64,
    /// token id
    pub token_id: String,
    /// tx type and specifics
    pub action: StoredTxAction,
    /// optional memo
    pub memo: Option<String>,
}

impl StoredTx {
    pub fn into_humanized(self, api: &dyn Api) -> StdResult<Tx> {
        let action = match self.action {
            StoredTxAction::Mint {
                minter,
                recipient,
                amount,
            } => TxAction::Mint {
                minter: api.addr_humanize(&minter)?,
                recipient: api.addr_humanize(&recipient)?,
                amount,
            },
            StoredTxAction::Burn {
                burner,
                owner,
                amount,
            } => {
                let bnr = if let Some(b) = burner {
                    Some(api.addr_humanize(&b)?)
                } else {
                    None
                };
                TxAction::Burn {
                    burner: bnr,
                    owner: api.addr_humanize(&owner)?,
                    amount,
                }
            }
            StoredTxAction::Transfer {
                from,
                sender,
                recipient,
                amount,
            } => {
                let sdr = if let Some(s) = sender {
                    Some(api.addr_humanize(&s)?)
                } else {
                    None
                };
                TxAction::Transfer {
                    from: api.addr_humanize(&from)?,
                    sender: sdr,
                    recipient: api.addr_humanize(&recipient)?,
                    amount,
                }
            }
        };
        let tx = Tx {
            tx_id: self.tx_id,
            block_height: self.block_height,
            block_time: self.block_time,
            token_id: self.token_id,
            action,
            memo: self.memo,
        };

        Ok(tx)
    }
}

/// tx type and specifics for storage with Addr
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TxAction {
    Mint {
        minter: Addr,
        recipient: Addr,
        amount: Uint256,
    },
    Burn {
        /// in the base specification, the burner MUST be the owner. In the additional
        /// specifications, it is OPTIONAL to allow other addresses to burn tokens.
        burner: Option<Addr>,
        owner: Addr,
        amount: Uint256,
    },
    /// `transfer` or `send` txs
    Transfer {
        /// previous owner
        from: Addr,
        /// optional sender if not owner
        sender: Option<Addr>,
        /// new owner
        recipient: Addr,
        /// amount of tokens transferred
        amount: Uint256,
    },
}

/// tx in storage
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Tx {
    /// tx id
    pub tx_id: u64,
    /// the block containing this tx
    pub block_height: u64,
    /// the time (in seconds since 01/01/1970) of the block containing this tx
    pub block_time: u64,
    /// token id
    pub token_id: String,
    /// tx type and specifics
    pub action: TxAction,
    /// optional memo
    pub memo: Option<String>,
}
