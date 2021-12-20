use colored::*;
use serde_json::Result;
use cosmwasm_std::{HumanAddr, Uint128, to_binary};
use secretcli::{secretcli::{account_address, query_contract, test_contract_handle, test_inst_init}};
use shade_protocol::{snip20::{InitConfig}, snip20, governance, staking,
                     band, oracle, asset::Contract, airdrop,
                     airdrop::{Reward, RequiredTask},
                     governance::{UserVote, Vote, ProposalStatus}, generic_response::ResponseStatus};
use network_integration::{utils::{print_header, print_warning, generate_label, print_contract,
                             STORE_GAS, GAS, VIEW_KEY, ACCOUNT_KEY, print_vec,
                                  SNIP20_FILE, AIRDROP_FILE, GOVERNANCE_FILE, MOCK_BAND_FILE, ORACLE_FILE},
                     contract_helpers::{initializer::initialize_initializer,
                                        governance::{init_contract, get_contract, add_contract,
                                                     create_proposal, get_latest_proposal,
                                                     create_and_trigger_proposal, trigger_latest_proposal},
                                        minter::{initialize_minter, setup_minters, get_balance},
                                        stake::setup_staker}};
use std::{thread, time};
use chrono;
use shade_protocol::snip20::InitialBalance;

#[test]
fn run_airdrop() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;
    let secondary_account = account_address("b")?;

    let half_airdrop = Uint128(500000);
    let full_airdrop = Uint128(1000000);
    let all_airdrop = Uint128(2000000);

    /// Initialize dummy snip20
    print_header("\nInitializing snip20");

    let snip_init_msg = snip20::InitMsg {
        name: "test".to_string(),
        admin: None,
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: Some(vec![InitialBalance{
            address: HumanAddr::from(account.clone()),
            amount: all_airdrop }]),
        prng_seed: Default::default(),
        config: Some(InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false)
        })
    };

    let snip = test_inst_init(&snip_init_msg, SNIP20_FILE,
                              &*generate_label(8),
                              ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                              Some("test"))?;
    print_contract(&snip);

    {
        let msg = snip20::HandleMsg::SetViewingKey {
            key: String::from(VIEW_KEY),
            padding: None };

        test_contract_handle(&msg, &snip, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    print_header("Initializing airdrop");

    let now = chrono::offset::Utc::now().timestamp() as u64;
    let duration = 180;

    let airdrop_init_msg = airdrop::InitMsg {
        admin: None,
        dump_address: Some(HumanAddr::from(account.clone())),
        airdrop_token: Contract {
            address: HumanAddr::from(snip.address.clone()),
            code_hash: snip.code_hash.clone()
        },
        start_time: None,
        end_time: Some(now + duration),
        rewards: vec![Reward {
            address: HumanAddr::from(account.clone()),
            amount: full_airdrop
        }, Reward {
            address: HumanAddr::from(secondary_account),
            amount: full_airdrop
        }],
        default_claim: Uint128(50),
        task_claim: vec![RequiredTask {
            address: HumanAddr::from(account.clone()),
            percent: Uint128(50) }]
    };

    let airdrop = test_inst_init(&airdrop_init_msg, AIRDROP_FILE, &*generate_label(8),
                              ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                              Some("test"))?;
    print_contract(&airdrop);

    /// Assert that we start with nothing
    {
        test_contract_handle(&snip20::HandleMsg::Send {
            recipient: HumanAddr::from(airdrop.address.clone()),
            amount: all_airdrop,
            msg: None,
            memo: None,
            padding: None
        }, &snip, ACCOUNT_KEY, Some(GAS), Some("test"), None)?;
    }
    assert_eq!(Uint128(0), get_balance(&snip, account.clone()));

    /// Query that airdrop is allowed
    {
        let msg = airdrop::QueryMsg::GetEligibility {
            address: HumanAddr::from(account.clone())
        };

        let query: airdrop::QueryAnswer = query_contract(&airdrop, msg)?;

        if let airdrop::QueryAnswer::Eligibility { total, claimed,
            unclaimed, finished_tasks } = query {
            assert_eq!(total, full_airdrop);
            assert_eq!(claimed, Uint128::zero());
            assert_eq!(unclaimed, half_airdrop);
            assert_eq!(finished_tasks.len(), 1);
        }
    }

    print_warning("Claiming half of the airdrop");
    /// Claim airdrop
    test_contract_handle(&airdrop::HandleMsg::Claim {},
                         &airdrop, ACCOUNT_KEY, Some(GAS),
                         Some("test"), None)?;

    /// Assert that we claimed
    assert_eq!(half_airdrop, get_balance(&snip, account.clone()));

    /// Query that half of the airdrop is claimed
    {
        let msg = airdrop::QueryMsg::GetEligibility {
            address: HumanAddr::from(account.clone())
        };

        let query: airdrop::QueryAnswer = query_contract(&airdrop, msg)?;

        if let airdrop::QueryAnswer::Eligibility { total, claimed,
            unclaimed, finished_tasks } = query {
            assert_eq!(total, full_airdrop);
            assert_eq!(claimed, half_airdrop);
            assert_eq!(unclaimed, Uint128::zero());
            assert_eq!(finished_tasks.len(), 1);
        }
    }

    print_warning("Enabling the other half of the airdrop");

    test_contract_handle(&airdrop::HandleMsg::CompleteTask {
        address: HumanAddr::from(account.clone()) },
                         &airdrop, ACCOUNT_KEY, Some(GAS),
                         Some("test"), None)?;

    print_warning("Claiming remaining half airdrop");

    test_contract_handle(&airdrop::HandleMsg::Claim {},
                         &airdrop, ACCOUNT_KEY, Some(GAS),
                         Some("test"), None)?;

    /// Assert that we claimed
    assert_eq!(full_airdrop, get_balance(&snip, account.clone()));

    /// Query that all of the airdrop is claimed
    {
        let msg = airdrop::QueryMsg::GetEligibility {
            address: HumanAddr::from(account.clone())
        };

        let query: airdrop::QueryAnswer = query_contract(&airdrop, msg)?;

        if let airdrop::QueryAnswer::Eligibility { total, claimed,
            unclaimed, finished_tasks } = query {
            assert_eq!(total, full_airdrop);
            assert_eq!(claimed, full_airdrop);
            assert_eq!(unclaimed, Uint128::zero());
            assert_eq!(finished_tasks.len(), 2);
        }
    }

    /// Try to claim expired tokens
    print_warning("Claiming expired tokens");
    thread::sleep(time::Duration::from_secs(duration));

    test_contract_handle(&airdrop::HandleMsg::Decay {},
                         &airdrop, ACCOUNT_KEY, Some(GAS),
                         Some("test"), None)?;

    assert_eq!(all_airdrop, get_balance(&snip, account.clone()));

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

    let s_sCRT = test_inst_init(&sscrt_init_msg, SNIP20_FILE, &*generate_label(8),
                                ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                                Some("test"))?;
    print_contract(&s_sCRT);

    {
        let msg = snip20::HandleMsg::SetViewingKey { key: String::from(VIEW_KEY), padding: None };

        test_contract_handle(&msg, &s_sCRT, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    println!("\n\tDepositing 1000000000uscrt");

    {

        let msg = snip20::HandleMsg::Deposit { padding: None };

        test_contract_handle(&msg, &s_sCRT, ACCOUNT_KEY, Some(GAS),
                             Some("test"), Some("1000000000uscrt"))?;
    }

    // Initialize initializer and snip20s
    let (initializer, shade, silk) = initialize_initializer(&account, &s_sCRT, &account)?;

    // Initialize Governance
    print_header("Initializing Governance");

    let governance_init_msg = governance::InitMsg {
        admin: None,
        // The next governance votes will not require voting
        staker: None,
        funding_token: Contract {
            address: HumanAddr::from(shade.address.clone()),
            code_hash: shade.code_hash.clone()
        },
        funding_amount: Uint128(1000000),
        funding_deadline: 180,
        voting_deadline: 180,
        // 5 shade is the minimum
        quorum: Uint128(5000000)
    };

    let governance = test_inst_init(&governance_init_msg, GOVERNANCE_FILE, &*generate_label(8),
                                    ACCOUNT_KEY, Some(STORE_GAS), Some(GAS),
                                    Some("test"))?;

    print_contract(&governance);

    // Add contracts
    add_contract("initializer".to_string(), &initializer, &governance)?;
    add_contract("shade".to_string(), &shade, &governance)?;
    add_contract("silk".to_string(), &silk, &governance)?;

    // Change contract admin
    {
        let msg = snip20::HandleMsg::ChangeAdmin {
            address: HumanAddr::from(governance.address.clone()),
            padding: None
        };

        test_contract_handle(&msg, &shade, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
        test_contract_handle(&msg, &silk, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;
    }

    // Print Contracts so far
    print_warning("Governance contracts so far");
    {
        let msg = governance::QueryMsg::GetSupportedContracts {};

        let query: governance::QueryAnswer = query_contract(&governance, &msg)?;

        if let governance::QueryAnswer::SupportedContracts { contracts } = query {
            print_vec("Contracts: ", contracts);
        }
    }
    // Set Snip20s
    print_warning("Getting Shade contract from governance");
    let shade_contract = get_contract(&governance, "shade".to_string())?;
    print_warning("Getting Silk contract from governance");
    let silk_contract = get_contract(&governance, "silk".to_string())?;

    // Initialize Band Mock
    let band = init_contract(&governance, "band_mock".to_string(),
                             MOCK_BAND_FILE,
                             band::InitMsg {})?;

    // Initialize Oracle
    let oracle = init_contract(&governance, "oracle".to_string(),
                               ORACLE_FILE,
                               oracle::InitMsg {
                                   admin: None,
                                   band: Contract {
                                       address: HumanAddr::from(band.address),
                                       code_hash: band.code_hash },
                                   sscrt: Contract {
                                       address: HumanAddr::from(s_sCRT.address.clone()),
                                       code_hash: s_sCRT.code_hash.clone() } })?;

    // Initialize Mint-Shade
    let mint_shade = initialize_minter(&governance, "shade_minter".to_string(),
                                       &shade_contract)?;

    // Initialize Mint-Silk
    let mint_silk = initialize_minter(&governance, "silk_minter".to_string(),
                                      &silk_contract)?;

    // Setup mint contracts
    // This also tests that governance can update allowed contracts
    setup_minters(&governance, &mint_shade, &mint_silk, &shade_contract, &silk_contract, &s_sCRT)?;

    // Initialize staking
    let staker = setup_staker(&governance, &shade_contract, account.clone())?;

    // Set governance to require voting
    print_warning("Enabling governance voting");
    create_and_trigger_proposal(&governance, governance::GOVERNANCE_SELF.to_string(),
                                governance::HandleMsg::UpdateConfig {
                                    admin: None,
                                    staker: Some(Contract {
                                        address: HumanAddr::from(staker.address.clone()),
                                        code_hash: staker.code_hash.clone()
                                    }),
                                    proposal_deadline: None,
                                    funding_amount: None,
                                    funding_deadline: None,
                                    minimum_votes: None
                                },
                                Some("Remove control from admin and initialize governance"))?;

    // Proposal admin command
    print_header("Creating proposal expected to fail");
    let admin_command = "{\"update_config\":{\"unbond_time\": {}, \"admin\": null}}";

    // Create a proposal and vote half of the votes
    create_proposal(&governance, governance::GOVERNANCE_SELF.to_string(),
                    governance::HandleMsg::AddAdminCommand {
                        name: "stake_unbond_time".to_string(),
                        proposal: admin_command.to_string() },
                    Some("Staker unbond time can be updated by admin whenever"))?;

    // Fund the proposal
    print_warning("Funding proposal");
    {
        let proposal = get_latest_proposal(&governance)?;

        // Check that its in a funding period
        {
            let msg = governance::QueryMsg::GetProposal { proposal_id: proposal };

            let query: governance::QueryAnswer = query_contract(&governance, msg)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Funding);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }

        let balance = get_balance(&shade, account.clone());

        test_contract_handle(&snip20::HandleMsg::Send {
            recipient: HumanAddr::from(governance.address.clone()),
            amount: Uint128(1000000),
            msg: Some(to_binary(&proposal).unwrap()),
            memo: None,
            padding: None
        }, &shade, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;

        // Check that its in a voting period
        {
            let msg = governance::QueryMsg::GetProposal { proposal_id: proposal };

            let query: governance::QueryAnswer = query_contract(&governance, msg)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Voting);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }

        print_warning("Checking that funds are returned");
        assert_eq!(balance, get_balance(&shade, account.clone()));
    }

    print_warning("Voting on proposal");
    {
        let proposal = get_latest_proposal(&governance)?;

        // Vote on proposal
        test_contract_handle(&staking::HandleMsg::Vote {
            proposal_id: proposal,
            votes: vec![UserVote { vote: Vote::Yes, weight: 50 }]},
                             &staker, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;

        // Verify that the proposal votes were properly done
        {
            let msg = governance::QueryMsg::GetProposalVotes {
                proposal_id: proposal
            };

            let query: governance::QueryAnswer = query_contract(&governance, msg)?;

            if let governance::QueryAnswer::ProposalVotes { status } = query {
                assert_eq!(status.abstain, Uint128(0));
                assert_eq!(status.no, Uint128(0));
                assert_eq!(status.yes, Uint128(2500000));
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }

        // Try to trigger the proposal before the time limit is reached
        trigger_latest_proposal(&governance)?;

        // Query its status
        {
            let msg = governance::QueryMsg::GetProposal { proposal_id: proposal };

            let query: governance::QueryAnswer = query_contract(&governance, msg)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Voting);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }

        // Wait for the time limit and try to trigger it
        thread::sleep(time::Duration::from_secs(180));
        trigger_latest_proposal(&governance)?;

        // Query its status and expect it to break
        {
            let msg = governance::QueryMsg::GetProposal { proposal_id: proposal };

            let query: governance::QueryAnswer = query_contract(&governance, msg)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Expired);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }
    }

    print_header("Creating admin command");
    create_proposal(&governance, governance::GOVERNANCE_SELF.to_string(),
                    governance::HandleMsg::AddAdminCommand {
                        name: "stake_unbond_time".to_string(),
                        proposal: admin_command.to_string() },
                    Some("Staker unbond time can be updated by admin whenever"))?;



    {
        let proposal = get_latest_proposal(&governance)?;

        print_warning("Funding proposal");
        test_contract_handle(&snip20::HandleMsg::Send {
            recipient: HumanAddr::from(governance.address.clone()),
            amount: Uint128(1000000),
            msg: Some(to_binary(&proposal).unwrap()),
            memo: None,
            padding: None
        }, &shade, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;

        // Vote on proposal
        print_warning("Voting on proposal");
        test_contract_handle(&staking::HandleMsg::Vote {
            proposal_id: proposal,
            votes: vec![UserVote { vote: Vote::Yes, weight: 50 },
                        UserVote { vote: Vote::No, weight: 25 },
                        UserVote { vote: Vote::Abstain, weight: 25 }]},
                             &staker, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;

        // Wait for the time limit and try to trigger it
        thread::sleep(time::Duration::from_secs(180));
        trigger_latest_proposal(&governance)?;

        // Query its status and expect it to finish
        {
            let msg = governance::QueryMsg::GetProposal { proposal_id: proposal };

            let query: governance::QueryAnswer = query_contract(&governance, msg)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Passed);
                assert_eq!(proposal.run_status, Some(ResponseStatus::Success));
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }
    }

    // Trigger the admin command
    print_warning("Triggering admin command");
        test_contract_handle(&governance::HandleMsg::TriggerAdminCommand {
            target: "staking".to_string(),
            command: "stake_unbond_time".to_string(),
            variables: vec!["240".to_string()],
            description: "Extending unbond time for staking contract".to_string()
        }, &governance, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;

    // Check that admin command did something
    {
        let msg = staking::QueryMsg::Config {};

        let query: staking::QueryAnswer = query_contract(&staker, msg)?;

        if let staking::QueryAnswer::Config { config } = query {
            assert_eq!(config.unbond_time, 240);
        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

    // Make a failed funding period
    print_header("Testing failed funding");
    create_proposal(&governance, governance::GOVERNANCE_SELF.to_string(),
                    governance::HandleMsg::AddAdminCommand {
                        name: "stake_unbond_time".to_string(),
                        proposal: admin_command.to_string() },
                    Some("This wont be funded :("))?;


    {
        print_warning("Trying to fund");
        let proposal = get_latest_proposal(&governance)?;

        let lost_amount = Uint128(500000);
        let balance_before = get_balance(&shade, account.clone());

        test_contract_handle(&snip20::HandleMsg::Send {
            recipient: HumanAddr::from(governance.address.clone()),
            amount: lost_amount,
            msg: Some(to_binary(&proposal).unwrap()),
            memo: None,
            padding: None
        }, &shade, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;

        let balance_after = get_balance(&shade, account.clone());

        assert_ne!(balance_before, balance_after);

        // Wait funding period
        thread::sleep(time::Duration::from_secs(180));

        // Trigger funding
        test_contract_handle(&snip20::HandleMsg::Send {
            recipient: HumanAddr::from(governance.address.clone()),
            amount: lost_amount,
            msg: Some(to_binary(&proposal).unwrap()),
            memo: None,
            padding: None
        }, &shade, ACCOUNT_KEY, Some(GAS),
                             Some("test"), None)?;

        assert_eq!(get_balance(&shade, account.clone()), balance_after);

        print_warning("Proposal must be expired");
        {
            let msg = governance::QueryMsg::GetProposal { proposal_id: proposal };

            let query: governance::QueryAnswer = query_contract(&governance, msg)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Expired);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }
    }

    Ok(())
}