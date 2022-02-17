use colored::*;
use cosmwasm_std::{to_binary, Binary, HumanAddr, Uint128};
use query_authentication::transaction::PubKey;
use query_authentication::{permit::Permit, transaction::PermitSignature};
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
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use secretcli::secretcli::{account_address, create_permit, query, handle, init, Report, create_key_account};
use serde::Serialize;
use serde_json::Result;
use shade_protocol::airdrop::account::{AddressProofPermit, FillerMsg};
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
use network_integration::utils::store_struct;
use secretcli::cli_types::NetContract;

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

fn proof_from_tree(indices: &Vec<usize>, tree: &Vec<Vec<[u8; 32]>>) -> Vec<Binary> {
    let mut current_indices: Vec<usize> = indices.clone();
    let mut helper_nodes: Vec<Binary> = Vec::new();

    for layer in tree {
        let mut siblings: Vec<usize> = Vec::new();
        let mut parents: Vec<usize> = Vec::new();

        for index in current_indices.iter() {
            if index % 2 == 0 {
                siblings.push(index + 1);
                parents.push(index / 2);
            } else {
                siblings.push(index - 1);
                parents.push((index - 1) / 2);
            }
        }

        for sibling in siblings {
            if !current_indices.contains(&sibling) {
                if let Some(item) = layer.get(sibling) {
                    helper_nodes.push(Binary(item.to_vec()));
                }
            }
        }

        parents.dedup();
        current_indices = parents.clone();
    }

    helper_nodes
}

fn setup_contracts(
    dump_address: Option<HumanAddr>,
    start_date: Option<u64>,
    end_date: Option<u64>,
    decay_start: Option<u64>,
    merkle_root: Binary,
    total_accounts: u32,
    max_amount: Uint128,
    default_claim: Uint128,
    task_claim: Vec<RequiredTask>,
    query_rounding: Uint128,
    airdrop_total: Uint128,
    reports: &mut Vec<Report>
) -> Result<(NetContract, NetContract)> {
    let account_a = account_address(ACCOUNT_KEY)?;

    let snip_init_msg = snip20::InitMsg {
        name: "test".to_string(),
        admin: None,
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: Some(vec![InitialBalance {
            address: HumanAddr::from(account_a.clone()),
            amount: airdrop_total,
        }]),
        prng_seed: Default::default(),
        config: Some(InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(true),
            enable_redeem: Some(true),
            enable_mint: Some(true),
            enable_burn: Some(false),
        }),
    };

    let snip = init(
        &snip_init_msg,
        SNIP20_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports
    )?;

    let airdrop_init_msg = airdrop::InitMsg {
        admin: None,
        dump_address,
        airdrop_token: Contract {
            address: HumanAddr::from(snip.address.clone()),
            code_hash: snip.code_hash.clone(),
        },
        airdrop_amount: airdrop_total,
        start_date,
        end_date,
        decay_start,
        merkle_root,
        total_accounts,
        max_amount,
        default_claim,
        task_claim,
        query_rounding,
    };

    let airdrop = init(
        &airdrop_init_msg,
        AIRDROP_FILE,
        &*generate_label(8),
        ACCOUNT_KEY,
        Some(STORE_GAS),
        Some(GAS),
        Some("test"),
        reports
    )?;

    {
        let msg = snip20::HandleMsg::SetViewingKey {
            key: String::from(VIEW_KEY),
            padding: None,
        };

        handle(&msg, &snip, ACCOUNT_KEY,
               Some(GAS), Some("test"), None, reports, None)?;
    }

    /// Assert that we start with nothing
    handle(
        &snip20::HandleMsg::Send {
            recipient: HumanAddr::from(airdrop.address.clone()),
            amount: airdrop_total,
            msg: None,
            memo: None,
            padding: None,
        },
        &snip,
        ACCOUNT_KEY,
        Some(GAS),
        Some("test"),
        None,
        reports,
        None
    )?;

    Ok((airdrop, snip))

}

fn create_account(
    permits: Vec<AddressProofPermit>,
    partial_tree: Vec<Binary>,
    reports: &mut Vec<Report>,
    airdrop: &NetContract
) -> Result<()> {
    print_warning("Creating an account");

    let msg = airdrop::HandleMsg::Account {
        addresses: permits,
        partial_tree,
        padding: None,
    };

    let tx_info = handle(&msg, airdrop, ACCOUNT_KEY, Some("10000000"),
                         Some("test"), None, reports, None)?.1;

    println!("Gas used: {}", tx_info.gas_used);

    Ok(())
}

