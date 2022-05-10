use query_authentication::viewing_keys::ViewingKey;
use secretcli::{cli_types::NetContract, secretcli::{account_address, init, handle, query, Report, start_loaded_local_testnet, enter_test_container, create_permit}};
use cosmwasm_std::{to_binary, Binary, HumanAddr, Uint128};
use network_integration::{
    contract_helpers::{
        minter::{get_balance},
    },
    utils::{
        generate_label, print_contract, print_header, ACCOUNT_KEY,
        BONDS_FILE, GAS, VIEW_KEY, SNIP20_FILE, STORE_GAS, MOCK_BAND_FILE, ORACLE_FILE,
    },
};
use query_authentication::{permit::Permit, transaction::PermitSignature};
use query_authentication::transaction::PubKey;
use serde::Serialize;
use serde_json::Result;
use shade_protocol::snip20::{self, InitMsg, InitialBalance, InitConfig};
use shade_protocol::bonds::{self, FillerMsg, AccountPermitMsg};
use shade_protocol::band::{self};
use mock_band::contract::*;
use shade_protocol::oracle::{self, InitMsg as OracleInitMsg};
use shade_protocol::utils::asset::Contract;
use std::{io::{self, Write, Repeat}, borrow::Borrow};

pub const ADMIN_KEY: &str = "b";
pub const LIMIT_ADMIN_KEY: &str = "c";

