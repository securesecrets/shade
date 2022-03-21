use colored::*;
use cosmwasm_std::{to_binary, Binary, HumanAddr, Uint128};
use network_integration::utils::store_struct;
use network_integration::{
    contract_helpers::{
        governance::{
            add_contract, create_and_trigger_proposal, create_proposal, get_contract,
            get_latest_proposal, init_with_gov, trigger_latest_proposal,
        },
        initializer::initialize_initializer,
        minter::{get_balance, initialize_minter, setup_minters},
        stake::setup_staker,
    },
    utils::{
        generate_label, print_contract, print_header, print_vec, print_warning, ACCOUNT_KEY,
        AIRDROP_FILE, GAS, GOVERNANCE_FILE, MOCK_BAND_FILE, ORACLE_FILE, SNIP20_FILE, STORE_GAS,
        VIEW_KEY,
    },
};
use query_authentication::transaction::PubKey;
use query_authentication::{permit::Permit, transaction::PermitSignature};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use secretcli::secretcli::{account_address, create_permit, handle, init, query};
use serde::Serialize;
use serde_json::Result;
use shade_protocol::airdrop::account::FillerMsg;
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::{
    airdrop::{
        self,
        account::{AccountPermitMsg, AddressProofMsg},
        claim_info::RequiredTask,
    },
    band, governance,
    governance::{
        proposal::ProposalStatus,
        vote::{UserVote, Vote},
    },
    oracle,
    snip20::{self, InitConfig, InitialBalance},
    staking,
};
use std::{thread, time};