fn update_account(
    permits: Vec<AddressProofPermit>,
    partial_tree: Vec<Binary>,
    reports: &mut Vec<Report>,
    airdrop: &NetContract
) -> Result<()> {
    print_warning("Updating account");
    let msg = airdrop::HandleMsg::Account {
        addresses: permits,
        partial_tree,
        padding: None,
    };

    let tx_info = handle(&msg, airdrop, ACCOUNT_KEY, Some("10000000"),
                         Some("test"), None, reports, None)?.1;

    println!("Gas used: {}", tx_info.gas_used);

    Ok(())
}

#[test]
fn run_airdrop() -> Result<()> {
    let account_a = account_address(ACCOUNT_KEY)?;
    let account_b = account_address("b")?;
    let account_c = account_address("c")?;
    let account_d = account_address("d")?;

    let a_airdrop = Uint128(50000000);
    let b_airdrop = Uint128(20000000);
    let ab_half_airdrop = Uint128(35000000);
    let c_airdrop = Uint128(10000000);
    let total_airdrop = a_airdrop + b_airdrop + c_airdrop; // 80000000
    let decay_amount = Uint128(10000000);

    let mut reports = vec![];

    print_header("Creating merkle tree");
    let leaves: Vec<[u8; 32]> = vec![
        Sha256::hash((account_a.clone() + &a_airdrop.to_string()).as_bytes()),
        Sha256::hash((account_b.clone() + &b_airdrop.to_string()).as_bytes()),
        Sha256::hash((account_c.clone() + &c_airdrop.to_string()).as_bytes()),
        Sha256::hash((account_d.clone() + &decay_amount.to_string()).as_bytes()),
    ];

    let merlke_tree = MerkleTree::<Sha256>::from_leaves(&leaves);

    print_header("Initializing airdrop and snip20");

    let now = chrono::offset::Utc::now().timestamp() as u64;
    let duration = 300;
    let decay_date = now + duration;
    let end_date = decay_date + 60;

    let (airdrop, snip) = setup_contracts(
        Some(HumanAddr::from(account_a.clone())),
        None, Some(end_date), Some(decay_date),
        Binary(merlke_tree.root().unwrap().to_vec()),
        leaves.len() as u32,
        a_airdrop, Uint128(50),
        vec![RequiredTask {
            address: HumanAddr::from(account_a.clone()),
            percent: Uint128(50),
        }],
        Uint128(30000000),
        total_airdrop + decay_amount, &mut reports)?;

    print_contract(&airdrop);
    print_contract(&snip);

    assert_eq!(Uint128(0), get_balance(&snip, account_a.clone()));

    print_warning("Creating initial permits");
    /// Create AB permit
    let b_address_proof = AddressProofMsg {
        address: HumanAddr(account_b.clone()),
        amount: b_airdrop,
        contract: HumanAddr(airdrop.address.clone()),
        index: 1,
        key: "key".to_string(),
    };

    let b_permit = create_signed_permit(
        FillerMsg::default(),
        Some(to_binary(&b_address_proof).unwrap().to_base64()),
        Some("wasm/MsgExecuteContract".to_string()),
        "b",
    );

    let a_address_proof = AddressProofMsg {
        address: HumanAddr(account_a.clone()),
        amount: a_airdrop,
        contract: HumanAddr(airdrop.address.clone()),
        index: 0,
        key: "key".to_string(),
    };

    print_warning("Creating proof");
    let initial_proof = proof_from_tree(&vec![0, 1], &merlke_tree.layers());

    let a_permit = create_signed_permit(
        FillerMsg::default(),
        Some(to_binary(&a_address_proof).unwrap().to_base64()),
        Some("wasm/MsgExecuteContract".to_string()),
        ACCOUNT_KEY,
    );
    let account_permit = create_signed_permit(
        AccountPermitMsg {
            contract: HumanAddr(airdrop.address.clone()),
            key: "key".to_string(),
        },
        None,
        None,
        ACCOUNT_KEY,
    );

    /// Create an account which will also claim whatever amount is available
    create_account(vec![b_permit.clone(), a_permit.clone()],
                   initial_proof, &mut reports, &airdrop)?;

    print_warning("Getting initial account information");
    {
        let msg = airdrop::QueryMsg::Account {
            permit: account_permit.clone(),
            current_date: None,
        };

        let query: airdrop::QueryAnswer = query(&airdrop, msg, None)?;

        if let airdrop::QueryAnswer::Account {
            total,
            claimed,
            unclaimed,
            finished_tasks,
            ..
        } = query
        {
            assert_eq!(total, a_airdrop + b_airdrop);
            assert_eq!(claimed, ab_half_airdrop);
            assert_eq!(unclaimed, Uint128::zero());
            assert_eq!(finished_tasks.len(), 1);
        }
    }

    /// Assert that we claimed
    assert_eq!(ab_half_airdrop, get_balance(&snip, account_a.clone()));

    print_warning("Enabling the other half of the airdrop");

    handle(&airdrop::HandleMsg::CompleteTask {
        address: HumanAddr::from(account_a.clone()),
        padding: None,
    }, &airdrop, ACCOUNT_KEY, Some(GAS), Some("test"), None, &mut reports, None)?;

    {
        let msg = airdrop::QueryMsg::Account {
            permit: account_permit.clone(),
            current_date: None,
        };

        let query: airdrop::QueryAnswer = query(&airdrop, msg, None)?;

        if let airdrop::QueryAnswer::Account {
            total,
            claimed,
            unclaimed,
            finished_tasks,
            ..
        } = query
        {
            assert_eq!(total, a_airdrop + b_airdrop);
            assert_eq!(claimed, ab_half_airdrop);
            assert_eq!(unclaimed, ab_half_airdrop);
            assert_eq!(finished_tasks.len(), 2);
        }
    }

    print_warning("Verifying query step functionality");

    {
        let msg = airdrop::QueryMsg::TotalClaimed {};

        let query: airdrop::QueryAnswer = query(&airdrop, msg, None)?;

        if let airdrop::QueryAnswer::TotalClaimed { claimed } = query {
            assert_eq!(claimed, Uint128(30000000));
        }
    }

    print_warning("Confirming full airdrop after adding C");

    let c_address_proof = AddressProofMsg {
        address: HumanAddr(account_c.clone()),
        amount: c_airdrop,
        contract: HumanAddr(airdrop.address.clone()),
        index: 2,
        key: "key".to_string(),
    };

    let c_permit = create_signed_permit(
        FillerMsg::default(),
        Some(to_binary(&c_address_proof).unwrap().to_base64()),
        Some("wasm/MsgExecuteContract".to_string()),
        "c",
    );
    let other_proof = proof_from_tree(&vec![2], &merlke_tree.layers());

    update_account(vec![c_permit], other_proof, &mut reports, &airdrop)?;

    /// Assert that we claimed
    assert_eq!(total_airdrop, get_balance(&snip, account_a.clone()));

    /// Query that all of the airdrop is claimed
    {
        let msg = airdrop::QueryMsg::Account {
            permit: account_permit.clone(),
            current_date: None,
        };

        let query: airdrop::QueryAnswer = query(&airdrop, msg, None)?;

        if let airdrop::QueryAnswer::Account {
            total,
            claimed,
            unclaimed,
            finished_tasks,
            ..
        } = query
        {
            assert_eq!(total, total_airdrop);
            assert_eq!(claimed, total_airdrop);
            assert_eq!(unclaimed, Uint128::zero());
            assert_eq!(finished_tasks.len(), 2);
        }
    }

    print_warning("Disabling permit");
    handle(&airdrop::HandleMsg::DisablePermitKey {
        key: "key".to_string(),
        padding: None,
    }, &airdrop, ACCOUNT_KEY, Some(GAS), Some("test"), None, &mut reports, None)?;

    {
        let msg = airdrop::QueryMsg::Account {
            permit: account_permit.clone(),
            current_date: None,
        };

        let query: Result<airdrop::QueryAnswer> = query(&airdrop, msg, Some(3));

        assert!(query.is_err());
    }

    let new_account_permit = create_signed_permit(
        AccountPermitMsg {
            contract: HumanAddr(airdrop.address.clone()),
            key: "new_key".to_string(),
        },
        None,
        None,
        ACCOUNT_KEY,
    );

    {
        let msg = airdrop::QueryMsg::Account {
            permit: new_account_permit.clone(),
            current_date: None,
        };

        let query: airdrop::QueryAnswer = query(&airdrop, msg, None)?;

        if let airdrop::QueryAnswer::Account {
            total,
            claimed,
            unclaimed,
            finished_tasks,
            ..
        } = query
        {
            assert_eq!(total, total_airdrop);
            assert_eq!(claimed, total_airdrop);
            assert_eq!(unclaimed, Uint128::zero());
            assert_eq!(finished_tasks.len(), 2);
        }
    }

    print_warning("Creating Viewing Key");
    {
        let msg = airdrop::QueryMsg::AccountWithKey {
            account: HumanAddr(account_a.clone()),
            current_date: None,
            key: "key".to_string()
        };

        let query: Result<airdrop::QueryAnswer> = query(&airdrop, msg, Some(3));

        assert!(query.is_err());
    }

    handle(&airdrop::HandleMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    }, &airdrop, ACCOUNT_KEY, Some(GAS), Some("test"), None, &mut reports, None)?;

    {
        let msg = airdrop::QueryMsg::AccountWithKey {
            account: HumanAddr(account_a.clone()),
            current_date: None,
            key: "key".to_string()
        };

        let query: airdrop::QueryAnswer = query(&airdrop, msg, None)?;

        if let airdrop::QueryAnswer::Account {
            total,
            claimed,
            unclaimed,
            finished_tasks,
            ..
        } = query
        {
            assert_eq!(total, total_airdrop);
            assert_eq!(claimed, total_airdrop);
            assert_eq!(unclaimed, Uint128::zero());
            assert_eq!(finished_tasks.len(), 2);
        }
    }
    {
        let msg = airdrop::QueryMsg::AccountWithKey {
            account: HumanAddr(account_a.clone()),
            current_date: None,
            key: "wrong".to_string()
        };

        let query: Result<airdrop::QueryAnswer> = query(&airdrop, msg, Some(3));

        assert!(query.is_err());
    }

    print_warning("Claiming partially decayed tokens");
    {
        let current = chrono::offset::Utc::now().timestamp() as u64;
        // Wait until times is between decay start and end of airdrop
        thread::sleep(time::Duration::from_secs((decay_date - current) + 20));
    }

    {
        let d_address_proof = AddressProofMsg {
            address: HumanAddr(account_d.clone()),
            amount: decay_amount,
            contract: HumanAddr(airdrop.address.clone()),
            index: 3,
            key: "key".to_string(),
        };

        let d_permit = create_signed_permit(
            FillerMsg::default(),
            Some(to_binary(&d_address_proof).unwrap().to_base64()),
            Some("wasm/MsgExecuteContract".to_string()),
            "d",
        );
        let d_proof = proof_from_tree(&vec![3], &merlke_tree.layers());

        update_account(vec![d_permit], d_proof, &mut reports, &airdrop)?;

        let balance = get_balance(&snip, account_a.clone());

        assert!(balance > total_airdrop);
        assert!(balance < total_airdrop + decay_amount);
    }

    let pre_decay_balance = get_balance(&snip, account_a.clone());

    /// Try to claim expired tokens
    print_warning("Claiming expired tokens");
    {
        let current = chrono::offset::Utc::now().timestamp() as u64;
        // Wait until times is between decay start and end of airdrop
        thread::sleep(time::Duration::from_secs(end_date - current + 20));
    }

    handle(&airdrop::HandleMsg::ClaimDecay { padding: None }, &airdrop,
           ACCOUNT_KEY, Some(GAS), Some("test"), None,
           &mut reports, None)?;

    assert_eq!(
        total_airdrop + decay_amount,
        get_balance(&snip, account_a.clone())
    );

    print_warning("Verifying query step functionality after decay claim");

    {
        let msg = airdrop::QueryMsg::TotalClaimed {};

        let query: airdrop::QueryAnswer = query(&airdrop, msg, None)?;

        if let airdrop::QueryAnswer::TotalClaimed { claimed } = query {
            assert_eq!(claimed, pre_decay_balance);
        }
    }

    store_struct("run_airdrop.json", &mut reports);

    Ok(())
}

