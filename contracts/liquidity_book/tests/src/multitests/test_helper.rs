extern crate rand;
use cosmwasm_std::{Addr, BlockInfo, ContractInfo, StdResult, Timestamp, Uint128, Uint256};
use rand::Rng;
use shade_multi_test::{
    interfaces::{
        lb_factory, snip20,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::{lb_pair::LbPair, lb_token::LbToken},
};
use shade_protocol::{
    lb_libraries::{constants::PRECISION, math::u24::U24, tokens::TokenType},
    liquidity_book::lb_pair::LiquidityParameters,
    multi_test::App,
    utils::{asset::Contract, cycle::parse_utc_datetime, MultiTestable},
};

pub const ID_ONE: u32 = 1 << 23;
pub const BASIS_POINT_MAX: u128 = 10_000;

// Avalanche market config for 10bps
pub const DEFAULT_BIN_STEP: u16 = 10;
pub const DEFAULT_BASE_FACTOR: u16 = 5_000;
pub const DEFAULT_FILTER_PERIOD: u16 = 30;
pub const DEFAULT_DECAY_PERIOD: u16 = 600;
pub const DEFAULT_REDUCTION_FACTOR: u16 = 5_000;
pub const DEFAULT_VARIABLE_FEE_CONTROL: u32 = 40_000;
pub const DEFAULT_PROTOCOL_SHARE: u16 = 1_000;
pub const DEFAULT_MAX_VOLATILITY_ACCUMULATOR: u32 = 350_000;
pub const DEFAULT_OPEN_STATE: bool = false;
pub const DEFAULT_FLASHLOAN_FEE: u128 = 800_000_000_000_000;

pub const SHADE: &str = "SHD";
pub const SSCRT: &str = "SSCRT";
pub const SILK: &str = "SILK";
pub const USDC: &str = "USDC";
pub const SBTC: &str = "SBTC";
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
    pub fn batman(&self) -> Addr {
        self.addrs[3].clone()
    }
    pub fn scare_crow(&self) -> Addr {
        self.addrs[4].clone()
    }
    pub fn altaf_bhai(&self) -> Addr {
        self.addrs[5].clone()
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
    let addr_strs = vec!["addr0", "addr1", "addr2", "addr3", "addr4", "addr5"];
    let hashes = vec![
        "addr0_hash".to_string(),
        "addr1_hash".to_string(),
        "addr2_hash".to_string(),
        "addr3_hash".to_string(),
        "addr4_hash".to_string(),
        "addr5_hash".to_string(),
    ];
    let mut addrs: Vec<Addr> = vec![];
    for addr in addr_strs {
        addrs.push(Addr::unchecked(addr.to_string()));
    }
    Addrs { addrs, hashes }
}

pub fn assert_approx_eq_rel(a: Uint256, b: Uint256, delta: Uint256, error_message: &str) {
    let abs_delta = (a).abs_diff(b);
    let percent_delta = abs_delta.multiply_ratio(Uint256::from(10_u128.pow(18)), b);

    if percent_delta > delta {
        panic!(
            "{}: expected delta {:?}, got {:?}",
            error_message, delta, percent_delta
        );
    }
}

pub fn assert_approx_eq_abs(a: Uint256, b: Uint256, delta: Uint256, error_message: &str) {
    let abs_delta = (a).abs_diff(b);
    if abs_delta > delta {
        panic!(
            "{}: expected delta {:?}, got {:?}",
            error_message, delta, abs_delta
        );
    }
}

pub fn setup(bin_step: Option<u16>) -> Result<(App, Contract, DeployedContracts), anyhow::Error> {
    // init snip-20's
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
    //1. Initialize the tokens
    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        SSCRT,
        SSCRT,
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
    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        SHADE,
        SHADE,
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
    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        SILK,
        SILK,
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

    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        USDC,
        USDC,
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
    snip20::init(
        &mut app,
        addrs.admin().as_str(),
        &mut deployed_contracts,
        SBTC,
        SBTC,
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

    //2. init factory
    let lb_factory = lb_factory::init(&mut app, addrs.admin().as_str(), addrs.altaf_bhai(), 0)?;
    let lb_token_stored_code = app.store_code(LbToken::default().contract());
    let lb_pair_stored_code = app.store_code(LbPair::default().contract());

    lb_factory::set_lb_pair_implementation(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        lb_pair_stored_code.code_id,
        lb_pair_stored_code.code_hash,
    )?;

    lb_factory::set_lb_token_implementation(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        lb_token_stored_code.code_id,
        lb_token_stored_code.code_hash,
    )?;

    lb_factory::set_pair_preset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        bin_step.unwrap_or(DEFAULT_BIN_STEP),
        DEFAULT_BASE_FACTOR,
        DEFAULT_FILTER_PERIOD,
        DEFAULT_DECAY_PERIOD,
        DEFAULT_REDUCTION_FACTOR,
        DEFAULT_VARIABLE_FEE_CONTROL,
        DEFAULT_PROTOCOL_SHARE,
        DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        DEFAULT_OPEN_STATE,
    )?;

    // add quote asset
    let shd: ContractInfo = deployed_contracts
        .get(&SupportedContracts::Snip20(SHADE.to_string()))
        .unwrap()
        .clone()
        .into();

    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        TokenType::CustomToken {
            contract_addr: shd.address,
            token_code_hash: shd.code_hash,
        },
    )?;
    let sscrt: ContractInfo = deployed_contracts
        .get(&SupportedContracts::Snip20(SSCRT.to_string()))
        .unwrap()
        .clone()
        .into();
    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        TokenType::CustomToken {
            contract_addr: sscrt.address,
            token_code_hash: sscrt.code_hash,
        },
    )?;

    let silk: ContractInfo = deployed_contracts
        .get(&SupportedContracts::Snip20(SILK.to_string()))
        .unwrap()
        .clone()
        .into();
    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        TokenType::CustomToken {
            contract_addr: silk.address,
            token_code_hash: silk.code_hash,
        },
    )?;

    let usdc: ContractInfo = deployed_contracts
        .get(&SupportedContracts::Snip20(USDC.to_string()))
        .unwrap()
        .clone()
        .into();
    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        TokenType::CustomToken {
            contract_addr: usdc.address,
            token_code_hash: usdc.code_hash,
        },
    )?;

    let sbtc: ContractInfo = deployed_contracts
        .get(&SupportedContracts::Snip20(SBTC.to_string()))
        .unwrap()
        .clone()
        .into();
    lb_factory::add_quote_asset(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        TokenType::CustomToken {
            contract_addr: sbtc.address,
            token_code_hash: sbtc.code_hash,
        },
    )?;

    Ok((app, lb_factory, deployed_contracts))
}

