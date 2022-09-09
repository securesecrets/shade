#![allow(clippy::field_reassign_with_default)] // This is triggered in `#[derive(JsonSchema)]`


use shade_protocol::cosmwasm_schema::cw_serde;

use crate::{
    batch,
    transaction_history::{RichTx, Tx},
    viewing_key::ViewingKey,
};
use shade_protocol::c_std::{Uint128, Uint256};
use shade_protocol::c_std::{Binary, Addr, StdError, StdResult};
use shade_protocol::secret_toolkit::permit::Permit;
use shade_protocol::{
    contract_interfaces::staking::snip20_staking::stake::{QueueItem, StakeConfig, VecQueue},
    utils::asset::Contract,
};

#[derive(Serialize, Deserialize)]
pub struct InstantiateMsg {
    pub name: String,
    pub admin: Option<Addr>,
    pub symbol: String,
    // Will default to staked token decimals if not set
    pub decimals: Option<u8>,
    pub share_decimals: u8,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,

    // Stake
    pub unbond_time: u64,
    pub staked_token: Contract,
    pub treasury: Option<Addr>,
    pub treasury_code_hash: Option<String>,

    // Distributors
    pub limit_transfer: bool,
    pub distributors: Option<Vec<Addr>>,
}

impl InstantiateMsg {
    pub fn config(&self) -> InitConfig {
        self.config.clone().unwrap_or_default()
    }
}

/// This type represents optional configuration values which can be overridden.
/// All values are optional and have defaults which are more private by default,
/// but can be overridden if necessary
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InitConfig {
    /// Indicates whether the total supply is public or should be kept secret.
    /// default: False
    pub public_total_supply: Option<bool>,
}

