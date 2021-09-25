use colored::*;
use serde_json::Result;
use cosmwasm_std::{HumanAddr, Uint128, to_binary};
use secretcli::{cli_types::NetContract,
                secretcli::{account_address, TestInit, TestHandle,
                            TestQuery, list_contracts_by_code}};
use shade_protocol::{snip20::{InitConfig, InitialBalance}, snip20, governance,
                     micro_mint, band, oracle, asset::Contract};
use network_tester::{utils::{print_header, print_warning, generate_label, print_contract, gov_init_contract,
                             gov_custom_proposal, gov_get_contract, STORE_GAS, GAS,
                             VIEW_KEY, ACCOUNT_KEY, print_vec},
                     contract_helpers::{initializer::initialize_initializer,
                                        minter::{initialize_minter, setup_minters}}};

fn main() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;

    println!("Using Account: {}", account.blue());

    /// Initialize sSCRT
    print_header("Initializing sSCRT");
    let sSCRT = snip20::InitMsg {
        name: "sSCRT".to_string(),
        admin: None,
        symbol: "SSCRT".to_string(),
        decimals: 6,
        initial_balances: None,
        prng_seed: Default::default(),
        config: Some(InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false)
        })
    }.inst_init("../../compiled/snip20.wasm.gz", &*generate_label(8),
                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                Some("test"))?;
    print_contract(&sSCRT);

    snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None }.t_handle(
        &sSCRT, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;

    println!("\n\tDepositing 1000000000uscrt");

    snip20::HandleMsg::Deposit { padding: None }.t_handle(&sSCRT, ACCOUNT_KEY,
                                                          Some(GAS), Some("test"),
                                                          Some("1000000000uscrt"))?;

    /// Initialize Governance
    print_header("Initializing Governance");
    let governance = governance::InitMsg {
        admin: None,
        proposal_deadline: 0,
        minimum_votes: Uint128(0),
    }.inst_init("../../compiled/governance.wasm.gz", &*generate_label(8),
                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                Some("test"))?;

    print_contract(&governance);

    /// Initialize initializer and snip20s
    initialize_initializer(&governance, &sSCRT, account.clone())?;

    /// Initialize Band Mock
    let band = gov_init_contract(&governance, "band_mock".to_string(),
                                 "../../compiled/mock_band.wasm.gz",
                                 band::InitMsg {})?;

    /// Print Contracts so far
    print_warning("Governance contracts so far");
    {
        let query: governance::QueryAnswer = governance::QueryMsg::GetSupportedContracts {
        }.t_query(&governance)?;

        if let governance::QueryAnswer::SupportedContracts { contracts } = query {
            print_vec("Contracts: ", contracts);
        }
    }
    /// Set Snip20s
    print_warning("Getting Shade contract from governance");
    let shade = gov_get_contract(&governance, "shade".to_string())?;
    print_warning("Getting Silk contract from governance");
    let silk = gov_get_contract(&governance, "silk".to_string())?;

    /// Initialize Oracle
    let oracle = gov_init_contract(&governance, "oracle".to_string(),
                                   "../../compiled/oracle.wasm.gz",
                                   oracle::InitMsg {
                                       admin: None,
                                       band: Contract {
                                           address: HumanAddr::from(band.address),
                                           code_hash: band.code_hash },
                                       sscrt: Contract {
                                           address: HumanAddr::from(sSCRT.address.clone()),
                                           code_hash: sSCRT.code_hash.clone() } })?;

    /// Initialize Mint-Shade
    let mint_shade = initialize_minter(&governance, "shade_minter".to_string(),
                                   &shade)?;

    /// Initialize Mint-Silk
    let mint_silk = initialize_minter(&governance, "silk_minter".to_string(),
                                   &silk)?;

    /// Setup mint contracts
    setup_minters(&governance, &mint_shade, &mint_silk, &shade, &silk, &sSCRT)?;



    Ok(())
}