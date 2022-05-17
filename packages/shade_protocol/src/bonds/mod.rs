pub mod errors;
pub mod utils;
pub mod rand;

use cosmwasm_std::Env;

use query_authentication::viewing_keys::ViewingKey;
use query_authentication::{
    permit::{Permit, bech32_to_canonical}};
    
use crate::bonds::utils::{create_hashed_password, ct_slice_compare, VIEWING_KEY_PREFIX, VIEWING_KEY_SIZE};
use crate::utils::generic_response::ResponseStatus;
use crate::utils::asset::Contract;
use crate::bonds::rand::{sha_256, Prng};
use crate::bonds::errors::{permit_rejected};
use cosmwasm_std::{Binary, HumanAddr, Uint128, StdResult, StdError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::snip20::Snip20Asset;
use secret_toolkit::utils::{HandleCallback};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub limit_admin: HumanAddr,
    pub admin: HumanAddr,
    pub oracle: Contract,
    pub treasury: HumanAddr,
    pub issued_asset: Contract,
    pub activated: bool,
    pub minting_bond: bool,
    pub bond_issuance_limit: Uint128,
    pub bonding_period: u64,
    pub discount: Uint128,
    pub global_issuance_limit: Uint128,
    pub global_minimum_bonding_period: u64,
    pub global_maximum_discount: Uint128,
    pub global_minimum_issued_price: Uint128,
    pub contract: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub limit_admin: HumanAddr,
    pub global_issuance_limit: Uint128,
    pub global_minimum_bonding_period: u64,
    pub global_maximum_discount: Uint128,
    pub admin: HumanAddr,
    pub oracle: Contract,
    pub treasury: HumanAddr,
    pub issued_asset: Contract,
    pub activated: bool,
    pub minting_bond: bool,
    pub bond_issuance_limit: Uint128,
    pub bonding_period: u64,
    pub discount: Uint128,
    pub global_minimum_issued_price: Uint128,
    pub allowance_key_entropy: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateLimitConfig {
        limit_admin: Option<HumanAddr>,
        global_issuance_limit: Option<Uint128>,
        global_minimum_bonding_period: Option<u64>,
        global_maximum_discount: Option<Uint128>,
        reset_total_issued: Option<bool>,
        reset_total_claimed: Option<bool>,
        padding: Option<String>,
    },
    UpdateConfig {
        admin: Option<HumanAddr>,
        oracle: Option<Contract>,
        treasury: Option<HumanAddr>,
        issued_asset: Option<Contract>,
        activated: Option<bool>,
        minting_bond: Option<bool>,
        bond_issuance_limit: Option<Uint128>,
        bonding_period: Option<u64>,
        discount: Option<Uint128>,
        global_minimum_issued_price: Option<Uint128>,
        allowance_key: Option<String>,
        padding: Option<String>,
    },
    OpenBond {
        collateral_asset: Contract,
        start_time: u64,
        end_time: u64,
        bond_issuance_limit: Option<Uint128>,
        bonding_period: Option<u64>,
        discount: Option<Uint128>,
        max_accepted_collateral_price: Uint128,
        err_collateral_price: Uint128,
        padding: Option<String>,
    },
    CloseBond {
        collateral_asset: Contract,
        padding: Option<String>,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Option<Binary>,
        padding: Option<String>,
    },
    Claim {
        padding: Option<String>,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateLimitConfig {
        status: ResponseStatus,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    Deposit {
        status: ResponseStatus,
        deposit_amount: Uint128,
        pending_claim_amount: Uint128,
        end_date: u64,
    },
    Claim {
        status: ResponseStatus,
        amount: Uint128,
    },
    OpenBond {
        status: ResponseStatus,
        deposit_contract: Contract,
        start_time: u64,
        end_time: u64,
        bond_issuance_limit: Uint128,
        bonding_period: u64,
        discount: Uint128,
        max_accepted_collateral_price: Uint128,
        err_collateral_price: Uint128,
    },
    ClosedBond {
        status: ResponseStatus,
        collateral_asset: Contract,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BondOpportunities {},
    Account {
        permit: AccountPermit,
    },
    CollateralAddresses {},
    PriceCheck {
        asset: String,
    },
    BondInfo {},
    CheckAllowance {},
}   

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    BondOpportunities {
        bond_opportunities: Vec<BondOpportunity>
    },
    Account {
        pending_bonds: Vec<PendingBond>,
    },
    CollateralAddresses {
        collateral_addresses: Vec<HumanAddr>,
    },
    PriceCheck {
        price: Uint128,
    },
    BondInfo {
        global_total_issued: Uint128,
        global_total_claimed: Uint128,
        issued_asset: Snip20Asset,
    },
    CheckAllowance {
        allowance: Uint128,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub address: HumanAddr,
    pub pending_bonds: Vec<PendingBond>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountKey(pub String);

impl ToString for AccountKey {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

//impl ViewingKey<32> for AccountKey {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SnipViewingKey(pub String);

impl SnipViewingKey {
    pub fn check_viewing_key(&self, hashed_pw: &[u8]) -> bool {
        let mine_hashed = create_hashed_password(&self.0);

        ct_slice_compare(&mine_hashed, hashed_pw)
    }

    pub fn new(env: &Env, seed: &[u8], entropy: &[u8]) -> Self {
        // 16 here represents the lengths in bytes of the block height and time.
        let entropy_len = 16 + env.message.sender.len() + entropy.len();
        let mut rng_entropy = Vec::with_capacity(entropy_len);
        rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
        rng_entropy.extend_from_slice(&env.block.time.to_be_bytes());
        rng_entropy.extend_from_slice(&env.message.sender.0.as_bytes());
        rng_entropy.extend_from_slice(entropy);

        let mut rng = Prng::new(seed, &rng_entropy);

        let rand_slice = rng.rand_bytes();

        let key = sha_256(&rand_slice);

        Self(VIEWING_KEY_PREFIX.to_string() + &base64::encode(key))
    }

    pub fn to_hashed(&self) -> [u8; VIEWING_KEY_SIZE] {
        create_hashed_password(&self.0)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

// Used for querying account information
pub type AccountPermit = Permit<AccountPermitMsg>;

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountPermitMsg {
    pub contract: HumanAddr,
    pub key: String,
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FillerMsg {
    pub coins: Vec<String>,
    pub contract: String,
    pub execute_msg: EmptyMsg,
    pub sender: String,
}

impl Default for FillerMsg {
    fn default() -> Self {
        Self {
            coins: vec![],
            contract: "".to_string(),
            sender: "".to_string(),
            execute_msg: EmptyMsg {},
        }
    }
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EmptyMsg {}

// Used to prove ownership over IBC addresses
pub type AddressProofPermit = Permit<FillerMsg>;

pub fn authenticate_ownership(permit: &AddressProofPermit, permit_address: &str) -> StdResult<()> {
    let signer_address = permit
        .validate(Some("wasm/MsgExecuteContract".to_string()))?
        .as_canonical();

    if signer_address != bech32_to_canonical(permit_address) {
        return Err(permit_rejected());
    }

    Ok(())
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddressProofMsg {
    // Address is necessary since we have other network permits present
    pub address: HumanAddr,
    // Reward amount
    pub amount: Uint128,
    // Used to prevent permits from being used elsewhere
    pub contract: HumanAddr,
    // Index of the address in the leaves array
    pub index: u32,
    // Used to identify permits
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PendingBond {
    pub deposit_denom: Snip20Asset,
    pub end_time: u64, // Will be turned into a time via block time calculations
    pub deposit_amount: Uint128,
    pub deposit_price: Uint128,
    pub claim_amount: Uint128,
    pub claim_price: Uint128,
    pub discount: Uint128,
    pub discount_price: Uint128,
}

// When users deposit and try to use the bond, a Bond Opportunity is selected via deposit denom
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BondOpportunity {
    pub issuance_limit: Uint128,
    pub amount_issued: Uint128,
    pub deposit_denom: Snip20Asset,
    pub start_time: u64,
    pub end_time: u64,
    pub bonding_period: u64,
    pub discount: Uint128,
    pub max_accepted_collateral_price: Uint128,
    pub err_collateral_price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SlipMsg {
    pub minimum_expected_amount: Uint128,
}