fn generate_memo(airdrop: &NetContract, address: String, index: u32) -> String {
    let mut memo_content = AddressProofMsg {
        address: HumanAddr(address),
        amount: Uint128(1000),
        contract: HumanAddr(airdrop.address.clone()),
        index,
        key: "key".to_string(),
    };

    to_binary(&memo_content).unwrap().to_base64()
}

fn generate_permits(airdrop: &NetContract
) -> Result<(Permit<FillerMsg>, Permit<FillerMsg>, Permit<FillerMsg>,
             Permit<FillerMsg>, Permit<FillerMsg>, Permit<FillerMsg>, Permit<FillerMsg>)> {
    let account_a = account_address(ACCOUNT_KEY)?;
    let account_b = account_address("b")?;
    let account_c = account_address("c")?;
    let account_d = account_address("d")?;
    let account_e = account_address("e")?;
    let account_f = account_address("f")?;
    let account_g = account_address("g")?;

    let a_permit = create_signed_permit(
        FillerMsg::default(),
        Some(generate_memo(airdrop, account_a, 0)),
        Some("wasm/MsgExecuteContract".to_string()),
        "a",
    );

    let b_permit = create_signed_permit(
        FillerMsg::default(),
        Some(generate_memo(airdrop, account_b, 1)),
        Some("wasm/MsgExecuteContract".to_string()),
        "b",
    );

    let c_permit = create_signed_permit(
        FillerMsg::default(),
        Some(generate_memo(airdrop, account_c, 2)),
        Some("wasm/MsgExecuteContract".to_string()),
        "c",
    );

    let d_permit = create_signed_permit(
        FillerMsg::default(),
        Some(generate_memo(airdrop, account_d, 3)),
        Some("wasm/MsgExecuteContract".to_string()),
        "d",
    );

    let e_permit = create_signed_permit(
        FillerMsg::default(),
        Some(generate_memo(airdrop, account_e, 4)),
        Some("wasm/MsgExecuteContract".to_string()),
        "e",
    );

    let f_permit = create_signed_permit(
        FillerMsg::default(),
        Some(generate_memo(airdrop, account_f, 5)),
        Some("wasm/MsgExecuteContract".to_string()),
        "f",
    );

    let g_permit = create_signed_permit(
        FillerMsg::default(),
        Some(generate_memo(airdrop, account_g, 6)),
        Some("wasm/MsgExecuteContract".to_string()),
        "g",
    );

    Ok((a_permit, b_permit, c_permit, d_permit, e_permit, f_permit, g_permit))
}

