pub mod errors;
pub mod rand;
pub mod utils;

use crate::c_std::Env;
use cosmwasm_std::MessageInfo;

use crate::{
    c_std::{Addr, Binary, Uint128},
    contract_interfaces::{
        bonds::{
            rand::{sha_256, Prng},
            utils::{
                create_hashed_password,
                ct_slice_compare,
                VIEWING_KEY_PREFIX,
                VIEWING_KEY_SIZE,
            },
        },
        query_auth::QueryPermit,
        snip20::helpers::Snip20Asset,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::utils::ExecuteCallback;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    pub limit_admin: Addr,
    pub shade_admin: Contract,
    pub oracle: Contract,
    pub treasury: Addr,
    pub issued_asset: Contract,
    pub activated: bool,
    pub bond_issuance_limit: Uint128,
    pub bonding_period: u64,
    pub discount: Uint128,
    pub global_issuance_limit: Uint128,
    pub global_minimum_bonding_period: u64,
    pub global_maximum_discount: Uint128,
    pub global_min_accepted_issued_price: Uint128,
    pub global_err_issued_price: Uint128,
    pub contract: Addr,
    pub airdrop: Option<Contract>,
    pub query_auth: Contract,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub limit_admin: Addr,
    pub global_issuance_limit: Uint128,
    pub global_minimum_bonding_period: u64,
    pub global_maximum_discount: Uint128,
    pub shade_admin: Contract,
    pub oracle: Contract,
    pub treasury: Addr,
    pub issued_asset: Contract,
    pub activated: bool,
    pub bond_issuance_limit: Uint128,
    pub bonding_period: u64,
    pub discount: Uint128,
    pub global_min_accepted_issued_price: Uint128,
    pub global_err_issued_price: Uint128,
    pub allowance_key_entropy: String,
    pub airdrop: Option<Contract>,
    pub query_auth: Contract,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateLimitConfig {
        limit_admin: Option<Addr>,
        shade_admin: Option<Contract>,
        global_issuance_limit: Option<Uint128>,
        global_minimum_bonding_period: Option<u64>,
        global_maximum_discount: Option<Uint128>,
        reset_total_issued: Option<bool>,
        reset_total_claimed: Option<bool>,
        padding: Option<String>,
    },
    UpdateConfig {
        oracle: Option<Contract>,
        treasury: Option<Addr>,
        issued_asset: Option<Contract>,
        activated: Option<bool>,
        bond_issuance_limit: Option<Uint128>,
        bonding_period: Option<u64>,
        discount: Option<Uint128>,
        global_min_accepted_issued_price: Option<Uint128>,
        global_err_issued_price: Option<Uint128>,
        allowance_key: Option<String>,
        airdrop: Option<Contract>,
        query_auth: Option<Contract>,
        padding: Option<String>,
    },
    OpenBond {
        deposit_asset: Contract,
        start_time: u64,
        end_time: u64,
        bond_issuance_limit: Option<Uint128>,
        bonding_period: Option<u64>,
        discount: Option<Uint128>,
        max_accepted_deposit_price: Uint128,
        err_deposit_price: Uint128,
        minting_bond: bool,
        padding: Option<String>,
    },
    CloseBond {
        deposit_asset: Contract,
        padding: Option<String>,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        msg: Option<Binary>,
        padding: Option<String>,
    },
    Claim {
        padding: Option<String>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
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
        max_accepted_deposit_price: Uint128,
        err_deposit_price: Uint128,
        minting_bond: bool,
    },
    ClosedBond {
        status: ResponseStatus,
        deposit_asset: Contract,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    BondOpportunities {},
    Account { permit: QueryPermit },
    DepositAddresses {},
    PriceCheck { asset: String },
    BondInfo {},
    CheckAllowance {},
    CheckBalance {},
}

#[cw_serde]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    BondOpportunities {
        bond_opportunities: Vec<BondOpportunity>,
    },
    Account {
        pending_bonds: Vec<PendingBond>,
    },
    DepositAddresses {
        deposit_addresses: Vec<Addr>,
    },
    PriceCheck {
        price: Uint128,
    },
    BondInfo {
        global_total_issued: Uint128,
        global_total_claimed: Uint128,
        issued_asset: Snip20Asset,
        global_min_accepted_issued_price: Uint128,
        global_err_issued_price: Uint128,
    },
    CheckAllowance {
        allowance: Uint128,
    },
    CheckBalance {
        balance: Uint128,
    },
}

#[cw_serde]
pub struct Account {
    pub address: Addr,
    pub pending_bonds: Vec<PendingBond>,
}

#[cw_serde]
pub struct SnipViewingKey(pub String);

impl SnipViewingKey {
    pub fn check_viewing_key(&self, hashed_pw: &[u8]) -> bool {
        let mine_hashed = create_hashed_password(&self.0);

        ct_slice_compare(&mine_hashed, hashed_pw)
    }

    pub fn new(info: &MessageInfo, env: &Env, seed: &[u8], entropy: &[u8]) -> Self {
        // 16 here represents the lengths in bytes of the block height and time.
        let entropy_len = 16 + info.sender.as_str().len() + entropy.len();
        let mut rng_entropy = Vec::with_capacity(entropy_len);
        rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
        rng_entropy.extend_from_slice(&env.block.time.seconds().to_be_bytes());
        rng_entropy.extend_from_slice(&info.sender.as_str().as_bytes());
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

#[cw_serde]
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
#[cw_serde]
pub struct BondOpportunity {
    pub issuance_limit: Uint128,
    pub amount_issued: Uint128,
    pub deposit_denom: Snip20Asset,
    pub start_time: u64,
    pub end_time: u64,
    pub bonding_period: u64,
    pub discount: Uint128,
    pub max_accepted_deposit_price: Uint128,
    pub err_deposit_price: Uint128,
    pub minting_bond: bool,
}

#[cw_serde]
pub struct SlipMsg {
    pub minimum_expected_amount: Uint128,
}
