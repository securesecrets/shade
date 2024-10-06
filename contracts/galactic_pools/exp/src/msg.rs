use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shade_protocol::{
    c_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, Uint128, WasmMsg},
    s_toolkit::{permit::Permit, utils::types::Contract},
};

use crate::state::Schedule;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<Vec<Addr>>,
    pub entropy: String,
    pub grand_prize_contract: Option<Addr>,
    pub schedules: MintingSchedule,
    pub season_ending_block: u64,
}

pub type MintingSchedule = Vec<MintingScheduleUint>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MintingScheduleUint {
    pub continue_with_current_season: bool,
    pub duration: u64,
    pub mint_per_block: Uint128,
    pub start_after: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddAdmin { address: Addr },
    AddContract { contracts: Vec<AddContract> },
    AddExp { address: Addr, exp: Uint128 },
    CreateViewingKey { entropy: String },
    GetWinners { no_of_winners: Option<u64> },
    RemoveAdmin { address: Addr },
    RemoveContract { contracts: Vec<Addr> },
    ResetSeason {},
    SetGrandPrizeContract { address: Addr },
    SetSchedule { schedule: MintingSchedule },
    SetViewingKey { key: String },
    UpdateLastClaimed {},
    UpdateWeights { weights: Vec<WeightUpdate> },
}

//////////////////////////////////////////////////////////////// Handle Answer ////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteAnswer {
    // Alphabetically sorted
    AddAdmin { status: ResponseStatus },
    AddContract { status: ResponseStatus },
    AddExp { status: ResponseStatus },
    BurnExp { status: ResponseStatus },
    CreateViewingKey { status: ResponseStatus },
    EndRound { status: ResponseStatus },
    GetWinners { winners: Vec<Addr> },
    Instantiate { status: ResponseStatus },
    RemoveAdmin { status: ResponseStatus },
    RemoveContract { status: ResponseStatus },
    SetGrandPrizeContract { status: ResponseStatus },
    SetSchedule { status: ResponseStatus },
    SetViewingKey { status: ResponseStatus },
    UpdateLastClaimed { status: ResponseStatus },
    UpdateRng { status: ResponseStatus },
    UpdateWeights { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

impl ExecuteMsg {
    /// Returns a StdResult<CosmosMsg> used to execute a SNIP20 contract function
    ///
    /// # Arguments
    ///
    /// * `block_size` - pad the message to blocks of this size
    /// * `callback_code_hash` - String holding the code hash of the contract being called
    /// * `contract_addr` - address of the contract being called
    /// * `send_amount` - Optional Uint128 amount of native coin to send with the callback message
    ///                 NOTE: Only a Deposit message should have an amount sent with it
    pub fn to_cosmos_msg(
        &self,
        mut block_size: usize,
        code_hash: String,
        contract_addr: String,
        send_amount: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        // can not have block size of 0
        if block_size == 0 {
            block_size = 1;
        }
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, block_size);
        let mut funds = Vec::new();
        if let Some(amount) = send_amount {
            funds.push(Coin {
                amount,
                denom: String::from("uscrt"),
            });
        }
        let execute = WasmMsg::Execute {
            contract_addr,
            code_hash,
            msg,
            funds,
        };
        Ok(execute.into())
    }
}
/// Take a Vec<u8> and pad it up to a multiple of `block_size`, using spaces at the end.
pub fn space_pad(message: &mut Vec<u8>, block_size: usize) -> &mut Vec<u8> {
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CheckUserExp {
        address: Addr,
        key: String,
        user_address: Addr,
        season: Option<u64>,
    },
    Contract {
        address: Addr,
        key: String,
    },
    ContractInfo {},
    GetWinner {
        address: Addr,
        key: String,
        no_of_winners: Option<u64>,
    },
    UserExp {
        address: Addr,
        key: String,
        season: Option<u64>,
    },
    VerifiedContracts {
        page_size: Option<u32>,
        start_page: Option<u32>,
    },
    WithPermit {
        permit: Permit,
        query: QueryWithPermit,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (String, String) {
        match self {
            Self::CheckUserExp { address, key, .. } => (address.to_string(), key.clone()),
            Self::Contract { address, key } => (address.to_string(), key.clone()),
            Self::GetWinner { address, key, .. } => (address.to_string(), key.to_string()),
            Self::UserExp { address, key, .. } => (address.to_string(), key.clone()),

            _ => panic!("This query type does not require authentication"),
        }
    }
}

/// queries using permits instead of viewing keys
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    UserExp { season: Option<u64> },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    ContractInfoResponse {
        info: ConfigRes,
    },
    ContractResponse {
        available_exp: Uint128,
        last_claimed: u64,
        total_xp: Uint128,
        unclaimed_exp: Uint128,
        weight: u64,
        xp_claimed: Uint128,
    },
    GetWinnersResponse {
        winners: Vec<Addr>,
    },
    UserExp {
        exp: Uint128,
    },
    VerifiedContractsResponse {
        contracts: Vec<VerifiedContractRes>,
    },
    ViewingKeyError {
        error: String,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct ConfigRes {
    pub admins: Vec<Addr>,
    pub contract_address: Addr,
    pub current_block: u64,
    pub minting_schedule: Schedule,
    pub season_count: u64,
    pub season_duration: u64,
    pub season_ending_block: u64,
    pub season_starting_block: u64,
    pub season_total_xp_cap: Uint128,
    pub total_weight: u64,
    pub verified_contracts: Vec<Addr>,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct VerifiedContractRes {
    pub address: Addr,
    pub available_xp: Uint128,
    pub code_hash: String,
    pub last_claimed: u64,
    pub weight: u64,
}

/// code hash and address of a contract
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct AddContract {
    /// contract's code hash string
    pub address: Addr,
    pub code_hash: String,
    pub weight: u64,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct WeightUpdate {
    pub address: Addr,
    pub weight: u64,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct Entropy {
    pub entropy: [u8; 32],
    pub seed: [u8; 32],
}
