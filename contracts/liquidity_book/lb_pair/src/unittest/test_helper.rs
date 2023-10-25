use core::result::Result::Ok;
use std::{any::Any, str::FromStr};

use cosmwasm_std::{
    Addr,
    BlockInfo,
    ContractInfo,
    StdError,
    StdResult,
    Timestamp,
    Uint128,
    Uint256,
};
use serde::de::Error;
use shade_multi_test::{
    interfaces::{
        lb_pair,
        snip20,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::{admin::init_admin_auth, lb_token::LbToken},
};
use shade_protocol::{
    lb_libraries::{
        tokens::TokenType,
        types::{ContractInstantiationInfo, StaticFeeParameters},
    },
    liquidity_book::lb_pair::{LiquidityParameters, RemoveLiquidity},
    multi_test::App,
    utils::{asset::Contract, cycle::parse_utc_datetime, MultiTestable},
};

use crate::error;

pub struct Addrs {
    addrs: Vec<Addr>,
    hashes: Vec<String>,
}

impl Addrs {
    pub fn admin(&self) -> Addr {
        self.addrs[0].clone()
    }

    pub fn user1(&self) -> Addr {
        self.addrs[1].clone()
    }

    pub fn user2(&self) -> Addr {
        self.addrs[2].clone()
    }

    pub fn user3(&self) -> Addr {
        self.addrs[3].clone()
    }

    pub fn all(&self) -> Vec<Addr> {
        self.addrs.clone()
    }

    pub fn a_hash(&self) -> String {
        self.hashes[0].clone()
    }

    pub fn b_hash(&self) -> String {
        self.hashes[1].clone()
    }

    pub fn c_hash(&self) -> String {
        self.hashes[2].clone()
    }

    pub fn _d_hash(&self) -> String {
        self.hashes[3].clone()
    }
}

/// inits 3 addresses
pub fn init_addrs() -> Addrs {
    let addr_strs = vec!["addr0", "addr1", "addr2", "addr3"];
    let hashes = vec![
        "addr0_hash".to_string(),
        "addr1_hash".to_string(),
        "addr2_hash".to_string(),
        "addr3_hash".to_string(),
    ];
    let mut addrs: Vec<Addr> = vec![];
    for addr in addr_strs {
        addrs.push(Addr::unchecked(addr.to_string()));
    }
    Addrs { addrs, hashes }
}

pub fn init_lb_pair() -> Result<(App, Contract, DeployedContracts), anyhow::Error> {
    let mut app = App::default();
    let addrs = init_addrs();
    let mut deployed_contracts = DeployedContracts::new();
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(
            parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string())
                .unwrap()
                .timestamp() as u64,
        ),
        chain_id: "chain_id".to_string(),
        random: None,
    });
    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        "SecretScrt",
        "SSCRT",
        6,
        Some(shade_protocol::snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: None,
            enable_redeem: None,
            enable_mint: Some(true),
            enable_burn: None,
            enable_transfer: Some(true),
        }),
    )
    .unwrap();
    let first_contract = deployed_contracts.iter().next().unwrap().1.clone();
    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        "Shade",
        "SHD",
        8,
        Some(shade_protocol::snip20::InitConfig {
            public_total_supply: Some(true),
            enable_deposit: None,
            enable_redeem: None,
            enable_mint: Some(true),
            enable_burn: None,
            enable_transfer: Some(true),
        }),
    )
    .unwrap();
    let second_contract = deployed_contracts.iter().next().unwrap().1.clone();
    let lb_token_stored_code = app.store_code(LbToken::default().contract());
    let admin_contract = init_admin_auth(&mut app, &addrs.admin());

    let lb_pair = lb_pair::init(
        &mut app,
        addrs.admin().as_str(),
        ContractInfo {
            address: Addr::unchecked("factory_address"),
            code_hash: "factory_code_hash".to_string(),
        },
        TokenType::CustomToken {
            contract_addr: first_contract.address,
            token_code_hash: first_contract.code_hash,
        },
        TokenType::CustomToken {
            contract_addr: second_contract.address,
            token_code_hash: second_contract.code_hash,
        },
        10,
        StaticFeeParameters {
            base_factor: 5000,
            filter_period: 30,
            decay_period: 600,
            reduction_factor: 5000,
            variable_fee_control: 40000,
            protocol_share: 1000,
            max_volatility_accumulator: 350000,
        },
        8388608,
        ContractInstantiationInfo {
            id: lb_token_stored_code.code_id,
            code_hash: lb_token_stored_code.code_hash,
        },
        "viewing_key".to_string(),
        String::new(),
        String::new(),
        addrs.admin(),
        admin_contract.into(),
    )?;

    Ok((app, lb_pair, deployed_contracts))
}