#[test]
fn run_testnet() -> Result<()> {
    let account = account_address(ACCOUNT_KEY)?;

    println!("Using Account: {}", account.blue());

    let mut reports = vec![];

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
            enable_burn: Some(false),
        }),
    };

    let s_sCRT = init(
        &sscrt_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;
    print_contract(&s_sCRT);

    {
        let msg = snip20::HandleMsg::SetViewingKey {
            key: String::from(VIEW_KEY),
            padding: None,
        };

        handle(
            &msg,
            &s_sCRT,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;
    }

    println!("\n\tDepositing 1000000000uscrt");

    {
        let msg = snip20::HandleMsg::Deposit { padding: None };

        handle(
            &msg,
            &s_sCRT,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            Some("1000000000uscrt"),
            &mut reports,
            None,
        )?;
    }

    // Initialize initializer and snip20s
    let (initializer, shade, silk) =
        initialize_initializer(account.clone(), &s_sCRT, account.clone(), &mut reports)?;

    // Initialize Governance
    print_header("Initializing Governance");

    let governance_init_msg = governance::InitMsg {
        admin: None,
        // The next governance votes will not require voting
        staker: None,
        funding_token: Contract {
            address: HumanAddr::from(shade.address.clone()),
            code_hash: shade.code_hash.clone(),
        },
        funding_amount: Uint128(1000000),
        funding_deadline: 180,
        voting_deadline: 180,
        // 5 shade is the minimum
        quorum: Uint128(5000000),
    };

    let governance = init(
        &governance_init_msg,
        GOVERNANCE_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        &mut reports,
    )?;

    print_contract(&governance);

    // Add contracts
    add_contract(
        "initializer".to_string(),
        &initializer,
        &governance,
        &mut reports,
    )?;
    add_contract("shade".to_string(), &shade, &governance, &mut reports)?;
    add_contract("silk".to_string(), &silk, &governance, &mut reports)?;

    // Change contract admin
    {
        let msg = snip20::HandleMsg::ChangeAdmin {
            address: HumanAddr::from(governance.address.clone()),
            padding: None,
        };

        handle(
            &msg,
            &shade,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;
        handle(
            &msg,
            &silk,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;
    }

    // Print Contracts so far
    print_warning("Governance contracts so far");
    {
        let msg = governance::QueryMsg::GetSupportedContracts {};

        let query: governance::QueryAnswer = query(&governance, &msg, None)?;

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
    let band = init_with_gov(
        &governance,
        "band_mock".to_string(),
        MOCK_BAND_FILE,
        band::InitMsg {},
        &mut reports,
    )?;

    // Initialize Oracle
    let oracle = init_with_gov(
        &governance,
        "oracle".to_string(),
        ORACLE_FILE,
        oracle::InitMsg {
            admin: None,
            band: Contract {
                address: HumanAddr::from(band.address),
                code_hash: band.code_hash,
            },
            sscrt: Contract {
                address: HumanAddr::from(s_sCRT.address.clone()),
                code_hash: s_sCRT.code_hash.clone(),
            },
        },
        &mut reports,
    )?;

    // Initialize Mint-Shade
    let mint_shade = initialize_minter(
        &governance,
        "shade_minter".to_string(),
        &shade_contract,
        &mut reports,
    )?;

    // Initialize Mint-Silk
    let mint_silk = initialize_minter(
        &governance,
        "silk_minter".to_string(),
        &silk_contract,
        &mut reports,
    )?;

    // Setup mint contracts
    // This also tests that governance can update allowed contracts
    setup_minters(
        &governance,
        &mint_shade,
        &mint_silk,
        &shade_contract,
        &silk_contract,
        &s_sCRT,
        &mut reports,
    )?;

    // Initialize staking
    let staker = setup_staker(&governance, &shade_contract, account.clone(), &mut reports)?;

    // Set governance to require voting
    print_warning("Enabling governance voting");
    create_and_trigger_proposal(
        &governance,
        governance::GOVERNANCE_SELF.to_string(),
        governance::HandleMsg::UpdateConfig {
            admin: None,
            staker: Some(Contract {
                address: HumanAddr::from(staker.address.clone()),
                code_hash: staker.code_hash.clone(),
            }),
            proposal_deadline: None,
            funding_amount: None,
            funding_deadline: None,
            minimum_votes: None,
        },
        Some("Remove control from admin and initialize governance"),
        &mut reports,
    )?;

    // Proposal admin command
    print_header("Creating proposal expected to fail");
    let admin_command = "{\"update_config\":{\"unbond_time\": {}, \"admin\": null}}";

    // Create a proposal and vote half of the votes
    create_proposal(
        &governance,
        governance::GOVERNANCE_SELF.to_string(),
        governance::HandleMsg::AddAdminCommand {
            name: "stake_unbond_time".to_string(),
            proposal: admin_command.to_string(),
        },
        Some("Staker unbond time can be updated by admin whenever"),
        &mut reports,
    )?;

    // Fund the proposal
    print_warning("Funding proposal");
    {
        let proposal = get_latest_proposal(&governance)?;

        // Check that its in a funding period
        {
            let msg = governance::QueryMsg::GetProposal {
                proposal_id: proposal,
            };

            let query: governance::QueryAnswer = query(&governance, msg, None)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Funding);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }

        let balance = get_balance(&shade, account.clone());

        handle(
            &snip20::HandleMsg::Send {
                recipient: HumanAddr::from(governance.address.clone()),
                amount: Uint128(1000000),
                msg: Some(to_binary(&proposal).unwrap()),
                memo: None,
                padding: None,
            },
            &shade,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;

        // Check that its in a voting period
        {
            let msg = governance::QueryMsg::GetProposal {
                proposal_id: proposal,
            };

            let query: governance::QueryAnswer = query(&governance, msg, None)?;

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
        let proposal_time = chrono::offset::Utc::now().timestamp() as u64;
        let proposal = get_latest_proposal(&governance)?;

        // Vote on proposal
        handle(
            &staking::HandleMsg::Vote {
                proposal_id: proposal,
                votes: vec![UserVote {
                    vote: Vote::Yes,
                    weight: 50,
                }],
            },
            &staker,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;

        // Verify that the proposal votes were properly done
        {
            let msg = governance::QueryMsg::GetProposalVotes {
                proposal_id: proposal,
            };

            let query: governance::QueryAnswer = query(&governance, msg, None)?;

            if let governance::QueryAnswer::ProposalVotes { status } = query {
                assert_eq!(status.abstain, Uint128(0));
                assert_eq!(status.no, Uint128(0));
                assert_eq!(status.yes, Uint128(2500000));
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }

        // Try to trigger the proposal before the time limit is reached
        trigger_latest_proposal(&governance, &mut reports)?;

        // Query its status
        {
            let msg = governance::QueryMsg::GetProposal {
                proposal_id: proposal,
            };

            let query: governance::QueryAnswer = query(&governance, msg, None)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Voting);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }

        // Wait for the time limit and try to trigger it
        let now = chrono::offset::Utc::now().timestamp() as u64;
        thread::sleep(time::Duration::from_secs(proposal_time + 184 - now));

        trigger_latest_proposal(&governance, &mut reports)?;

        // Query its status and expect it to break
        {
            let msg = governance::QueryMsg::GetProposal {
                proposal_id: proposal,
            };

            let query: governance::QueryAnswer = query(&governance, msg, None)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Expired);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }
    }

    print_header("Creating admin command");
    create_proposal(
        &governance,
        governance::GOVERNANCE_SELF.to_string(),
        governance::HandleMsg::AddAdminCommand {
            name: "stake_unbond_time".to_string(),
            proposal: admin_command.to_string(),
        },
        Some("Staker unbond time can be updated by admin whenever"),
        &mut reports,
    )?;

    {
        let proposal_time = chrono::offset::Utc::now().timestamp() as u64;
        let proposal = get_latest_proposal(&governance)?;

        print_warning("Funding proposal");
        handle(
            &snip20::HandleMsg::Send {
                recipient: HumanAddr::from(governance.address.clone()),
                amount: Uint128(1000000),
                msg: Some(to_binary(&proposal).unwrap()),
                memo: None,
                padding: None,
            },
            &shade,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;

        // Vote on proposal
        print_warning("Voting on proposal");
        handle(
            &staking::HandleMsg::Vote {
                proposal_id: proposal,
                votes: vec![
                    UserVote {
                        vote: Vote::Yes,
                        weight: 50,
                    },
                    UserVote {
                        vote: Vote::No,
                        weight: 25,
                    },
                    UserVote {
                        vote: Vote::Abstain,
                        weight: 25,
                    },
                ],
            },
            &staker,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;

        // Wait for the time limit and try to trigger it
        let now = chrono::offset::Utc::now().timestamp() as u64;
        thread::sleep(time::Duration::from_secs(proposal_time + 184 - now));
        trigger_latest_proposal(&governance, &mut reports)?;

        // Query its status and expect it to finish
        {
            let msg = governance::QueryMsg::GetProposal {
                proposal_id: proposal,
            };

            let query: governance::QueryAnswer = query(&governance, msg, None)?;

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
    handle(
        &governance::HandleMsg::TriggerAdminCommand {
            target: "staking".to_string(),
            command: "stake_unbond_time".to_string(),
            variables: vec!["240".to_string()],
            description: "Extending unbond time for staking contract".to_string(),
        },
        &governance,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        &mut reports,
        None,
    )?;

    // Check that admin command did something
    {
        let msg = staking::QueryMsg::Config {};

        let query: staking::QueryAnswer = query(&staker, msg, None)?;

        if let staking::QueryAnswer::Config { config } = query {
            assert_eq!(config.unbond_time, 240);
        } else {
            assert!(false, "Query returned unexpected response")
        }
    }

    // Make a failed funding period
    print_header("Testing failed funding");
    create_proposal(
        &governance,
        governance::GOVERNANCE_SELF.to_string(),
        governance::HandleMsg::AddAdminCommand {
            name: "stake_unbond_time".to_string(),
            proposal: admin_command.to_string(),
        },
        Some("This wont be funded :("),
        &mut reports,
    )?;

    {
        print_warning("Trying to fund");
        let proposal = get_latest_proposal(&governance)?;
        let proposal_time = chrono::offset::Utc::now().timestamp() as u64;

        let lost_amount = Uint128(500000);
        let balance_before = get_balance(&shade, account.clone());

        handle(
            &snip20::HandleMsg::Send {
                recipient: HumanAddr::from(governance.address.clone()),
                amount: lost_amount,
                msg: Some(to_binary(&proposal).unwrap()),
                memo: None,
                padding: None,
            },
            &shade,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;

        let balance_after = get_balance(&shade, account.clone());

        assert_ne!(balance_before, balance_after);

        // Wait funding period
        let now = chrono::offset::Utc::now().timestamp() as u64;
        thread::sleep(time::Duration::from_secs(proposal_time + 184 - now));

        // Trigger funding
        handle(
            &snip20::HandleMsg::Send {
                recipient: HumanAddr::from(governance.address.clone()),
                amount: lost_amount,
                msg: Some(to_binary(&proposal).unwrap()),
                memo: None,
                padding: None,
            },
            &shade,
            ACCOUNT_KEY,
            Some(GAS),
            Some("test"),
            None,
            &mut reports,
            None,
        )?;

        assert_eq!(get_balance(&shade, account.clone()), balance_after);

        print_warning("Proposal must be expired");
        {
            let msg = governance::QueryMsg::GetProposal {
                proposal_id: proposal,
            };

            let query: governance::QueryAnswer = query(&governance, msg, None)?;

            if let governance::QueryAnswer::Proposal { proposal } = query {
                assert_eq!(proposal.status, ProposalStatus::Expired);
            } else {
                assert!(false, "Query returned unexpected response")
            }
        }
    }

    store_struct("run_testnet.json", &mut reports);

    Ok(())
}
