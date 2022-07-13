use shade_protocol::c_std::{Binary, Addr, Uint128};
use network_integration::utils::{
    generate_label, print_contract, print_header, SHD_STAKING_FILE, GAS, SNIP20_FILE, STORE_GAS,
};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use secretcli::cli_types::NetContract;
use secretcli::secretcli::{account_address, init};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use shade_protocol::utils::asset::Contract;
use shade_protocol::contract_interfaces::{
    staking::snip20_staking,
    snip20,
};
use std::{env, fs};
use shade_protocol::contract_interfaces::snip20::InitialBalance;

fn main() -> Result<()> {
    // Initialize snip20
    print_header("Initializing Snip20");

    let snip_init_msg = snip20::InitMsg {
        name: "Shade".to_string(),
        admin: None,
        symbol: "SHD".to_string(),
        decimals: 8,
        initial_balances: Some(vec![InitialBalance {
            address: Addr::from("secret1xtl6rt2pwhseuzct00h8uw6trkzjj2l8lu38se".to_string()),
            amount: Uint128::new(1000000000000000),
        }]),
        prng_seed: Default::default(),
        config: Some(snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(true),
            enable_burn: Some(true),
            enable_transfer: Some(true),
        }),
    };

    let snip = init(
        &snip_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        "test1",
        Some(STORE_GAS),
        Some(GAS),
        None,
        &mut vec![],
    )?;

    print_contract(&snip);

    // Initialize staker
    print_header("Initializing Staking");
    let init_msg = snip20_staking::InitMsg {
        name: "StakedShade".to_string(),
        admin: None,
        symbol: "STKSHD".to_string(),
        decimals: Some(8),
        share_decimals: 18,
        prng_seed: Default::default(),
        public_total_supply: true,
        unbond_time: 180,
        staked_token: Contract { address: Addr(snip.address.clone()), code_hash: snip.code_hash },
        treasury: Some(Addr(snip.address)),
        treasury_code_hash: None,
        limit_transfer: true,
        distributors: None
    };

    let stake = init(
        &init_msg,
        SHD_STAKING_FILE,
        &*generate_label(8),
        "test1",
        Some(STORE_GAS),
        Some(GAS),
        None,
        &mut vec![],
    )?;

    print_contract(&stake);

    Ok(())
}