pub fn extract_contract_info(
    deployed_contracts: &DeployedContracts,
    symbol: &str,
) -> StdResult<ContractInfo> {
    Ok(deployed_contracts
        .get(&SupportedContracts::Snip20(symbol.to_string()))
        .unwrap()
        .clone()
        .into())
}

pub fn token_type_snip20_generator(contract: &ContractInfo) -> StdResult<TokenType> {
    Ok(TokenType::CustomToken {
        contract_addr: contract.address.clone(),
        token_code_hash: contract.code_hash.clone(),
    })
}
pub fn token_type_native_generator(denom: String) -> StdResult<TokenType> {
    Ok(TokenType::NativeToken { denom })
}

fn safe64_divide(numerator: u128, denominator: u64) -> u64 {
    (numerator / denominator as u128) as u64
}

pub fn get_id(active_id: u32, i: u32, nb_bin_y: u8) -> u32 {
    let mut id: u32 = active_id + i;

    if nb_bin_y > 0 {
        id = id - nb_bin_y as u32 + 1;
    };

    safe24(id)
}

pub fn get_total_bins(nb_bin_x: u8, nb_bin_y: u8) -> u8 {
    if nb_bin_x > 0 && nb_bin_y > 0 {
        return nb_bin_x + nb_bin_y - 1; // Convert to u256
    }
    nb_bin_x + nb_bin_y
}

// Placeholder function for safe24
// Ensure the value fits into 24 bits.
fn safe24(value: u32) -> u32 {
    if value >= (1 << 24) {
        panic!("Value too large for 24 bits");
    }
    value
}

// Utility function to bound a value within a range [min, max]
pub fn bound<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

