use colored::*;
use serde_json::Result;
use cosmwasm_std::{HumanAddr, Uint128, to_binary};
use secretcli::{cli_types::NetContract,
                secretcli::{account_address, query_contract, test_contract_handle,
                            test_inst_init, list_contracts_by_code}};
use shade_protocol::{snip20::{InitConfig, InitialBalance}, snip20, governance, staking,
                     micro_mint, band, oracle, asset::Contract, airdrop, airdrop::Reward};
use network_integration::{utils::{print_header, print_warning, generate_label, print_contract,
                             STORE_GAS, GAS, VIEW_KEY, ACCOUNT_KEY, print_vec},
                     contract_helpers::{initializer::initialize_initializer,
                                        governance::{init_contract, get_contract, add_contract,
                                                     create_proposal, trigger_latest_proposal},
                                        minter::{initialize_minter, setup_minters, get_balance},
                                        stake::setup_staker}};

#[test]
fn run_airdrop() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;

    /// Initialize dummy snip20
    print_header("\nInitializing snip20");

    let snip_init_msg = snip20::InitMsg {
        name: "test".to_string(),
        admin: None,
        symbol: "TEST".to_string(),
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
    };

    let snip = test_inst_init(&snip_init_msg, "../../compiled/snip20.wasm.gz", &*generate_label(8),
                               ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                               Some("test"))?;
    print_contract(&snip);

    {
        let msg = snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None };

        test_contract_handle(&msg, &snip, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    /// Assert that we start with nothing
    assert_eq!(Uint128(0), get_balance(&snip, account.clone()));

    let expected_airdrop = Uint128(1000000);

    print_header("Initializing airdrop");

    let airdrop_init_msg = airdrop::InitMsg {
        admin: None,
        airdrop_snip20: Contract {
            address: HumanAddr::from(snip.address.clone()),
            code_hash: snip.code_hash.clone()
        },
        start_time: None,
        end_time: None,
        rewards: vec![Reward {
            address: HumanAddr::from(account.clone()),
            amount: expected_airdrop
        }]
    };

    let airdrop = test_inst_init(&airdrop_init_msg, "../../compiled/airdrop.wasm.gz", &*generate_label(8),
                              ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                              Some("test"))?;
    print_contract(&airdrop);

    /// Query that airdrop is allowed
    {
        let msg = airdrop::QueryMsg::GetEligibility {
            address: HumanAddr::from(account.clone())
        };

        let query: airdrop::QueryAnswer = query_contract(&airdrop, msg)?;

        if let airdrop::QueryAnswer::Eligibility { amount, claimed } = query {
            assert_eq!(amount, expected_airdrop);
            assert_eq!(claimed, false);
        }
    }

    /// Register airdrop as allowed minter
    test_contract_handle(&snip20::HandleMsg::SetMinters {
        minters: vec![HumanAddr::from(airdrop.address.clone())], padding: None },
                         &snip, ACCOUNT_KEY, Some(GAS),
                         Some("test"), None)?;

    print_header("Claiming airdrop");
    /// Claim airdrop
    test_contract_handle(&airdrop::HandleMsg::Claim {},
                         &airdrop, ACCOUNT_KEY, Some(GAS),
                         Some("test"), None)?;

    /// Assert that we claimed
    assert_eq!(expected_airdrop, get_balance(&snip, account.clone()));

    /// Query that airdrop is claimed
    {
        let msg = airdrop::QueryMsg::GetEligibility {
            address: HumanAddr::from(account.clone())
        };

        let query: airdrop::QueryAnswer = query_contract(&airdrop, msg)?;

        if let airdrop::QueryAnswer::Eligibility { amount, claimed } = query {
            assert_eq!(amount, expected_airdrop);
            assert_eq!(claimed, true);
        }
    }

    Ok(())
}

#[test]
fn run_testnet() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;

    println!("Using Account: {}", account.blue());

    /// Initialize sSCRT
    print_header("Initializing sSCRT");

    let sscrt_init_msg = snip20::InitMsg {
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
    };

    let sSCRT = test_inst_init(&sscrt_init_msg, "../../compiled/snip20.wasm.gz", &*generate_label(8),
                               ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                               Some("test"))?;
    print_contract(&sSCRT);

    {
        let msg = snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None };

        test_contract_handle(&msg, &sSCRT, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    println!("\n\tDepositing 1000000000uscrt");

    {

        let msg = snip20::HandleMsg::Deposit { padding: None };

        test_contract_handle(&msg, &sSCRT, ACCOUNT_KEY, Some(GAS),
                             Some("test"), Some("1000000000uscrt"))?;
    }

    /// Initialize Governance
    print_header("Initializing Governance");

    let governance_init_msg = governance::InitMsg {
        admin: None,
        proposal_deadline: 0,
        quorum: Uint128(0)
    };

    let governance = test_inst_init(&governance_init_msg, "../../compiled/governance.wasm.gz", &*generate_label(8),
                                    ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                                    Some("test"))?;

    print_contract(&governance);

    /// Initialize initializer and snip20s
    initialize_initializer(&governance, &sSCRT, account.clone())?;

    /// Print Contracts so far
    print_warning("Governance contracts so far");
    {
        let msg = governance::QueryMsg::GetSupportedContracts {};

        let query: governance::QueryAnswer = query_contract(&governance, &msg)?;

        if let governance::QueryAnswer::SupportedContracts { contracts } = query {
            print_vec("Contracts: ", contracts);
        }
    }
    /// Set Snip20s
    print_warning("Getting Shade contract from governance");
    let shade = get_contract(&governance, "shade".to_string())?;
    print_warning("Getting Silk contract from governance");
    let silk = get_contract(&governance, "silk".to_string())?;

    /// Initialize staking
    let staker = setup_staker(&governance, &shade, account.clone())?;

    /// Initialize Band Mock
    let band = init_contract(&governance, "band_mock".to_string(),
                             "../../compiled/mock_band.wasm.gz",
                             band::InitMsg {})?;

    /// Initialize Oracle
    let oracle = init_contract(&governance, "oracle".to_string(),
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
    /// This also tests that governance can update allowed contracts
    setup_minters(&governance, &mint_shade, &mint_silk, &shade, &silk, &sSCRT)?;

    ///

    Ok(())
}