pub fn extract_error_msg<T: Any>(error: &StdResult<T>) -> String {
    match error {
        Ok(_response) => panic!("Expected error, but had Ok response"),
        Err(err) => match err {
            StdError::GenericErr { msg, .. } => msg.to_string(),
            _ => panic!("Unexpected error result {:?}", err),
        },
    }
}

pub fn liquidity_parameters_helper(
    deployed_contracts: &DeployedContracts,
    amount_x: Uint128,
    amount_y: Uint128,
) -> StdResult<LiquidityParameters> {
    let array_x: Vec<f64> = vec![
        0.16666666, 0.16666666, 0.16666666, 0.16666666, 0.16666666, 0.16666666, 0.0, 0.0, 0.0, 0.0,
        0.0,
    ];

    let distribution_y: Vec<u64> = array_x
        .clone()
        .into_iter()
        .map(|el| (el * 1e18) as u64)
        .collect();

    let array_y: Vec<f64> = vec![
        0.0, 0.0, 0.0, 0.0, 0.0, 0.16666666, 0.16666666, 0.16666666, 0.16666666, 0.16666666,
        0.16666666,
    ];
    let distribution_x: Vec<u64> = array_y
        .clone()
        .into_iter()
        .map(|el| (el * 1e18) as u64)
        .collect();

    let snip20_1 = deployed_contracts
        .get(&SupportedContracts::Snip20("SSCRT".to_string()))
        .unwrap()
        .clone();
    let snip20_2 = deployed_contracts
        .get(&SupportedContracts::Snip20("SHD".to_string()))
        .unwrap()
        .clone();

    let liquidity_parameters = LiquidityParameters {
        token_x: TokenType::CustomToken {
            contract_addr: snip20_1.address,
            token_code_hash: snip20_1.code_hash,
        },
        token_y: TokenType::CustomToken {
            contract_addr: snip20_2.address,
            token_code_hash: snip20_2.code_hash,
        },
        bin_step: 10,
        amount_x,
        amount_y,
        amount_x_min: amount_x.multiply_ratio(90u128, 100u128),
        amount_y_min: amount_y.multiply_ratio(90u128, 100u128),
        active_id_desired: 8388608,
        id_slippage: 15,
        delta_ids: [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5].into(),
        distribution_x,
        distribution_y,
        deadline: 99999999999,
    };

    Ok(liquidity_parameters)
}

pub fn remove_liquidity_parameters_helper(
    deployed_contracts: &DeployedContracts,
    percentage: u8,
) -> StdResult<(RemoveLiquidity, Vec<(u32, Uint256)>)> {
    let amount_x = Uint128::from(100_000_000u128);
    let amount_y = Uint128::from(100_000_000u128);

    let log_shares_array = lp_tokens_tempate_for_100_sscrts()?;

    // Assuming log_shares_array is your original array
    let mut divided_log_shares_array = Vec::new();

    for (id, amount) in &log_shares_array {
        let divided_amount = amount.multiply_ratio(percentage, 100u8); // Assuming Uint256 supports division
        divided_log_shares_array.push((id.clone(), divided_amount));
    }

    // Now divided_log_shares_array contains the divided amounts

    // Separate IDs and amounts into two vectors
    let ids: Vec<u32> = divided_log_shares_array.iter().map(|&(id, _)| id).collect();
    let amounts: Vec<Uint256> = divided_log_shares_array
        .iter()
        .map(|&(_, amount)| amount)
        .collect();

    let snip20_1 = deployed_contracts
        .get(&SupportedContracts::Snip20("SSCRT".to_string()))
        .unwrap()
        .clone();
    let snip20_2 = deployed_contracts
        .get(&SupportedContracts::Snip20("SHD".to_string()))
        .unwrap()
        .clone();

    let liquidity_parameters = RemoveLiquidity {
        token_x: TokenType::CustomToken {
            contract_addr: snip20_1.address,
            token_code_hash: snip20_1.code_hash,
        },
        token_y: TokenType::CustomToken {
            contract_addr: snip20_2.address,
            token_code_hash: snip20_2.code_hash,
        },
        bin_step: 10,
        amount_x_min: amount_x.multiply_ratio(percentage - 1, 100u128),
        amount_y_min: amount_y.multiply_ratio(percentage - 1, 100u128),
        ids,
        amounts,
        deadline: 99999999999,
    };

    Ok((liquidity_parameters, divided_log_shares_array))
}