//#[test]
fn airdrop_gas_prices() -> Result<()> {
    let loops = 3;
    let account_a = account_address(ACCOUNT_KEY)?;
    let account_b = account_address("b")?;
    let account_c = account_address("c")?;
    let account_d = account_address("d")?;

    create_key_account("e")?;
    let account_e = account_address("e")?;

    create_key_account("f")?;
    let account_f = account_address("f")?;

    create_key_account("g")?;
    let account_g = account_address("g")?;

    let leaves: Vec<[u8; 32]> = vec![
        Sha256::hash((account_a.clone() + "1000").as_bytes()),
        Sha256::hash((account_b.clone() + "1000").as_bytes()),
        Sha256::hash((account_c.clone() + "1000").as_bytes()),
        Sha256::hash((account_d.clone() + "1000").as_bytes()),
        Sha256::hash((account_e.clone() + "1000").as_bytes()),
        Sha256::hash((account_f.clone() + "1000").as_bytes()),
        Sha256::hash((account_g.clone() + "1000").as_bytes()),
    ];

    let merlke_tree = MerkleTree::<Sha256>::from_leaves(&leaves);

    // Create account and update account
    print_header("1 Permit (TX signer)");
    let mut reports_0 = vec![];
    for _ in 0..loops {
        {
            let (airdrop, snip) = setup_contracts(
                Some(HumanAddr::from(account_a.clone())),
                None, None, None,
                Binary(merlke_tree.root().unwrap().to_vec()),
                leaves.len() as u32,
                Uint128(1001), Uint128(20),
                vec![RequiredTask {
                    address: HumanAddr::from(account_a.clone()),
                    percent: Uint128(80),
                }],
                Uint128(30000000),
                Uint128(7000), &mut vec![])?;

            let (a_permit, b_permit, c_permit,
                d_permit, e_permit, f_permit,
                g_permit) = generate_permits(&airdrop)?;

            create_account(vec![a_permit.clone()],
                           proof_from_tree(&vec![0], &merlke_tree.layers()),
                           &mut reports_0, &airdrop)?;
        }

        {
            let (airdrop, snip) = setup_contracts(
                Some(HumanAddr::from(account_a.clone())),
                None, None, None,
                Binary(merlke_tree.root().unwrap().to_vec()),
                leaves.len() as u32,
                Uint128(1001), Uint128(20),
                vec![RequiredTask {
                    address: HumanAddr::from(account_a.clone()),
                    percent: Uint128(80),
                }],
                Uint128(30000000),
                Uint128(7000), &mut vec![])?;

            let (a_permit, b_permit, c_permit,
                d_permit, e_permit, f_permit,
                g_permit) = generate_permits(&airdrop)?;

            create_account(vec![b_permit.clone()],
                           proof_from_tree(&vec![1], &merlke_tree.layers()),
                           &mut vec![], &airdrop)?;

            update_account(vec![a_permit],
                           proof_from_tree(&vec![0],
                                           &merlke_tree.layers()), &mut reports_0, &airdrop)?;
        }
    }
    store_struct("airdrop_0_permit.json", &reports_0);
    // 1 Permit
    print_header("1 Permit");
    let mut reports_1 = vec![];
    for _ in 0..loops {
        let (airdrop, snip) = setup_contracts(
            Some(HumanAddr::from(account_a.clone())),
            None, None, None,
            Binary(merlke_tree.root().unwrap().to_vec()),
            leaves.len() as u32,
            Uint128(1001), Uint128(20),
            vec![RequiredTask {
                address: HumanAddr::from(account_a.clone()),
                percent: Uint128(80),
            }],
            Uint128(30000000),
            Uint128(7000), &mut vec![])?;

        let (a_permit, b_permit, c_permit,
            d_permit, e_permit, f_permit,
            g_permit) = generate_permits(&airdrop)?;

        create_account(vec![b_permit.clone()],
                       proof_from_tree(&vec![1], &merlke_tree.layers()),
                       &mut reports_1, &airdrop)?;

        update_account(vec![c_permit.clone()],
                       proof_from_tree(&vec![2], &merlke_tree.layers()),
                       &mut reports_1, &airdrop)?;
    }
    store_struct("airdrop_1_permit.json", &reports_1);
    // 2 Permits
    print_header("2 Permits");
    let mut reports_2 = vec![];
    for _ in 0..loops {
        let (airdrop, snip) = setup_contracts(
            Some(HumanAddr::from(account_a.clone())),
            None, None, None,
            Binary(merlke_tree.root().unwrap().to_vec()),
            leaves.len() as u32,
            Uint128(1001), Uint128(20),
            vec![RequiredTask {
                address: HumanAddr::from(account_a.clone()),
                percent: Uint128(80),
            }],
            Uint128(30000000),
            Uint128(7000), &mut vec![])?;

        let (a_permit, b_permit, c_permit,
            d_permit, e_permit, f_permit,
            g_permit) = generate_permits(&airdrop)?;

        create_account(vec![b_permit.clone(), c_permit.clone()],
                       proof_from_tree(&vec![1, 2], &merlke_tree.layers()),
                       &mut reports_2, &airdrop)?;

        update_account(vec![d_permit.clone(), e_permit.clone()],
                       proof_from_tree(&vec![3, 4], &merlke_tree.layers()),
                       &mut reports_2, &airdrop)?;
    }
    store_struct("airdrop_2_permit.json", &reports_2);
    // 3 Permits
    print_header("3 Permits");
    let mut reports_3 = vec![];
    for _ in 0..loops {
        let (airdrop, snip) = setup_contracts(
            Some(HumanAddr::from(account_a.clone())),
            None, None, None,
            Binary(merlke_tree.root().unwrap().to_vec()),
            leaves.len() as u32,
            Uint128(1001), Uint128(20),
            vec![RequiredTask {
                address: HumanAddr::from(account_a.clone()),
                percent: Uint128(80),
            }],
            Uint128(30000000),
            Uint128(7000), &mut vec![])?;

        let (a_permit, b_permit, c_permit,
            d_permit, e_permit, f_permit,
            g_permit) = generate_permits(&airdrop)?;

        create_account(vec![b_permit.clone(), c_permit.clone(), d_permit.clone()],
                       proof_from_tree(&vec![1, 2, 3], &merlke_tree.layers()),
                       &mut reports_3, &airdrop)?;

        update_account(vec![e_permit.clone(), f_permit.clone(), g_permit.clone()],
                       proof_from_tree(&vec![4, 5, 6], &merlke_tree.layers()),
                       &mut reports_3, &airdrop)?;
    }
    store_struct("airdrop_3_permit.json", &reports_3);
    // 4 Permits
    print_header("4 Permits");
    let mut reports_4 = vec![];
    for _ in 0..loops {
        {
            let (airdrop, snip) = setup_contracts(
                Some(HumanAddr::from(account_a.clone())),
                None, None, None,
                Binary(merlke_tree.root().unwrap().to_vec()),
                leaves.len() as u32,
                Uint128(1001), Uint128(20),
                vec![RequiredTask {
                    address: HumanAddr::from(account_a.clone()),
                    percent: Uint128(80),
                }],
                Uint128(30000000),
                Uint128(7000), &mut vec![])?;

            let (a_permit, b_permit, c_permit,
                d_permit, e_permit, f_permit,
                g_permit) = generate_permits(&airdrop)?;

            create_account(vec![b_permit.clone(), c_permit.clone(), d_permit.clone(), e_permit.clone()],
                           proof_from_tree(&vec![1,2,3,4], &merlke_tree.layers()),
                           &mut reports_4, &airdrop)?;
        }

        {
            let (airdrop, snip) = setup_contracts(
                Some(HumanAddr::from(account_a.clone())),
                None, None, None,
                Binary(merlke_tree.root().unwrap().to_vec()),
                leaves.len() as u32,
                Uint128(1001), Uint128(20),
                vec![RequiredTask {
                    address: HumanAddr::from(account_a.clone()),
                    percent: Uint128(80),
                }],
                Uint128(30000000),
                Uint128(7000), &mut vec![])?;

            let (a_permit, b_permit, c_permit,
                d_permit, e_permit, f_permit,
                g_permit) = generate_permits(&airdrop)?;

            create_account(vec![a_permit.clone()],
                           proof_from_tree(&vec![0], &merlke_tree.layers()),
                           &mut vec![], &airdrop)?;

            update_account(vec![b_permit.clone(), c_permit.clone(), d_permit.clone(), e_permit.clone()],
                           proof_from_tree(&vec![1,2,3,4],
                                           &merlke_tree.layers()), &mut reports_4, &airdrop)?;
        }
    }
    store_struct("airdrop_4_permit.json", &reports_4);
    // 5 Permits
    print_header("5 Permits");
    let mut reports_5 = vec![];
    for _ in 0..loops {
        {
            let (airdrop, snip) = setup_contracts(
                Some(HumanAddr::from(account_a.clone())),
                None, None, None,
                Binary(merlke_tree.root().unwrap().to_vec()),
                leaves.len() as u32,
                Uint128(1001), Uint128(20),
                vec![RequiredTask {
                    address: HumanAddr::from(account_a.clone()),
                    percent: Uint128(80),
                }],
                Uint128(30000000),
                Uint128(7000), &mut vec![])?;

            let (a_permit, b_permit, c_permit,
                d_permit, e_permit, f_permit,
                g_permit) = generate_permits(&airdrop)?;

            create_account(vec![b_permit.clone(), c_permit.clone(), d_permit.clone(),
                                e_permit.clone(), f_permit.clone()],
                           proof_from_tree(&vec![1,2,3,4,5], &merlke_tree.layers()),
                           &mut reports_5, &airdrop)?;
        }

        {
            let (airdrop, snip) = setup_contracts(
                Some(HumanAddr::from(account_a.clone())),
                None, None, None,
                Binary(merlke_tree.root().unwrap().to_vec()),
                leaves.len() as u32,
                Uint128(1001), Uint128(20),
                vec![RequiredTask {
                    address: HumanAddr::from(account_a.clone()),
                    percent: Uint128(80),
                }],
                Uint128(30000000),
                Uint128(7000), &mut vec![])?;

            let (a_permit, b_permit, c_permit,
                d_permit, e_permit, f_permit,
                g_permit) = generate_permits(&airdrop)?;

            create_account(vec![a_permit.clone()],
                           proof_from_tree(&vec![0], &merlke_tree.layers()),
                           &mut vec![], &airdrop)?;

            update_account(vec![b_permit.clone(), c_permit.clone(), d_permit.clone(),
                                e_permit.clone(), f_permit.clone()],
                           proof_from_tree(&vec![1,2,3,4,5],
                                           &merlke_tree.layers()), &mut reports_5, &airdrop)?;
        }
    }
    store_struct("airdrop_5_permit.json", &reports_5);
    Ok(())
}