impl InitConfig {
    pub fn public_total_supply(&self) -> bool {
        self.public_total_supply.unwrap_or(false)
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    // Staking
    UpdateStakeConfig {
        unbond_time: Option<u64>,
        disable_treasury: bool,
        treasury: Option<Addr>,
        padding: Option<String>,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    Unbond {
        amount: Uint128,
        padding: Option<String>,
    },
    ClaimUnbond {
        padding: Option<String>,
    },
    ClaimRewards {
        padding: Option<String>,
    },
    StakeRewards {
        padding: Option<String>,
    },

    // Balance
    ExposeBalance {
        recipient: Addr,
        code_hash: Option<String>,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    ExposeBalanceWithCooldown {
        recipient: Addr,
        code_hash: Option<String>,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },

    // Distributors
    SetDistributorsStatus {
        enabled: bool,
        padding: Option<String>,
    },
    AddDistributors {
        distributors: Vec<Addr>,
        padding: Option<String>,
    },
    SetDistributors {
        distributors: Vec<Addr>,
        padding: Option<String>,
    },

    // Base ERC-20 stuff
    Transfer {
        recipient: Addr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    Send {
        recipient: Addr,
        recipient_code_hash: Option<String>,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    BatchTransfer {
        actions: Vec<batch::TransferAction>,
        padding: Option<String>,
    },
    BatchSend {
        actions: Vec<batch::SendAction>,
        padding: Option<String>,
    },
    RegisterReceive {
        code_hash: String,
        padding: Option<String>,
    },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },

    // Allowance
    IncreaseAllowance {
        spender: Addr,
        amount: Uint128,
        expiration: Option<u64>,
        padding: Option<String>,
    },
    DecreaseAllowance {
        spender: Addr,
        amount: Uint128,
        expiration: Option<u64>,
        padding: Option<String>,
    },
    TransferFrom {
        owner: Addr,
        recipient: Addr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    SendFrom {
        owner: Addr,
        recipient: Addr,
        recipient_code_hash: Option<String>,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    BatchTransferFrom {
        actions: Vec<batch::TransferFromAction>,
        padding: Option<String>,
    },
    BatchSendFrom {
        actions: Vec<batch::SendFromAction>,
        padding: Option<String>,
    },

    // Admin
    ChangeAdmin {
        address: Addr,
        padding: Option<String>,
    },
    SetContractStatus {
        level: ContractStatusLevel,
        padding: Option<String>,
    },

    // Permit
    RevokePermit {
        permit_name: String,
        padding: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateStakeConfig {
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
    },
    Unbond {
        status: ResponseStatus,
    },
    ClaimUnbond {
        status: ResponseStatus,
    },
    ClaimRewards {
        status: ResponseStatus,
    },
    StakeRewards {
        status: ResponseStatus,
    },
    ExposeBalance {
        status: ResponseStatus,
    },
    SetDistributorsStatus {
        status: ResponseStatus,
    },
    AddDistributors {
        status: ResponseStatus,
    },
    SetDistributors {
        status: ResponseStatus,
    },

    // Base
    Transfer {
        status: ResponseStatus,
    },
    Send {
        status: ResponseStatus,
    },
    BatchTransfer {
        status: ResponseStatus,
    },
    BatchSend {
        status: ResponseStatus,
    },
    RegisterReceive {
        status: ResponseStatus,
    },
    CreateViewingKey {
        key: ViewingKey,
    },
    SetViewingKey {
        status: ResponseStatus,
    },

    // Allowance
    IncreaseAllowance {
        spender: Addr,
        owner: Addr,
        allowance: Uint128,
    },
    DecreaseAllowance {
        spender: Addr,
        owner: Addr,
        allowance: Uint128,
    },
    TransferFrom {
        status: ResponseStatus,
    },
    SendFrom {
        status: ResponseStatus,
    },
    BatchTransferFrom {
        status: ResponseStatus,
    },
    BatchSendFrom {
        status: ResponseStatus,
    },

    // Other
    ChangeAdmin {
        status: ResponseStatus,
    },
    SetContractStatus {
        status: ResponseStatus,
    },

    // Permit
    RevokePermit {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Staking
    StakeConfig {},
    TotalStaked {},
    // Total token shares per token
    StakeRate {},
    Unbonding {},
    Unfunded {
        start: u64,
        total: u64,
    },
    Staked {
        address: Addr,
        key: String,
        time: Option<u64>,
    },

    // Distributors
    Distributors {},

    // Snip20 stuff
    TokenInfo {},
    TokenConfig {},
    ContractStatus {},
    Allowance {
        owner: Addr,
        spender: Addr,
        key: String,
    },
    Balance {
        address: Addr,
        key: String,
    },
    TransferHistory {
        address: Addr,
        key: String,
        page: Option<u32>,
        page_size: u32,
    },
    TransactionHistory {
        address: Addr,
        key: String,
        page: Option<u32>,
        page_size: u32,
    },
    WithPermit {
        permit: Permit,
        query: QueryWithPermit,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&Addr>, ViewingKey) {
        match self {
            Self::Staked { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::Balance { address, key } => (vec![address], ViewingKey(key.clone())),
            Self::TransferHistory { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::TransactionHistory { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
            Self::Allowance {
                owner,
                spender,
                key,
                ..
            } => (vec![owner, spender], ViewingKey(key.clone())),
            _ => panic!("This query type does not require authentication"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    Staked {
        time: Option<u64>,
    },

    // Snip20 stuff
    Allowance {
        owner: Addr,
        spender: Addr,
    },
    Balance {},
    TransferHistory {
        page: Option<u32>,
        page_size: u32,
    },
    TransactionHistory {
        page: Option<u32>,
        page_size: u32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    // Stake
    StakedConfig {
        config: StakeConfig,
    },
    TotalStaked {
        tokens: Uint128,
        shares: Uint256,
    },
    // Shares per token
    StakeRate {
        shares: Uint256,
    },
    Staked {
        tokens: Uint128,
        shares: Uint256,
        pending_rewards: Uint128,
        unbonding: Uint128,
        unbonded: Option<Uint128>,
        cooldown: VecQueue<QueueItem>,
    },
    Unbonding {
        total: Uint128,
    },
    Unfunded {
        total: Uint128,
    },

    // Distributors
    Distributors {
        distributors: Option<Vec<Addr>>,
    },

    // Snip20 stuff
    TokenInfo {
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: Option<Uint128>,
    },
    TokenConfig {
        public_total_supply: bool,
    },
    ContractStatus {
        status: ContractStatusLevel,
    },
    ExchangeRate {
        rate: Uint128,
        denom: String,
    },
    Allowance {
        spender: Addr,
        owner: Addr,
        allowance: Uint128,
        expiration: Option<u64>,
    },
    Balance {
        amount: Uint128,
    },
    TransferHistory {
        txs: Vec<Tx>,
        total: Option<u64>,
    },
    TransactionHistory {
        txs: Vec<RichTx>,
        total: Option<u64>,
    },
    ViewingKeyError {
        msg: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateViewingKeyResponse {
    pub key: String,
}

#[cw_serde]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[cw_serde]
pub enum ContractStatusLevel {
    NormalRun,
    StopBonding,
    StopAllButUnbond, //Can set time to 0 for instant unbond
    StopAll,
}

pub fn status_level_to_u8(status_level: ContractStatusLevel) -> u8 {
    match status_level {
        ContractStatusLevel::NormalRun => 0,
        ContractStatusLevel::StopBonding => 1,
        ContractStatusLevel::StopAllButUnbond => 2,
        ContractStatusLevel::StopAll => 3,
    }
}

pub fn u8_to_status_level(status_level: u8) -> StdResult<ContractStatusLevel> {
    match status_level {
        0 => Ok(ContractStatusLevel::NormalRun),
        1 => Ok(ContractStatusLevel::StopBonding),
        2 => Ok(ContractStatusLevel::StopAllButUnbond),
        3 => Ok(ContractStatusLevel::StopAll),
        _ => Err(StdError::generic_err("Invalid state level")),
    }
}

// Take a Vec<u8> and pad it up to a multiple of `block_size`, using spaces at the end.
pub fn space_pad(block_size: usize, message: &mut Vec<u8>) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use shade_protocol::c_std::{from_slice, StdResult};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum Something {
        Var { padding: Option<String> },
    }

    #[test]
    fn test_deserialization_of_missing_option_fields() -> StdResult<()> {
        let input = b"{ \"var\": {} }";
        let obj: Something = from_slice(input)?;
        assert_eq!(
            obj,
            Something::Var { padding: None },
            "unexpected value: {:?}",
            obj
        );
        Ok(())
    }
}