pub fn mint_increase_allowance_helper(
    mut app: &mut App,
    deployed_contracts: &DeployedContracts,
    addrs: &Addrs,
    lb_pair_contract_info: &Contract,
) -> StdResult<()> {
    //adding minters and minting

    snip20::add_minters_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        "SHD",
        vec![addrs.admin().to_string()],
    )?;

    // mint token for user1
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        "SHD",
        &vec![],
        addrs.user1().into_string(),
        Uint128::from(1_000_000_000u128),
    )?;

    snip20::add_minters_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        "SSCRT",
        vec![addrs.admin().to_string()],
    )?;

    // mint token for user1
    snip20::mint_exec(
        &mut app,
        addrs.admin().as_str(),
        &deployed_contracts,
        "SSCRT",
        &vec![],
        addrs.user1().into_string(),
        Uint128::from(1_000_000_000u128),
    )?;

    snip20::set_viewing_key_exec(
        &mut app,
        addrs.user1().as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;

    // query balance for token_minted
    let balance = snip20::balance_query(
        &mut app,
        addrs.user1().as_str(),
        &deployed_contracts,
        "SHD",
        "viewing_key".to_owned(),
    )?;

    assert_eq!(balance, Uint128::from(1_000_000_000u128));

    // setting allowance to snip20's
    snip20::set_allowance_exec(
        &mut app,
        addrs.user1().as_str(),
        &deployed_contracts,
        "SHD",
        lb_pair_contract_info.address.to_string(),
        Uint128::MAX,
        None,
    )?;
    snip20::set_allowance_exec(
        &mut app,
        addrs.user1().as_str(),
        &deployed_contracts,
        "SSCRT",
        lb_pair_contract_info.address.to_string(),
        Uint128::MAX,
        None,
    )?;

    Ok(())
}

pub fn lp_tokens_tempate_for_100_sscrts() -> StdResult<[(u32, Uint256); 11]> {
    let log_shares_array: [(u32, Uint256); 11] = [
        (
            8388603,
            Uint256::from_str("5671372555160729777097267814946398569754525696")?,
        ),
        (
            8388604,
            Uint256::from_str("5671372555160729777097267814946398569754525696")?,
        ),
        (
            8388605,
            Uint256::from_str("5671372555160729777097267814946398569754525696")?,
        ),
        (
            8388606,
            Uint256::from_str("5671372555160729777097267814946398569754525696")?,
        ),
        (
            8388607,
            Uint256::from_str("5671372555160729777097267814946398569754525696")?,
        ),
        (
            8388608,
            Uint256::from_str("11342745110321459554194535629892797139509051392")?,
        ),
        (
            8388609,
            Uint256::from_str("5677043927715890506874365082761344968316680222")?,
        ),
        (
            8388610,
            Uint256::from_str("5682720971643606397381239447844106313273880236")?,
        ),
        (
            8388611,
            Uint256::from_str("5688403692615250003778620687291950419593054116")?,
        ),
        (
            8388612,
            Uint256::from_str("5694092096307865253782399307979242370015547170")?,
        ),
        (
            8388613,
            Uint256::from_str("5699786188404173119036181707287221612381479384")?,
        ),
    ];
    return Ok(log_shares_array);
}

pub fn assert_approx_eq_rel(a: u128, b: u128, max_value_delta: u128) {
    // If b is zero, a must also be zero.
    if b == 0 {
        assert_eq!(a, b, "Expected zero but got {}", a);
        return;
    }

    // Calculate the percent difference
    let delta = if a > b { a - b } else { b - a };
    if delta > max_value_delta {
        // Log the error (you could replace these println! statements with actual logging)
        println!("Error: a ~= b not satisfied [uint]");
        println!("    Expected: {}", b);
        println!("      Actual: {}", a);
        println!(" Max  Delta: {}", max_value_delta);
        println!("      Delta: {}", delta);

        // Fail the assertion (you can replace this with custom error handling if desired)
        panic!("Approximate equality check failed.");
    }
}