pub fn generate_random<T>(min: T, max: T) -> T
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd,
{
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

pub fn liquidity_parameters_generator(
    // Assuming lbPair has methods to get tokenX and tokenY
    // lbPair: &LBPair,
    _deployed_contracts: &DeployedContracts,
    active_id: u32,
    token_x: ContractInfo,
    token_y: ContractInfo,
    amount_x: Uint128,
    amount_y: Uint128,
    nb_bins_x: u8,
    nb_bins_y: u8,
) -> StdResult<LiquidityParameters> {
    let total = get_total_bins(nb_bins_x, nb_bins_y);

    if active_id > U24::MAX {
        panic!("active_id too big");
    }

    let mut distribution_x: Vec<u64> = Vec::new();
    let mut distribution_y: Vec<u64> = Vec::new();

    let mut delta_ids = Vec::new();

    for i in 0..total {
        if nb_bins_y > 0 {
            delta_ids.push(i as i64 - nb_bins_y as i64 + 1_i64);
        } else {
            delta_ids.push(i as i64);
        }
        let id = get_id(active_id, i.into(), nb_bins_y);
        let distrib_x = if id >= active_id && nb_bins_x > 0 {
            safe64_divide(PRECISION, nb_bins_x as u64)
        } else {
            0
        };

        distribution_x.push(distrib_x);

        let distrib_y = if id <= active_id && nb_bins_y > 0 {
            safe64_divide(PRECISION, nb_bins_y as u64)
        } else {
            0
        };
        distribution_y.push(distrib_y);
    }

    let liquidity_parameters = LiquidityParameters {
        token_x: TokenType::CustomToken {
            contract_addr: token_x.address,
            token_code_hash: token_x.code_hash,
        },
        token_y: TokenType::CustomToken {
            contract_addr: token_y.address,
            token_code_hash: token_y.code_hash,
        },
        bin_step: DEFAULT_BIN_STEP,
        amount_x,
        amount_y,
        amount_x_min: amount_x.multiply_ratio(90u128, 100u128),
        amount_y_min: amount_y.multiply_ratio(90u128, 100u128),
        active_id_desired: active_id,
        id_slippage: 15,
        delta_ids,
        distribution_x,
        distribution_y,
        deadline: 99999999999,
    };

    Ok(liquidity_parameters)
}

pub fn liquidity_parameters_generator_with_native(
    // Assuming lbPair has methods to get tokenX and tokenY
    // lbPair: &LBPair,
    _deployed_contracts: &DeployedContracts,
    active_id: u32,
    token_x: TokenType,
    token_y: TokenType,
    amount_x: Uint128,
    amount_y: Uint128,
    nb_bins_x: u8,
    nb_bins_y: u8,
) -> StdResult<LiquidityParameters> {
    let total = get_total_bins(nb_bins_x, nb_bins_y);

    if active_id > U24::MAX {
        panic!("active_id too big");
    }

    let mut distribution_x: Vec<u64> = Vec::new();
    let mut distribution_y: Vec<u64> = Vec::new();

    let mut delta_ids = Vec::new();

    for i in 0..total {
        if nb_bins_y > 0 {
            delta_ids.push(i as i64 - nb_bins_y as i64 + 1_i64);
        } else {
            delta_ids.push(i as i64);
        }
        let id = get_id(active_id, i.into(), nb_bins_y);
        let distrib_x = if id >= active_id && nb_bins_x > 0 {
            safe64_divide(PRECISION, nb_bins_x as u64)
        } else {
            0
        };

        distribution_x.push(distrib_x);

        let distrib_y = if id <= active_id && nb_bins_y > 0 {
            safe64_divide(PRECISION, nb_bins_y as u64)
        } else {
            0
        };
        distribution_y.push(distrib_y);
    }

    let token_x_temp;
    let token_y_temp;

    if token_x.is_native_token() {
        token_x_temp = TokenType::NativeToken {
            denom: token_x.unique_key(),
        }
    } else {
        token_x_temp = TokenType::CustomToken {
            contract_addr: token_x.address(),
            token_code_hash: token_x.code_hash(),
        }
    }

    if token_y.is_native_token() {
        token_y_temp = TokenType::NativeToken {
            denom: token_y.unique_key(),
        }
    } else {
        token_y_temp = TokenType::CustomToken {
            contract_addr: token_y.address(),
            token_code_hash: token_y.code_hash(),
        }
    }

    let liquidity_parameters = LiquidityParameters {
        token_x: token_x_temp,
        token_y: token_y_temp,
        bin_step: DEFAULT_BIN_STEP,
        amount_x,
        amount_y,
        amount_x_min: amount_x.multiply_ratio(90u128, 100u128),
        amount_y_min: amount_y.multiply_ratio(90u128, 100u128),
        active_id_desired: active_id,
        id_slippage: 15,
        delta_ids,
        distribution_x,
        distribution_y,
        deadline: 99999999999,
    };

    Ok(liquidity_parameters)
}

// pub fn mint_increase_allowance_helper(
//     mut app: &mut App,
//     deployed_contracts: &DeployedContracts,
//     addrs: &Addrs,
//     lb_pair_contract_info: &Contract,
// ) -> StdResult<()> {
//     //adding minters and minting

//     snip20::add_minters_exec(
//         &mut app,
//         addrs.admin().as_str(),
//         &deployed_contracts,
//         SSCRT,
//         vec![addrs.admin().to_string()],
//     )?;

//     snip20::mint_exec(
//         &mut app,
//         addrs.admin().as_str(),
//         &deployed_contracts,
//         SSCRT,
//         &vec![],
//         addrs.user1().into_string(),
//         Uint128::from(1_000_000_000u128),
//     )?;

//     snip20::add_minters_exec(
//         &mut app,
//         addrs.admin().as_str(),
//         &deployed_contracts,
//         SHADE,
//         vec![addrs.admin().to_string()],
//     )?;

//     // mint token for user1
//     snip20::mint_exec(
//         &mut app,
//         addrs.admin().as_str(),
//         &deployed_contracts,
//         SHADE,
//         &vec![],
//         addrs.user1().into_string(),
//         Uint128::from(1_000_000_000u128),
//     )?;

//     snip20::set_viewing_key_exec(
//         &mut app,
//         addrs.user1().as_str(),
//         &deployed_contracts,
//         SHADE,
//         "viewing_key".to_owned(),
//     )?;

//     // query balance for token_minted
//     let balance = snip20::balance_query(
//         &mut app,
//         addrs.user1().as_str(),
//         &deployed_contracts,
//         SHADE,
//         "viewing_key".to_owned(),
//     )?;

//     assert_eq!(balance, Uint128::from(1_000_000_000u128));

//     // setting allowance to snip20's
//     snip20::set_allowance_exec(
//         &mut app,
//         addrs.user1().as_str(),
//         &deployed_contracts,
//         SSCRT,
//         lb_pair_contract_info.address.to_string(),
//         Uint128::MAX,
//         None,
//     )?;
//     snip20::set_allowance_exec(
//         &mut app,
//         addrs.user1().as_str(),
//         &deployed_contracts,
//         SHADE,
//         lb_pair_contract_info.address.to_string(),
//         Uint128::MAX,
//         None,
//     )?;
//     snip20::set_allowance_exec(
//         &mut app,
//         addrs.user1().as_str(),
//         &deployed_contracts,
//         SILK,
//         lb_pair_contract_info.address.to_string(),
//         Uint128::MAX,
//         None,
//     )?;
//     snip20::set_allowance_exec(
//         &mut app,
//         addrs.user1().as_str(),
//         &deployed_contracts,
//         USDC,
//         lb_pair_contract_info.address.to_string(),
//         Uint128::MAX,
//         None,
//     )?;
//     snip20::set_allowance_exec(
//         &mut app,
//         addrs.user1().as_str(),
//         &deployed_contracts,
//         SBTC,
//         lb_pair_contract_info.address.to_string(),
//         Uint128::MAX,
//         None,
//     )?;
//     Ok(())
// }

pub fn mint_token_helper(
    app: &mut App,
    deployed_contracts: &DeployedContracts,
    addrs: &Addrs,
    user: String,
    tokens_to_mint: Vec<(&str, Uint128)>,
) -> StdResult<()> {
    let admin = &addrs.admin().to_string();

    // Adding minters and minting for SSCRT and SHADE
    for (token, amount) in tokens_to_mint {
        snip20::add_minters_exec(
            app,
            admin,
            deployed_contracts,
            token,
            vec![admin.to_string()],
        )?;
        snip20::mint_exec(
            app,
            admin,
            deployed_contracts,
            token,
            &vec![],
            user.clone(),
            amount,
        )?;
        snip20::set_viewing_key_exec(
            app,
            &user.clone(),
            deployed_contracts,
            token,
            "viewing_key".to_owned(),
        )?;
    }

    Ok(())
}

pub fn increase_allowance_helper(
    app: &mut App,
    deployed_contracts: &DeployedContracts,
    sender: String,
    spender: String,
    tokens_to_mint: Vec<(&str, Uint128)>,
) -> StdResult<()> {
    for (token, _) in tokens_to_mint {
        snip20::set_allowance_exec(
            app,
            &sender.clone(),
            deployed_contracts,
            token,
            spender.clone(),
            Uint128::MAX,
            None,
        )?;
    }

    Ok(())
}