fn setup_contracts(
    global_issuance_limit: Uint128,
    global_minimum_bonding_period: u64,
    global_maximum_discount: Uint128,
    activated: bool,
    minting_bond: bool,
    bond_issuance_period: u64,
    discount: Uint128,
    bond_issuance_limit: Uint128,
    bonding_period: u64,
    reports: &mut Vec<Report>,
) -> Result<(NetContract, NetContract, NetContract, NetContract, NetContract)> {
    println!("Starting setup of account_addresses");
    io::stdout().flush();
    let account_a = account_address(ACCOUNT_KEY)?;
    //println!("Completed a");
    //io::stdout().flush();
    let account_admin = account_address(ADMIN_KEY)?;
    let account_limit_admin = account_address(LIMIT_ADMIN_KEY)?;

    print_header("Set up account_addresses");
    print_header("Initializing snip20s");
    let mint_snip_init_msg = snip20::InitMsg {
        name: "test_mint".to_string(),
        admin: None,
        symbol: "MINT".to_string(),
        decimals: 6,
        initial_balances: None,
        prng_seed: Default::default(),
        config: Some(InitConfig{
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
    };

    print_header("Mint snip init");
    let mint_snip = init(
        &mint_snip_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    print_header("Mint snip initiated");

    let collat_snip_init_msg = snip20::InitMsg {
        name: "test_collat".to_string(),
        admin: None,
        symbol: "COLL".to_string(),
        decimals: 6,
        initial_balances: Some(vec![InitialBalance {
            address: HumanAddr::from(account_a.clone()),
            amount: Uint128(1_000_000_000_000_000),
        }]),
        prng_seed: Default::default(),
        config: Some(InitConfig{
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
    };

    print_header("Collateral snip init");
    let collateral_snip = init(
        &collat_snip_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    print_header("Collateral snip initiated");
    print_header("Initiating mockband and oracle");

    let mockband_init_msg = band::InitMsg{};

    let mockband = init(
        &mockband_init_msg,
        MOCK_BAND_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    print_header("Mockband initiated");

    let oracle_init_msg = oracle::InitMsg{
        admin: Some(HumanAddr::from(account_limit_admin.clone())),
        band: Contract{ address: HumanAddr::from(mockband.address.clone()), code_hash: mockband.code_hash.clone()},
        sscrt: Contract { address: HumanAddr::from(""), code_hash: "".to_string() }
    };

    let oracle = init(
        &oracle_init_msg,
        ORACLE_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    print_header("Oracle Initiated");

    let bonds_init_msg = bonds::InitMsg{
        limit_admin: HumanAddr::from(account_limit_admin.clone()),
        global_issuance_limit,
        global_minimum_bonding_period,
        global_maximum_discount,
        admin: HumanAddr::from(account_admin.clone()),
        oracle: Contract {
            address: HumanAddr::from(oracle.address.clone()),
            code_hash: oracle.code_hash.clone(),
        },
        treasury: HumanAddr::from(account_admin),
        issued_asset: Contract {
            address: HumanAddr::from(mint_snip.address.clone()),
            //address: HumanAddr::from("hehe"),
            code_hash: mint_snip.code_hash.clone(),
            //code_hash: "hehe".to_string(),
        },
        activated,
        minting_bond,
        bond_issuance_limit,
        bonding_period,
        discount,
        global_minimum_issued_price: Uint128(1),
        allowance_key_entropy: Some(VIEW_KEY.to_string()),
    };

    let bonds = init(
        &bonds_init_msg,
        BONDS_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    let msg = snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None };

    handle(&msg, &mint_snip, ACCOUNT_KEY, Some(GAS), Some("test"), None, reports, None)?;
    handle(&msg, &collateral_snip, ACCOUNT_KEY, Some(GAS), Some("test"), None, reports, None)?;

    Ok((bonds, mint_snip, collateral_snip, mockband, oracle))
}

fn setup_contracts_allowance(
    global_issuance_limit: Uint128,
    global_minimum_bonding_period: u64,
    global_maximum_discount: Uint128,
    activated: bool,
    minting_bond: bool,
    bond_issuance_period: u64,
    discount: Uint128,
    bond_issuance_limit: Uint128,
    bonding_period: u64,
    reports: &mut Vec<Report>,
) -> Result<(NetContract, NetContract, NetContract, NetContract, NetContract)> {
    println!("Starting setup of account_addresses");
    io::stdout().flush();
    let account_a = account_address(ACCOUNT_KEY)?;
    //println!("Completed a");
    //io::stdout().flush();
    let account_admin = account_address(ADMIN_KEY)?;
    let account_limit_admin = account_address(LIMIT_ADMIN_KEY)?;

    print_header("Set up account_addresses");
    print_header("Initializing snip20s");
    let issued_snip_init_msg = snip20::InitMsg {
        name: "test_issue".to_string(),
        admin: None,
        symbol: "ISSU".to_string(),
        decimals: 6,
        initial_balances: Some(vec![InitialBalance {
            address: HumanAddr::from(account_admin.clone()),
            amount: Uint128(1_000_000_000_000_000),
        }]),
        prng_seed: Default::default(),
        config: Some(InitConfig{
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
    };

    print_header("Mint snip init");
    let issued_snip = init(
        &issued_snip_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    print_header("Issued snip initiated");

    let collat_snip_init_msg = snip20::InitMsg {
        name: "test_collat".to_string(),
        admin: None,
        symbol: "COLL".to_string(),
        decimals: 6,
        initial_balances: Some(vec![InitialBalance {
            address: HumanAddr::from(account_a.clone()),
            amount: Uint128(1_000_000_000_000_000),
        }]),
        prng_seed: Default::default(),
        config: Some(InitConfig{
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
    };

    print_header("Collateral snip init");
    let collateral_snip = init(
        &collat_snip_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    print_header("Collateral snip initiated");
    print_header("Initiating mockband and oracle");

    let mockband_init_msg = band::InitMsg{};

    let mockband = init(
        &mockband_init_msg,
        MOCK_BAND_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    print_header("Mockband initiated");

    let oracle_init_msg = oracle::InitMsg{
        admin: Some(HumanAddr::from(account_limit_admin.clone())),
        band: Contract{ address: HumanAddr::from(mockband.address.clone()), code_hash: mockband.code_hash.clone()},
        sscrt: Contract { address: HumanAddr::from(""), code_hash: "".to_string() }
    };

    let oracle = init(
        &oracle_init_msg,
        ORACLE_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    print_header("Oracle Initiated");

    let bonds_init_msg = bonds::InitMsg{
        limit_admin: HumanAddr::from(account_limit_admin.clone()),
        global_issuance_limit,
        global_minimum_bonding_period,
        global_maximum_discount,
        admin: HumanAddr::from(account_admin.clone()),
        oracle: Contract {
            address: HumanAddr::from(oracle.address.clone()),
            code_hash: oracle.code_hash.clone(),
        },
        treasury: HumanAddr::from(account_admin),
        issued_asset: Contract {
            address: HumanAddr::from(issued_snip.address.clone()),
            //address: HumanAddr::from("hehe"),
            code_hash: issued_snip.code_hash.clone(),
            //code_hash: "hehe".to_string(),
        },
        activated,
        minting_bond,
        bond_issuance_limit,
        bonding_period,
        discount,
        global_minimum_issued_price: Uint128(1),
        allowance_key_entropy: Some(VIEW_KEY.to_string().clone()),
    };

    let bonds = init(
        &bonds_init_msg,
        BONDS_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    let msg = snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None };

    handle(&msg, &issued_snip, ACCOUNT_KEY, Some(GAS), Some("test"), None, reports, None)?;
    handle(&msg, &collateral_snip, ACCOUNT_KEY, Some(GAS), Some("test"), None, reports, None)?;

    Ok((bonds, issued_snip, collateral_snip, mockband, oracle))
}

fn setup_additional_snip20_with_vk(
    name: String,
    symbol: String,
    decimals: u8,
    reports: &mut Vec<Report>,
) -> Result<NetContract> {
    let account_a = account_address(ACCOUNT_KEY)?;
    let snip_init_msg = snip20::InitMsg {
        name: name,
        admin: None,
        symbol: symbol,
        decimals: decimals,
        initial_balances: Some(vec![InitialBalance {
            address: HumanAddr::from(account_a.clone()),
            amount: Uint128(1_000_000_000_000_000),
        }]),
        prng_seed: Default::default(),
        config: Some(InitConfig{
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
    };

    print_header("Additional snip init");
    let new_snip = init(
        &snip_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports,
    )?;

    let snip_msg = snip20::HandleMsg::SetViewingKey { key: VIEW_KEY.to_string(), padding: None };

    let snip_tx_info = handle(
        &snip_msg,
        &new_snip,
        ADMIN_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", snip_tx_info.gas_used);

    
    Ok(new_snip)
}

fn open_bond(
    collat_snip: &NetContract,
    now: u64,
    end: u64,
    opp_limit: Option<Uint128>,
    period: Option<u64>,
    disc: Option<Uint128>,
    max_collateral_price: Uint128,
    reports: &mut Vec<Report>,
    bonds: &NetContract,
) -> Result<()> {

    let msg = bonds::HandleMsg::OpenBond { 
        collateral_asset: Contract { address: HumanAddr::from(collat_snip.address.clone()), code_hash: collat_snip.code_hash.clone() },
        start_time: now,
        end_time: end, 
        bond_issuance_limit: opp_limit,
        bonding_period: period,
        discount: disc,
        max_accepted_collateral_price: max_collateral_price,
        err_collateral_price: Uint128(10000000000000000)
    };

    let tx_info = handle(
        &msg,
        bonds,
        ADMIN_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", tx_info.gas_used);

    Ok(())
}

fn update_bonds_config(
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
    bonds: &NetContract,
    reports: &mut Vec<Report>,
) -> Result<()> {
    let msg = bonds::HandleMsg::UpdateConfig { 
        admin,
        oracle,
        treasury, 
        issued_asset, 
        activated,
        minting_bond,
        bond_issuance_limit, 
        bonding_period,
        discount,
        global_minimum_issued_price,
        allowance_key
    };

    let tx_info = handle(
        &msg,
        bonds,
        ADMIN_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", tx_info.gas_used);

    Ok(())
}

fn update_bonds_limit_config(
    limit_admin: Option<HumanAddr>,
    global_issuance_limit: Option<Uint128>,
    global_minimum_bonding_period: Option<u64>,
    global_maximum_discount: Option<Uint128>,
    reset_total_issued: Option<bool>,
    reset_total_claimed: Option<bool>,
    bonds: &NetContract,
    reports: &mut Vec<Report>,
) -> Result<()> {
    let msg = bonds::HandleMsg::UpdateLimitConfig { 
        limit_admin,
        global_issuance_limit,
        global_minimum_bonding_period,
        global_maximum_discount,
        reset_total_issued,
        reset_total_claimed
    };

    let tx_info = handle(
        &msg,
        bonds,
        ADMIN_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", tx_info.gas_used);

    Ok(())
}

fn close_bond(
    collat_snip: &NetContract,
    bonds: &NetContract,
    reports: &mut Vec<Report>,
) -> Result<()> {
    let msg = bonds::HandleMsg::CloseBond { 
        collateral_asset: Contract { 
            address: HumanAddr::from(collat_snip.address.clone()), 
            code_hash: collat_snip.code_hash.clone() }
    };

    let tx_info = handle(
        &msg,
        bonds,
        ADMIN_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", tx_info.gas_used);

    Ok(())
}

fn buy_bond(
    collat_snip: &NetContract,
    amount: Uint128,
    reports: &mut Vec<Report>,
    bonds: &NetContract,
) -> Result<()> {
    let msg = snip20::HandleMsg::Send { 
        recipient: HumanAddr::from(bonds.address.clone()), 
        amount: amount, 
        msg: None, 
        memo: None, 
        padding: None 
    }; 

    let tx_info = handle(
        &msg,
        collat_snip,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", tx_info.gas_used);

    Ok(())
}

fn claim_bond(
    bonds: &NetContract,
    reports: &mut Vec<Report>,
) -> Result<()> {
    let msg = bonds::HandleMsg::Claim {  };
    
    let tx_info = handle(
        &msg,
        bonds,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", tx_info.gas_used);
    print_header("Opportunity claim attempted");

    Ok(())
}

fn print_bond_opps(
    bonds: &NetContract,
    reports: &mut Vec<Report>,
) -> Result<()> {
    let bond_opp_quer_msg = bonds::QueryMsg::BondOpportunities {  };
    let opp_query: bonds::QueryAnswer = query(&bonds, bond_opp_quer_msg, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query
    {
        let opp_iter = bond_opportunities.iter();
        for bond in opp_iter{
            println!("\nBond opp: {}\n Starts: {}\n Ends: {}\n Bonding period: {}\n Discount: {}\n Amount Available: {}\n", 
            bond.deposit_denom.token_info.symbol,
            bond.start_time,
            bond.end_time,
            bond.bonding_period,
            bond.discount,
            (bond.issuance_limit - bond.amount_issued).unwrap(),
        )
        }
        
    }

    Ok(())
}

fn print_pending_bonds(
    bonds: &NetContract,
    reports: &mut Vec<Report>,
) -> Result<()> {
    // Create permit
    let account_permit = create_signed_permit(
        AccountPermitMsg {
            contract: HumanAddr(bonds.address.clone()),
            key: "key".to_string(),
        },
        None,
        None,
        ACCOUNT_KEY,
    );

    let account_quer_msg = bonds::QueryMsg::Account { permit: account_permit  };
    let account_query: bonds::QueryAnswer = query(&bonds, account_quer_msg, None)?;
    
    if let bonds::QueryAnswer::Account { pending_bonds, 
    } = account_query
    {
        let pend_iter = pending_bonds.iter();
        for pending in pend_iter{
            println!("\nBond opp: {}\n Ends: {}\n Deposit Amount: {}\n Deposit Price: {}\n Claim Amount: {}\n Claim Price: {}\n Discount: {}\n Discount Price: {}", 
            pending.deposit_denom.token_info.symbol,
            pending.end,
            pending.deposit_amount,
            pending.deposit_price,
            pending.claim_amount,
            pending.claim_price,
            pending.discount,
            pending.discount_price,
        )
        }
    }
    
    Ok(())
}

fn set_viewing_keys(
    key: String,
    reports: &mut Vec<Report>,
    bonds: &NetContract,
    issued_snip20: &NetContract,
    collat_snip20: &NetContract,
) -> Result<()> {

    // let msg = bonds::HandleMsg::SetViewingKey { 
    //     key: key.clone(),
    // };

    // let tx_info = handle(
    //     &msg,
    //     bonds,
    //     ACCOUNT_KEY,
    //     Some(GAS),
    //     Some("test"),
    //     None,
    //     reports,
    //     None,
    // )?.1;

    // println!("Gas used: {}", tx_info.gas_used);

    let issued_snip_msg = snip20::HandleMsg::SetViewingKey { key: key.clone(), padding: None };

    let issued_snip_tx_info = handle(
        &issued_snip_msg,
        issued_snip20,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", issued_snip_tx_info.gas_used);

    let collat_snip_msg = snip20::HandleMsg::SetViewingKey { key: key, padding: None };

    let collat_snip_tx_info = handle(
        &collat_snip_msg,
        collat_snip20,
        ADMIN_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", collat_snip_tx_info.gas_used);

    Ok(())
}

// struct Key {pub key: String,}

// impl ToString for Key {
//     fn to_string(&self) -> String {
//         self.0.clone()
//     }
// }

// impl ViewingKey<32> for Key{}

fn set_band_prices(
    collat_snip: &NetContract,
    issued_snip: &NetContract,
    coll_price: Uint128,
    issued_price: Uint128,
    reports: &mut Vec<Report>,
    band: &NetContract
) -> Result<()> {
    let coll_msg = mock_band::contract::HandleMsg::MockPrice { 
        symbol: "COLL".to_string(), 
        price: coll_price
    };

    let coll_tx_info = handle(
        &coll_msg,
        band,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", coll_tx_info.gas_used);

    let issued_msg = mock_band::contract::HandleMsg::MockPrice { 
        symbol: "ISSU".to_string(), 
        price: issued_price
    };

    let issued_tx_info = handle(
        &issued_msg,
        band,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;
    
    println!("Gas used: {}", issued_tx_info.gas_used);

    Ok(())
}

fn set_additional_band_price(
    new_snip: &NetContract,
    new_price: Uint128,
    new_symbol: String,
    band: &NetContract,
    reports: &mut Vec<Report>,
) -> Result<()> {
    let msg = mock_band::contract::HandleMsg::MockPrice { 
        symbol: new_symbol,
        price: new_price 
    };

    let tx_info = handle(
        &msg, 
        band,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None
    )?.1;

    println!("Gas used: {}", tx_info.gas_used);

    Ok(())
}

fn set_minting_privileges(
    mint_snip20: &NetContract,
    bonds: &NetContract,
    reports: &mut Vec<Report>,
) -> Result<()> {
    let msg = snip20::HandleMsg::SetMinters { minters: vec![HumanAddr::from(bonds.address.clone())], padding: None };

    print_header("Trying to set");
    let tx_info = handle(
        &msg,
        mint_snip20,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;
    
    println!("Gas used: {}", tx_info.gas_used);
    
    Ok(())
}

fn increase_allowance(
    bonds: &NetContract,
    issued_snip: &NetContract,
    amount: Uint128,
    reports: &mut Vec<Report>,
) -> Result<()> {
    let account_admin = account_address(ADMIN_KEY)?;
    let allowance_snip_msg = snip20::HandleMsg::IncreaseAllowance { owner: HumanAddr::from(account_admin.clone()), spender: HumanAddr::from(bonds.address.clone()), amount: amount };

    let allowance_snip_tx_info = handle(
        &allowance_snip_msg,
        &issued_snip,
        ADMIN_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None,
    )?.1;

    println!("Gas used: {}", allowance_snip_tx_info.gas_used);

    Ok(())
}

fn create_signed_permit<T: Clone + Serialize>(
    params: T,
    memo: Option<String>,
    msg_type: Option<String>,
    signer: &str,
) -> Permit<T> {
    let mut permit = Permit {
        params,
        signature: PermitSignature {
            pub_key: PubKey {
                r#type: "".to_string(),
                value: Default::default(),
            },
            signature: Default::default(),
        },
        account_number: None,
        chain_id: Some("testnet".to_string()),
        sequence: None,
        memo,
    };

    let unsigned_msg = permit.create_signed_tx(msg_type);

    let signed_info = create_permit(unsigned_msg, signer).unwrap();

    permit.signature = PermitSignature {
        pub_key: query_authentication::transaction::PubKey {
            r#type: signed_info.pub_key.msg_type,
            value: Binary::from_base64(&signed_info.pub_key.value).unwrap(),
        },
        signature: Binary::from_base64(&signed_info.signature).unwrap(),
    };

    permit
}

#[test]
fn run_bonds_singular() -> Result<()> {
    let account_a = account_address(ACCOUNT_KEY)?;
    let account_admin = account_address(ADMIN_KEY)?;
    let mut reports = vec![];

    let now = chrono::offset::Utc::now().timestamp() as u64;
    let end = now + 600u64;
    print_header("Initializing bonds and snip20");
    println!("Printed header");
    let (bonds, mint_snip, collateral_snip, mockband, oracle) = setup_contracts(
        Uint128(100_000_000_000), 
        1u64, 
        Uint128(7_000_000_000_000_000_000), 
        true, 
        true, 
        240, 
        Uint128(6), 
        Uint128(100_000_000), 
        130, 
        &mut reports,
    )?;

    print_contract(&mint_snip);
    print_contract(&collateral_snip);
    print_contract(&bonds);
    print_contract(&mockband);
    print_contract(&oracle);

    set_band_prices(&collateral_snip, &mint_snip, Uint128(5_000_000_000_000_000_000), Uint128(2_000_000_000_000_000_000), &mut reports, &mockband)?;
    print_header("Band prices set");

    set_minting_privileges(&mint_snip, &bonds, &mut reports)?;
    print_header("Minting privileges set");

    print_header("Asserting");
    assert_eq!(Uint128(0), get_balance(&mint_snip, account_a.clone()));
    print_header("Done asserting");

    // Open bond opportunity
    let opp_limit = Uint128(100_000_000_000);
    let period = 1u64;
    let disc = Uint128(6_000_000_000_000_000_000);
    open_bond(&collateral_snip, now, end, Some(opp_limit), Some(period), Some(disc), Uint128(100_000_000_000_000_000_000), &mut reports, &bonds)?;
    print_header("Bond Opened");

    let g_issued_query_msg = bonds::QueryMsg::BondInfo {  };
    let g_issued_query: bonds::QueryAnswer = query(&bonds, g_issued_query_msg, None)?;
    if let bonds::QueryAnswer::BondInfo { global_total_issued, global_total_claimed, issued_asset
    } = g_issued_query
    {
        assert_eq!(global_total_issued, Uint128(100_000_000_000));
        assert_eq!(global_total_claimed, Uint128(0));
    }

    let bond_opp_quer_msg = bonds::QueryMsg::BondOpportunities {  };
    let opp_query: bonds::QueryAnswer = query(&bonds, bond_opp_quer_msg, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query
    {
        assert_eq!(bond_opportunities[0].amount_issued, Uint128(0));
        assert_eq!(bond_opportunities[0].bonding_period, 1);
        assert_eq!(bond_opportunities[0].discount, disc);
        println!("\nBond opp: {}\n Starts: {}\n Ends: {}\n Bonding period: {}\n Discount: {}\n Amount Available: {}\n", 
        bond_opportunities[0].deposit_denom.token_info.symbol,
        bond_opportunities[0].start_time,
        bond_opportunities[0].end_time,
        bond_opportunities[0].bonding_period,
        bond_opportunities[0].discount,
        (bond_opportunities[0].issuance_limit - bond_opportunities[0].amount_issued).unwrap(),
    )
    }

    buy_bond(&collateral_snip, Uint128(100_000_000), &mut reports, &bonds)?;
    print_header("Bond opp bought");
    set_viewing_keys(VIEW_KEY.to_string(), &mut reports, &bonds, &mint_snip, &collateral_snip)?;

    // Create permit
    let account_permit = create_signed_permit(
        AccountPermitMsg {
            contract: HumanAddr(bonds.address.clone()),
            key: "key".to_string(),
        },
        None,
        None,
        ACCOUNT_KEY,
    );

    let account_quer_msg = bonds::QueryMsg::Account { permit: account_permit };
    let account_query: bonds::QueryAnswer = query(&bonds, account_quer_msg, None)?;
    
    if let bonds::QueryAnswer::Account { pending_bonds, 
    } = account_query
    {
        assert_eq!(pending_bonds[0].deposit_amount, Uint128(100_000_000));
        assert_eq!(pending_bonds[0].claim_amount, Uint128(265_957_446));
        assert_eq!(pending_bonds[0].deposit_denom.token_info.symbol, "COLL".to_string());
        println!("\nBond opp: {}\n Ends: {}\n Deposit Amount: {}\n Deposit Price: {}\n Claim Amount: {}\n Claim Price: {}\n Discount: {}\n Discount Price: {}", 
            pending_bonds[0].deposit_denom.token_info.symbol,
            pending_bonds[0].end,
            pending_bonds[0].deposit_amount,
            pending_bonds[0].deposit_price,
            pending_bonds[0].claim_amount,
            pending_bonds[0].claim_price,
            pending_bonds[0].discount,
            pending_bonds[0].discount_price,
        )
    }

    claim_bond(&bonds, &mut reports)?;


    let bond_opp_query_msg_2 = bonds::QueryMsg::BondOpportunities {  };
    let opp_query_2: bonds::QueryAnswer = query(&bonds, bond_opp_query_msg_2, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query_2
    {
        assert_eq!(bond_opportunities[0].amount_issued, Uint128(265_957_446));
        assert_eq!(bond_opportunities[0].bonding_period, 1);
        assert_eq!(bond_opportunities[0].discount, disc);
        println!("\nBond opp: {}\n Starts: {}\n Ends: {}\n Bonding period: {}\n Discount: {}\n Amount Available: {}\n", 
        bond_opportunities[0].deposit_denom.token_info.symbol,
        bond_opportunities[0].start_time,
        bond_opportunities[0].end_time,
        bond_opportunities[0].bonding_period,
        bond_opportunities[0].discount,
        (bond_opportunities[0].issuance_limit - bond_opportunities[0].amount_issued).unwrap(),
    )
    }

    let issued_snip_query_msg = snip20::QueryMsg::Balance { address: HumanAddr::from(account_a), key: VIEW_KEY.to_string() };
    let issued_snip_query: snip20::QueryAnswer = query(&mint_snip, issued_snip_query_msg, None)?;

    if let snip20::QueryAnswer::Balance { amount, 
    } = issued_snip_query
    {
        println!("Account A Current MINT Balance: {}\n", amount);
        assert_eq!(amount, Uint128(265_957_446));
        io::stdout().flush().unwrap();
    }

    let collat_snip_query_msg = snip20::QueryMsg::Balance { address: HumanAddr::from(account_admin), key: VIEW_KEY.to_string() };
    let collat_snip_query: snip20::QueryAnswer = query(&collateral_snip, collat_snip_query_msg, None)?;

    if let snip20::QueryAnswer::Balance { amount, 
    } = collat_snip_query
    {
        assert_eq!(amount, Uint128(100_000_000));
        println!("Account Admin Current COLL Balance: {}\n", amount);
        io::stdout().flush().unwrap();
    }

    close_bond(&collateral_snip, &bonds, &mut reports)?;

    let bond_opp_query_msg_3 = bonds::QueryMsg::BondOpportunities {  };
    let opp_query_3: bonds::QueryAnswer = query(&bonds, bond_opp_query_msg_3, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query_3
    {
        assert_eq!(bond_opportunities.is_empty(), true);
    }

    buy_bond(&collateral_snip, Uint128(10), &mut reports, &bonds)?;

    Ok(())
}

#[test]
fn run_bonds_multiple_opps() -> Result<()> {
    let account_a = account_address(ACCOUNT_KEY)?;
    let account_admin = account_address(ADMIN_KEY)?;
    let mut reports = vec![];

    let now = chrono::offset::Utc::now().timestamp() as u64;
    let end = now + 600u64;
    print_header("Initializing bonds and snip20");
    println!("Printed header");
    let (bonds, mint_snip, coll_snip, mockband, oracle) = setup_contracts(
        Uint128(1_000_000_000_000), 
        2, 
        Uint128(7_000_000_000_000_000_000), 
        true, 
        true, 
        240, 
        Uint128(6), 
        Uint128(100_000_000), 
        130, 
        &mut reports,
    )?;

    set_viewing_keys(VIEW_KEY.to_string(), &mut reports, &bonds, &mint_snip, &coll_snip)?;

    let sefi = setup_additional_snip20_with_vk("sefi".to_string(), "SEFI".to_string(), 8, &mut reports)?;

    set_band_prices(&coll_snip, &mint_snip, Uint128(5_000_000_000_000_000_000), Uint128(2_000_000_000_000_000_000), &mut reports, &mockband)?;

    set_additional_band_price(&sefi, Uint128(10_000_000_000_000_000), "SEFI".to_string(), &mockband, &mut reports)?;
    
    print_header("Band prices set");

    set_minting_privileges(&mint_snip, &bonds, &mut reports)?;
    print_header("Minting privileges set");

    // Open bond opportunity
    let opp_limit = Uint128(100_000_000_000);
    let period = 2u64;
    let disc = Uint128(6_000_000_000_000_000_000);
    open_bond(&coll_snip, now, end, Some(opp_limit), Some(period), Some(disc), Uint128(10_000_000_000_000_000_000), &mut reports, &bonds)?;
    print_header("Bond Opened");

    // Open second opportunity
    let opp_limit_2 = Uint128(200_000_000_000);
    let period_2 = 400u64;
    let disc_2 = Uint128(4_000_000_000_000_000_000);
    open_bond(&sefi, now, end, Some(opp_limit_2), Some(period_2), Some(disc_2), Uint128(10_000_000_000_000_000_000), &mut reports, &bonds)?;
    print_header("Second Bond Opened");

    let g_issued_query_msg = bonds::QueryMsg::BondInfo {  };
    let g_issued_query: bonds::QueryAnswer = query(&bonds, g_issued_query_msg, None)?;
    if let bonds::QueryAnswer::BondInfo { global_total_issued, global_total_claimed, issued_asset
    } = g_issued_query
    {
        assert_eq!(global_total_issued, Uint128(300_000_000_000));
    }

    print_bond_opps(&bonds, &mut reports)?;

    let bond_opp_quer_msg = bonds::QueryMsg::BondOpportunities {  };
    let opp_query: bonds::QueryAnswer = query(&bonds, bond_opp_quer_msg, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query
    {
        assert_eq!(bond_opportunities[0].amount_issued, Uint128(0));
        assert_eq!(bond_opportunities[0].bonding_period, 2);
        assert_eq!(bond_opportunities[0].discount, disc);
        assert_eq!(bond_opportunities[1].amount_issued, Uint128(0));
        assert_eq!(bond_opportunities[1].bonding_period, 400);
        assert_eq!(bond_opportunities[1].discount, disc_2);
    }

    buy_bond(&coll_snip, Uint128(100_000_000), &mut reports, &bonds)?;
    print_header("Bond opp bought");

    buy_bond(&sefi, Uint128(1_000_000_000), &mut reports, &bonds)?;
    print_header("Second opp bought");

    print_pending_bonds(&bonds, &mut reports)?;

    // Create permit
    let account_permit = create_signed_permit(
        AccountPermitMsg {
            contract: HumanAddr(bonds.address.clone()),
            key: "key".to_string(),
        },
        None,
        None,
        ACCOUNT_KEY,
    );


    let account_quer_msg = bonds::QueryMsg::Account { permit: account_permit };
    let account_query: bonds::QueryAnswer = query(&bonds, account_quer_msg, None)?;
    
    if let bonds::QueryAnswer::Account { pending_bonds, 
    } = account_query
    {
        assert_eq!(pending_bonds[0].deposit_amount, Uint128(100_000_000));
        assert_eq!(pending_bonds[0].claim_amount, Uint128(265_957_446));
        assert_eq!(pending_bonds[0].deposit_denom.token_info.symbol, "COLL".to_string());
        assert_eq!(pending_bonds[1].deposit_amount, Uint128(1_000_000_000));
        assert_eq!(pending_bonds[1].claim_amount, Uint128(52_083));
        assert_eq!(pending_bonds[1].deposit_denom.token_info.symbol, "SEFI".to_string());
    }

    claim_bond(&bonds, &mut reports)?;

    print_pending_bonds(&bonds, &mut reports)?;

    let issued_snip_query_msg = snip20::QueryMsg::Balance { address: HumanAddr::from(account_a), key: VIEW_KEY.to_string() };
    let issued_snip_query: snip20::QueryAnswer = query(&mint_snip, issued_snip_query_msg, None)?;

    if let snip20::QueryAnswer::Balance { amount, 
    } = issued_snip_query
    {
        assert_eq!(amount, Uint128(265_957_446));
        println!("Account A Current MINT Balance: {}\n", amount);
        io::stdout().flush().unwrap();
    }

    Ok(())
}

#[test]
fn run_bonds_singular_allowance() -> Result<()> {
    let account_a = account_address(ACCOUNT_KEY)?;
    let account_admin = account_address(ADMIN_KEY)?;
    let account_limit_admin = account_address(LIMIT_ADMIN_KEY)?;
    let mut reports = vec![];

    let now = chrono::offset::Utc::now().timestamp() as u64;
    let end = now + 600u64;
    print_header("Initializing bonds and snip20");
    println!("Printed header");
    let (bonds, issued_snip, collateral_snip, mockband, oracle) = setup_contracts_allowance(
        Uint128(100_000_000_000), 
        2, 
        Uint128(7_000_000_000_000_000_000), 
        true, 
        false, 
        240, 
        Uint128(6), 
        Uint128(100_000_000), 
        130, 
        &mut reports,
    )?;

    print_contract(&issued_snip);
    print_contract(&collateral_snip);
    print_contract(&bonds);
    print_contract(&mockband);
    print_contract(&oracle);

    set_band_prices(&collateral_snip, &issued_snip, Uint128(5_000_000_000_000_000_000), Uint128(2_000_000_000_000_000_000), &mut reports, &mockband)?;
    print_header("Band prices set");

    set_minting_privileges(&issued_snip, &bonds, &mut reports)?;
    print_header("Minting privileges set");

    print_header("Asserting");
    assert_eq!(Uint128(0), get_balance(&issued_snip, account_a.clone()));
    print_header("Done asserting");

    // Allocated allowance to bonds from admin ("treasury, eventually")
    increase_allowance(&bonds, &issued_snip, Uint128(100_000_000_000_000), &mut reports)?;
    

    // Open bond opportunity
    let opp_limit = Uint128(100_000_000_000);
    let period = 2u64;
    let disc = Uint128(6_000_000_000_000_000_000);
    open_bond(&collateral_snip, now, end, Some(opp_limit), Some(period), Some(disc), Uint128(10_000_000_000_000_000_000), &mut reports, &bonds)?;
    print_header("Bond Opened");

    let g_issued_query_msg = bonds::QueryMsg::BondInfo {  };
    let g_issued_query: bonds::QueryAnswer = query(&bonds, g_issued_query_msg, None)?;
    if let bonds::QueryAnswer::BondInfo { global_total_issued, global_total_claimed, issued_asset 
    } = g_issued_query
    {
        assert_eq!(global_total_issued, Uint128(100_000_000_000));
    }

    let bond_opp_quer_msg = bonds::QueryMsg::BondOpportunities {  };
    let opp_query: bonds::QueryAnswer = query(&bonds, bond_opp_quer_msg, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query
    {
        assert_eq!(bond_opportunities[0].amount_issued, Uint128(0));
        assert_eq!(bond_opportunities[0].bonding_period, 2);
        assert_eq!(bond_opportunities[0].discount, disc);
        println!("\nBond opp: {}\n Starts: {}\n Ends: {}\n Bonding period: {}\n Discount: {}\n Amount Available: {}\n", 
        bond_opportunities[0].deposit_denom.token_info.symbol,
        bond_opportunities[0].start_time,
        bond_opportunities[0].end_time,
        bond_opportunities[0].bonding_period,
        bond_opportunities[0].discount,
        (bond_opportunities[0].issuance_limit - bond_opportunities[0].amount_issued).unwrap(),
    )
    }

    buy_bond(&collateral_snip, Uint128(100_000_000), &mut reports, &bonds)?;
    print_header("Bond opp bought");
    set_viewing_keys(VIEW_KEY.to_string(), &mut reports, &bonds, &issued_snip, &collateral_snip)?;

    // Create permit
    let account_permit = create_signed_permit(
        AccountPermitMsg {
            contract: HumanAddr(bonds.address.clone()),
            key: "key".to_string(),
        },
        None,
        None,
        ACCOUNT_KEY,
    );


    let account_quer_msg = bonds::QueryMsg::Account { permit: account_permit };
    let account_query: bonds::QueryAnswer = query(&bonds, account_quer_msg, None)?;
    
    if let bonds::QueryAnswer::Account { pending_bonds, 
    } = account_query
    {
        assert_eq!(pending_bonds[0].deposit_amount, Uint128(100_000_000));
        assert_eq!(pending_bonds[0].claim_amount, Uint128(265_957_446));
        assert_eq!(pending_bonds[0].deposit_denom.token_info.symbol, "COLL".to_string());
        println!("\nBond opp: {}\n Ends: {}\n Deposit Amount: {}\n Deposit Price: {}\n Claim Amount: {}\n Claim Price: {}\n Discount: {}\n Discount Price: {}", 
            pending_bonds[0].deposit_denom.token_info.symbol,
            pending_bonds[0].end,
            pending_bonds[0].deposit_amount,
            pending_bonds[0].deposit_price,
            pending_bonds[0].claim_amount,
            pending_bonds[0].claim_price,
            pending_bonds[0].discount,
            pending_bonds[0].discount_price,
        )
    }

    claim_bond(&bonds, &mut reports)?;


    let bond_opp_query_msg_2 = bonds::QueryMsg::BondOpportunities {  };
    let opp_query_2: bonds::QueryAnswer = query(&bonds, bond_opp_query_msg_2, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query_2
    {
        assert_eq!(bond_opportunities[0].amount_issued, Uint128(265_957_446));
        assert_eq!(bond_opportunities[0].bonding_period, 2);
        assert_eq!(bond_opportunities[0].discount, disc);
        println!("\nBond opp: {}\n Starts: {}\n Ends: {}\n Bonding period: {}\n Discount: {}\n Amount Available: {}\n", 
        bond_opportunities[0].deposit_denom.token_info.symbol,
        bond_opportunities[0].start_time,
        bond_opportunities[0].end_time,
        bond_opportunities[0].bonding_period,
        bond_opportunities[0].discount,
        (bond_opportunities[0].issuance_limit - bond_opportunities[0].amount_issued).unwrap(),
    )
    }

    let issued_snip_query_msg = snip20::QueryMsg::Balance { address: HumanAddr::from(account_a), key: VIEW_KEY.to_string() };
    let issued_snip_query: snip20::QueryAnswer = query(&issued_snip, issued_snip_query_msg, None)?;

    if let snip20::QueryAnswer::Balance { amount, 
    } = issued_snip_query
    {
        assert_eq!(amount, Uint128(265_957_446));
        println!("Account A Current ISSU Balance: {}\n", amount);
        io::stdout().flush().unwrap();
    }

    let collat_snip_query_msg = snip20::QueryMsg::Balance { address: HumanAddr::from(account_admin), key: VIEW_KEY.to_string() };
    let collat_snip_query: snip20::QueryAnswer = query(&collateral_snip, collat_snip_query_msg, None)?;

    if let snip20::QueryAnswer::Balance { amount, 
    } = collat_snip_query
    {
        assert_eq!(amount, Uint128(100_000_000));
        println!("Account Admin Current COLL Balance: {}\n", amount);
        io::stdout().flush().unwrap();
    }

    close_bond(&collateral_snip, &bonds, &mut reports)?;

    let bond_opp_query_msg_3 = bonds::QueryMsg::BondOpportunities {  };
    let opp_query_3: bonds::QueryAnswer = query(&bonds, bond_opp_query_msg_3, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query_3
    {
        assert_eq!(bond_opportunities.is_empty(), true);
    }

    buy_bond(&collateral_snip, Uint128(10), &mut reports, &bonds)?;

    Ok(())
}

#[test]
fn run_bonds_bad_opportunities() -> Result<()> {
    let account_a = account_address(ACCOUNT_KEY)?;
    let account_admin = account_address(ADMIN_KEY)?;
    let account_limit_admin = account_address(LIMIT_ADMIN_KEY)?;
    let mut reports = vec![];

    let now = chrono::offset::Utc::now().timestamp() as u64;
    let end = now + 600u64;
    print_header("Initializing bonds and snip20");
    println!("Printed header");
    let (bonds, issued_snip, collateral_snip, mockband, oracle) = setup_contracts_allowance(
        Uint128(100_000_000_000), 
        5, 
        Uint128(10), 
        false, 
        false, 
        240, 
        Uint128(10), 
        Uint128(100_000_000), 
        130, 
        &mut reports,
    )?;

    print_contract(&issued_snip);
    print_contract(&collateral_snip);
    print_contract(&bonds);
    print_contract(&mockband);
    print_contract(&oracle);

    set_band_prices(&collateral_snip, &issued_snip, Uint128(100_000_000_000_000_000_000), Uint128(2_000_000_000_000_000_000), &mut reports, &mockband)?;
    print_header("Band prices set");

    assert_eq!(Uint128(0), get_balance(&issued_snip, account_a.clone()));

    // Allocated allowance to bonds from admin ("treasury, eventually")
    increase_allowance(&bonds, &issued_snip, Uint128(100_000_000_000_000), &mut reports)?;
    

    // Open bond opportunity
    let opp_limit = Uint128(100_000_000_000);
    let period = 2u64;
    let disc = Uint128(6_000_000_000_000_000_000);
    open_bond(&collateral_snip, now, end, Some(opp_limit), Some(period), Some(disc), Uint128(10_000_000_000_000_000_000), &mut reports, &bonds)?;
    print_header("Opp while deactivated attempted");

    let bond_opp_quer_msg = bonds::QueryMsg::BondOpportunities {  };
    let opp_query: bonds::QueryAnswer = query(&bonds, bond_opp_quer_msg, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query
    {
        assert_eq!(bond_opportunities[0].amount_issued, Uint128(0));
        assert_eq!(bond_opportunities[0].bonding_period, 2);
        assert_eq!(bond_opportunities[0].discount, disc);
        println!("\nBond opp: {}\n Starts: {}\n Ends: {}\n Bonding period: {}\n Discount: {}\n Amount Available: {}\n", 
        bond_opportunities[0].deposit_denom.token_info.symbol,
        bond_opportunities[0].start_time,
        bond_opportunities[0].end_time,
        bond_opportunities[0].bonding_period,
        bond_opportunities[0].discount,
        (bond_opportunities[0].issuance_limit - bond_opportunities[0].amount_issued).unwrap(),
    )
    }
    print_header("Attempted to print opps");

    update_bonds_config(None, None, None, None, Some(true), None, None, None, None, None, None, &bonds, &mut reports)?;

    open_bond(&collateral_snip, now, end, Some(opp_limit), Some(period), Some(disc), Uint128(10_000_000_000_000_000_000), &mut reports, &bonds)?;
    print_header("Opp with bad discount attempted");

    let bond_opp_quer_msg = bonds::QueryMsg::BondOpportunities {  };
    let opp_query: bonds::QueryAnswer = query(&bonds, bond_opp_quer_msg, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query
    {
        assert_eq!(bond_opportunities[0].amount_issued, Uint128(0));
        assert_eq!(bond_opportunities[0].bonding_period, 2);
        assert_eq!(bond_opportunities[0].discount, disc);
        println!("\nBond opp: {}\n Starts: {}\n Ends: {}\n Bonding period: {}\n Discount: {}\n Amount Available: {}\n", 
        bond_opportunities[0].deposit_denom.token_info.symbol,
        bond_opportunities[0].start_time,
        bond_opportunities[0].end_time,
        bond_opportunities[0].bonding_period,
        bond_opportunities[0].discount,
        (bond_opportunities[0].issuance_limit - bond_opportunities[0].amount_issued).unwrap(),
    )
    }
    print_header("Attempted to print opps");

    buy_bond(&collateral_snip, Uint128(100_000_000), &mut reports, &bonds)?;
    print_header("Bond opp bought");
    set_viewing_keys(VIEW_KEY.to_string(), &mut reports, &bonds, &issued_snip, &collateral_snip)?;

    // Create permit
    let account_permit = create_signed_permit(
        AccountPermitMsg {
            contract: HumanAddr(bonds.address.clone()),
            key: "key".to_string(),
        },
        None,
        None,
        ACCOUNT_KEY,
    );

    let account_quer_msg = bonds::QueryMsg::Account { permit: account_permit };
    let account_query: bonds::QueryAnswer = query(&bonds, account_quer_msg, None)?;
    
    if let bonds::QueryAnswer::Account { pending_bonds, 
    } = account_query
    {
        assert_eq!(pending_bonds[0].deposit_amount, Uint128(100_000_000));
        assert_eq!(pending_bonds[0].claim_amount, Uint128(265_957_446));
        assert_eq!(pending_bonds[0].deposit_denom.token_info.symbol, "COLL".to_string());
        println!("\nBond opp: {}\n Ends: {}\n Deposit Amount: {}\n Deposit Price: {}\n Claim Amount: {}\n Claim Price: {}\n Discount: {}\n Discount Price: {}", 
            pending_bonds[0].deposit_denom.token_info.symbol,
            pending_bonds[0].end,
            pending_bonds[0].deposit_amount,
            pending_bonds[0].deposit_price,
            pending_bonds[0].claim_amount,
            pending_bonds[0].claim_price,
            pending_bonds[0].discount,
            pending_bonds[0].discount_price,
        )
    }

    claim_bond(&bonds, &mut reports)?;


    let bond_opp_query_msg_2 = bonds::QueryMsg::BondOpportunities {  };
    let opp_query_2: bonds::QueryAnswer = query(&bonds, bond_opp_query_msg_2, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query_2
    {
        assert_eq!(bond_opportunities[0].amount_issued, Uint128(265_957_446));
        assert_eq!(bond_opportunities[0].bonding_period, 2);
        assert_eq!(bond_opportunities[0].discount, disc);
        println!("\nBond opp: {}\n Starts: {}\n Ends: {}\n Bonding period: {}\n Discount: {}\n Amount Available: {}\n", 
        bond_opportunities[0].deposit_denom.token_info.symbol,
        bond_opportunities[0].start_time,
        bond_opportunities[0].end_time,
        bond_opportunities[0].bonding_period,
        bond_opportunities[0].discount,
        (bond_opportunities[0].issuance_limit - bond_opportunities[0].amount_issued).unwrap(),
    )
    }

    let issued_snip_query_msg = snip20::QueryMsg::Balance { address: HumanAddr::from(account_a), key: VIEW_KEY.to_string() };
    let issued_snip_query: snip20::QueryAnswer = query(&issued_snip, issued_snip_query_msg, None)?;

    if let snip20::QueryAnswer::Balance { amount, 
    } = issued_snip_query
    {
        assert_eq!(amount, Uint128(265_957_446));
        println!("Account A Current ISSU Balance: {}\n", amount);
        io::stdout().flush().unwrap();
    }

    let collat_snip_query_msg = snip20::QueryMsg::Balance { address: HumanAddr::from(account_admin), key: VIEW_KEY.to_string() };
    let collat_snip_query: snip20::QueryAnswer = query(&collateral_snip, collat_snip_query_msg, None)?;

    if let snip20::QueryAnswer::Balance { amount, 
    } = collat_snip_query
    {
        assert_eq!(amount, Uint128(100_000_000));
        println!("Account Admin Current COLL Balance: {}\n", amount);
        io::stdout().flush().unwrap();
    }

    close_bond(&collateral_snip, &bonds, &mut reports)?;

    let bond_opp_query_msg_3 = bonds::QueryMsg::BondOpportunities {  };
    let opp_query_3: bonds::QueryAnswer = query(&bonds, bond_opp_query_msg_3, None)?;
    
    if let bonds::QueryAnswer::BondOpportunities { bond_opportunities,
    } = opp_query_3
    {
        assert_eq!(bond_opportunities.is_empty(), true);
    }

    buy_bond(&collateral_snip, Uint128(10), &mut reports, &bonds)?;

    Ok(())
}