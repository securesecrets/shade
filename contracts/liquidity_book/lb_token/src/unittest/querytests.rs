use core::panic;
use std::ops::Add;

use super::testhelpers::*;

use crate::contract::{execute, instantiate, query};

use shade_protocol::lb_libraries::lb_token::{
    expiration::*, permissions::*, state_structs::*, txhistory::*,
};
use shade_protocol::liquidity_book::lb_token::*;

use cosmwasm_std::{from_binary, testing::*, Addr, Response, StdResult, Uint256};

/////////////////////////////////////////////////////////////////////////////////
// Tests
/////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_q_init() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());

    // instantiate
    let (init_result, mut deps) = init_helper_default();
    assert_ne!(init_result.unwrap(), Response::default());

    // check contract info
    let msg = QueryMsg::TokenContractInfo {};
    let q_result = query(deps.as_ref(), mock_env(), msg);
    let q_answer = from_binary::<QueryAnswer>(&q_result?)?;
    match q_answer {
        QueryAnswer::TokenContractInfo {
            admin,
            curators,
            all_token_ids,
        } => {
            assert_eq!(&admin.unwrap(), &addr0);
            assert_eq!(&curators, &vec![addr0.clone()]);
            assert_eq!(&all_token_ids, &vec!["0".to_string()]);
        }
        _ => panic!("query error"),
    }

    // set_viewing_key
    let info = mock_info("addr0", &[]);
    let msg = ExecuteMsg::SetViewingKey {
        key: "vkey".to_string(),
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info, msg)?;

    // query balance
    let msg = QueryMsg::Balance {
        owner: addr0.clone(),
        viewer: addr0,
        key: "vkey".to_string(),
        token_id: "0".to_string(),
    };
    let q_result = query(deps.as_ref(), mock_env(), msg);
    let q_answer = from_binary::<QueryAnswer>(&q_result?)?;
    match q_answer {
        QueryAnswer::Balance { amount } => assert_eq!(amount, Uint256::from(1000u128)),
        _ => panic!("query error"),
    }

    Ok(())
}

#[test]
fn test_query_tokenid_public_info_sanity() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, deps) = init_helper_default();

    // view public info of fungible token
    let msg = QueryMsg::TokenIdPublicInfo {
        token_id: "0".to_string(),
    };
    let q_result = query(deps.as_ref(), mock_env(), msg);
    let q_answer = from_binary::<QueryAnswer>(&q_result?)?;
    match q_answer {
        QueryAnswer::TokenIdPublicInfo {
            token_id_info,
            total_supply,
            owner,
        } => {
            assert!(serde_json::to_string(&token_id_info)
                .unwrap()
                .contains("\"public_metadata\":{\"token_uri\":\"public uri\""));
            assert_eq!(token_id_info.private_metadata, None);
            assert_eq!(token_id_info.curator, addr.a());
            assert_eq!(total_supply, Some(Uint256::from(1000u128)));
            assert!(owner.is_none());
        }
        _ => panic!("query error"),
    }

    Ok(())
}

#[test]
fn test_query_registered_code_hash() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // register receive addr.a
    let info = mock_info(addr.a().as_str(), &[]);
    let msg_reg_receive = ExecuteMsg::RegisterReceive {
        code_hash: addr.a_hash(),
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info, msg_reg_receive)?;

    let msg_q_code_hash = QueryMsg::RegisteredCodeHash { contract: addr.a() };
    let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_q_code_hash)?)?;
    match q_answer {
        QueryAnswer::RegisteredCodeHash { code_hash } => assert_eq!(code_hash, Some(addr.a_hash())),
        _ => panic!("query error"),
    }

    Ok(())
}

#[test]
fn test_query_balance() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate + curate more tokens
    let (_init_result, mut deps) = init_helper_default();
    let mut info = mock_info("addr0", &[]);
    mint_addtl_default(&mut deps, mock_env(), info.clone())?;

    // cannot view balance without viewing keys
    let msg0_q_bal0_novk = QueryMsg::Balance {
        owner: addr.a(),
        viewer: addr.a(),
        key: "vkeya".to_string(),
        token_id: "0".to_string(),
    };
    let q_answer =
        from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg0_q_bal0_novk)?)?;
    match q_answer {
        QueryAnswer::ViewingKeyError { msg } => {
            assert!(msg.contains("Wrong viewing key for this address or viewing key not set"))
        }
        _ => panic!("query error"),
    }

    // owner can view balance with viewing keys
    // i) generate all viewing keys
    let vks = generate_viewing_keys(&mut deps, mock_env(), info.clone(), addr.all())?;

    // ii) query
    let msg0_q_bal0 = QueryMsg::Balance {
        owner: addr.a(),
        viewer: addr.a(),
        key: vks.a(),
        token_id: "0".to_string(),
    };
    let q_answer =
        from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg0_q_bal0.clone())?)?;
    match q_answer {
        QueryAnswer::Balance { amount } => assert_eq!(amount, Uint256::from(1000u128)),
        _ => panic!("query error"),
    }

    // addr1 cannot view a's balance with b's viewing keys
    let msg1_q_bal0 = QueryMsg::Balance {
        owner: addr.a(),
        viewer: addr.b(),
        key: vks.a(),
        token_id: "0".to_string(),
    };
    let mut q_result = query(deps.as_ref(), mock_env(), msg1_q_bal0.clone());
    assert!(extract_error_msg(&q_result).contains("you do have have permission to view balance"));

    // `b` cannot view `a`'s balance using `b` viewing keys, if `a` gives wrong permission
    let msg_perm_1_wrong = ExecuteMsg::GivePermission {
        allowed_address: addr.b(),
        token_id: "0".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: Some(true),
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(1000u128)),
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg_perm_1_wrong)?;
    q_result = query(deps.as_ref(), mock_env(), msg1_q_bal0.clone());
    assert!(extract_error_msg(&q_result).contains("you do have have permission to view balance"));

    // `b` can view `a`'s balance using `b` viewing keys, once `a` gives correct permission
    let mut env = mock_env();
    info.sender = addr.a();
    let msg_perm_1 = ExecuteMsg::GivePermission {
        allowed_address: addr.b(),
        token_id: "0".to_string(),
        view_balance: Some(true),
        view_balance_expiry: Some(Expiration::AtHeight(env.block.height.add(1))),
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: None,
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg_perm_1)?;
    let q_answer =
        from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg1_q_bal0.clone())?)?;
    match q_answer {
        QueryAnswer::Balance { amount } => assert_eq!(amount, Uint256::from(1000u128)),
        _ => panic!("query error"),
    }

    // `b` cannot view `a`'s token_id "0a" balance, because only got permission for token_id "0"...
    let msg1_q_bal0_0a = QueryMsg::Balance {
        owner: addr.a(),
        viewer: addr.b(),
        key: vks.b(),
        token_id: "0a".to_string(),
    };
    q_result = query(deps.as_ref(), mock_env(), msg1_q_bal0_0a);
    assert!(extract_error_msg(&q_result).contains("you do have have permission to view balance"));

    // ... but `a` can still view its own tokens
    let msg0_q_bal0_0a = QueryMsg::Balance {
        owner: addr.a(),
        viewer: addr.a(),
        key: vks.a(),
        token_id: "0a".to_string(),
    };
    let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg0_q_bal0_0a)?)?;
    match q_answer {
        QueryAnswer::Balance { amount } => assert_eq!(amount, Uint256::from(800u128)),
        _ => panic!("query error"),
    }

    // `c` cannot view `a`'s balance, because `a` gave permission only to `b`
    let msg2_q_bal0 = QueryMsg::Balance {
        owner: addr.a(),
        viewer: addr.c(),
        key: vks.c(),
        token_id: "0a".to_string(),
    };
    q_result = query(deps.as_ref(), mock_env(), msg2_q_bal0);
    assert!(extract_error_msg(&q_result).contains("you do have have permission to view balance"));

    // `b` cannot view `a`'s balance using `b` viewing keys, because [correct] permission expired
    // i) add block height
    env.block.height += 2;
    q_result = query(deps.as_ref(), mock_env(), msg1_q_bal0.clone());
    assert!(q_result.is_ok());
    // ii) a handle must happen in order to trigger the block height change (won't be required once upgraded to CosmWasm v1.0)
    let random_msg = ExecuteMsg::AddCurators {
        add_curators: vec![],
        padding: None,
    };
    execute(deps.as_mut(), env, info, random_msg)?;
    // iii) query now
    q_result = query(deps.as_ref(), mock_env(), msg1_q_bal0);
    assert!(extract_error_msg(&q_result).contains("you do have have permission to view balance"));

    // `a` can still view owns own balance, even after permission given to `b` has expired
    let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg0_q_bal0)?)?;
    match q_answer {
        QueryAnswer::Balance { amount } => assert_eq!(amount, Uint256::from(1000u128)),
        _ => panic!("query error"),
    }

    Ok(())
}

#[test]
fn test_query_all_balance() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    let mut info = mock_info("addr0", &[]);
    mint_addtl_default(&mut deps, mock_env(), info.clone())?;
    let vks = generate_viewing_keys(
        &mut deps,
        mock_env(),
        info.clone(),
        vec![addr.a(), addr.b()],
    )?;

    // addr.b cannot query addr.a's AllBalance
    let msg = QueryMsg::AllBalances {
        owner: addr.a(),
        key: vks.b(),
        tx_history_page: None,
        tx_history_page_size: None,
    };
    let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg)?)?;
    match q_answer {
        QueryAnswer::ViewingKeyError { msg } => {
            assert!(msg.contains("Wrong viewing key for this address or viewing key not set"))
        }
        _ => panic!("query error"),
    }

    // addr.a can query AllBalance
    let msg_q_allbal = QueryMsg::AllBalances {
        owner: addr.a(),
        key: vks.a(),
        tx_history_page: None,
        tx_history_page_size: None,
    };
    let q_answer =
        from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_q_allbal.clone())?)?;
    match q_answer {
        QueryAnswer::AllBalances(i) => assert_eq!(
            i,
            vec![
                OwnerBalance {
                    token_id: "0".to_string(),
                    amount: Uint256::from(1000u128)
                },
                OwnerBalance {
                    token_id: "0a".to_string(),
                    amount: Uint256::from(800u128)
                },
            ]
        ),
        _ => panic!("query error"),
    }

    // mint additional token_id "0", doesn't create another entry in AllBalance
    let msg_mint = ExecuteMsg::MintTokens {
        mint_tokens: vec![TokenAmount {
            token_id: "0".to_string(),
            balances: vec![TokenIdBalance {
                address: addr.a(),
                amount: Uint256::from(100u128),
            }],
        }],
        memo: None,
        padding: None,
    };
    info.sender = addr.a();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_mint)?;
    let q_answer =
        from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_q_allbal.clone())?)?;
    match q_answer {
        QueryAnswer::AllBalances(i) => assert_eq!(
            i,
            vec![
                OwnerBalance {
                    token_id: "0".to_string(),
                    amount: Uint256::from(1100u128)
                },
                OwnerBalance {
                    token_id: "0a".to_string(),
                    amount: Uint256::from(800u128)
                },
            ]
        ),
        _ => panic!("query error"),
    }

    // // curate a list of tokens
    // let mut curate0 = default_curate_value();
    // curate0.token_info.token_id = "test_foo".to_string();
    // let mut curate1 = default_curate_value();
    // curate1.token_info.token_id = "test_bar".to_string();
    // let mut curate2 = default_curate_value();
    // curate2.token_info.token_id = "test_hello".to_string();
    // let mut curate3 = default_curate_value();
    // curate3.token_info.token_id = "test_aha".to_string();
    // let msg_curate = ExecuteMsg::CurateTokenIds {
    //     initial_tokens: vec![curate0, curate1, curate2, curate3],
    //     memo: None,
    //     padding: None,
    // };
    // info.sender = addr.a();
    // execute(deps.as_mut(), mock_env(), info, msg_curate)?;

    // // returns all balances in token_id alphabetical order
    // let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_q_allbal)?)?;
    // match q_answer {
    //     QueryAnswer::AllBalances(i) => assert_eq!(
    //         i,
    //         vec![
    //             OwnerBalance {
    //                 token_id: "0".to_string(),
    //                 amount: Uint256::from(1100u128)
    //             },
    //             OwnerBalance {
    //                 token_id: "0a".to_string(),
    //                 amount: Uint256::from(800u128)
    //             },
    //             OwnerBalance {
    //                 token_id: "test_aha".to_string(),
    //                 amount: Uint256::from(1000u128)
    //             },
    //             OwnerBalance {
    //                 token_id: "test_bar".to_string(),
    //                 amount: Uint256::from(1000u128)
    //             },
    //             OwnerBalance {
    //                 token_id: "test_foo".to_string(),
    //                 amount: Uint256::from(1000u128)
    //             },
    //             OwnerBalance {
    //                 token_id: "test_hello".to_string(),
    //                 amount: Uint256::from(1000u128)
    //             },
    //         ]
    //     ),
    //     _ => panic!("query error"),
    // }

    Ok(())
}

#[test]
fn test_query_transaction_history() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // generate vks
    let mut info = mock_info(addr.a().as_str(), &[]);
    let vks = generate_viewing_keys(
        &mut deps,
        mock_env(),
        info.clone(),
        vec![addr.a(), addr.b()],
    )?;

    // query tx history
    let msg_tx_hist_a_a = QueryMsg::TransactionHistory {
        address: addr.a(),
        key: vks.a(),
        page: None,
        page_size: 10u32,
    };
    let q_answer =
        from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_tx_hist_a_a.clone())?)?;
    match q_answer {
        QueryAnswer::TransactionHistory { txs, total } => {
            match &txs[0].action {
                TxAction::Mint {
                    minter,
                    recipient,
                    amount,
                } => {
                    assert_eq!(minter, &addr.a());
                    assert_eq!(recipient, &addr.a());
                    assert_eq!(amount, &Uint256::from(1000u128));
                }
                _ => panic!("wrong tx history variant"),
            };
            assert_eq!(total, 1_u64);
        }
        _ => panic!("query error"),
    }

    // curate more tokens
    mint_addtl_default(&mut deps, mock_env(), info.clone())?;
    // query tx history
    let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_tx_hist_a_a)?)?;
    if let QueryAnswer::TransactionHistory { txs, total } = q_answer {
        if let TxAction::Mint {
            minter,
            recipient,
            amount,
        } = &txs[0].action
        {
            assert_eq!(minter, &addr.a());
            assert_eq!(recipient, &addr.c());
            assert_eq!(amount, &Uint256::from(1u128));
        }
        if let TxAction::Mint {
            minter,
            recipient,
            amount,
        } = &txs[1].action
        {
            assert_eq!(minter, &addr.a());
            assert_eq!(recipient, &addr.c());
            assert_eq!(amount, &Uint256::from(1u128));
        }
        if let TxAction::Mint {
            minter,
            recipient,
            amount,
        } = &txs[2].action
        {
            assert_eq!(minter, &addr.a());
            assert_eq!(recipient, &addr.b());
            assert_eq!(amount, &Uint256::from(500u128));
        }
        if let TxAction::Mint {
            minter,
            recipient,
            amount,
        } = &txs[3].action
        {
            assert_eq!(minter, &addr.a());
            assert_eq!(recipient, &addr.a());
            assert_eq!(amount, &Uint256::from(800u128));
        }
        assert_eq!(total, 5_u64);
    }

    // transfer token
    let msg_trans = ExecuteMsg::Transfer {
        token_id: "0a".to_string(),
        from: addr.a(),
        recipient: addr.b(),
        amount: Uint256::from(10u128),
        memo: None,
        padding: None,
    };
    info.sender = addr.a();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_trans)?;

    let msg_tx_hist_b_b = QueryMsg::TransactionHistory {
        address: addr.b(),
        key: vks.b(),
        page: None,
        page_size: 10u32,
    };
    let q_answer =
        from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_tx_hist_b_b.clone())?)?;
    if let QueryAnswer::TransactionHistory { txs, total } = q_answer {
        if let TxAction::Transfer {
            from,
            sender,
            recipient,
            amount,
        } = &txs[0].action
        {
            assert_eq!(from, &addr.a());
            assert_eq!(sender, &None);
            assert_eq!(recipient, &addr.b());
            assert_eq!(amount, &Uint256::from(10u128));
        }
        // addr.b has only two records in history
        assert_eq!(total, 2_u64);
    }

    // burn token twice in a single tx -> get two txs in history record
    let msg_burn = ExecuteMsg::BurnTokens {
        burn_tokens: vec![TokenAmount {
            token_id: "0a".to_string(),
            balances: vec![
                TokenIdBalance {
                    address: addr.b(),
                    amount: Uint256::from(3u128),
                },
                TokenIdBalance {
                    address: addr.b(),
                    amount: Uint256::from(4u128),
                },
            ],
        }],
        memo: None,
        padding: None,
    };
    info.sender = addr.a();
    execute(deps.as_mut(), mock_env(), info, msg_burn)?;

    let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_tx_hist_b_b)?)?;
    if let QueryAnswer::TransactionHistory { txs, total } = q_answer {
        if let TxAction::Burn {
            burner,
            owner,
            amount,
        } = &txs[0].action
        {
            assert_eq!(burner, &None);
            assert_eq!(owner, &addr.b());
            assert_eq!(amount, &Uint256::from(4u128));
        }
        if let TxAction::Burn {
            burner,
            owner,
            amount,
        } = &txs[1].action
        {
            assert_eq!(burner, &None);
            assert_eq!(owner, &addr.b());
            assert_eq!(amount, &Uint256::from(3u128));
        }
        // addr.b see two additional history records
        assert_eq!(total, 4_u64);
    }

    Ok(())
}

#[test]
fn test_query_permission() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());
    let addr1 = Addr::unchecked("addr1".to_string());

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // give permission to transfer: addr0 grants addr1
    let mut info = mock_info("addr0", &[]);
    let msg0_perm_1 = ExecuteMsg::GivePermission {
        allowed_address: addr1.clone(),
        token_id: "0".to_string(),
        view_balance: Some(true),
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(10u128)),
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg0_perm_1)?;

    // query permission fails: no viewing key
    let msg_q = QueryMsg::Permission {
        owner: addr0.clone(),
        allowed_address: addr1.clone(),
        key: "vkey".to_string(),
        token_id: "0".to_string(),
    };
    let q_result = query(deps.as_ref(), mock_env(), msg_q.clone());
    let q_answer = from_binary::<QueryAnswer>(&q_result?)?;
    match q_answer {
        QueryAnswer::ViewingKeyError { msg } => {
            assert!(msg.contains("Wrong viewing key for this address or viewing key not set"))
        }
        _ => panic!("query error"),
    }

    // query permission succeeds with owner's viewing key
    // i) set_viewing_key
    info.sender = addr0.clone();
    let msg_vk = ExecuteMsg::SetViewingKey {
        key: "vkey".to_string(),
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg_vk)?;
    // ii) query permissions
    let q_result = query(deps.as_ref(), mock_env(), msg_q);
    let q_answer = from_binary::<QueryAnswer>(&q_result?)?;
    match q_answer {
        QueryAnswer::Permission(perm) => assert_eq!(
            perm.unwrap_or_default(),
            Permission {
                view_balance_perm: true,
                view_balance_exp: Expiration::default(),
                view_pr_metadata_perm: false,
                view_pr_metadata_exp: Expiration::default(),
                trfer_allowance_perm: Uint256::from(10u128),
                trfer_allowance_exp: Expiration::default(),
            }
        ),
        _ => panic!("query error"),
    }

    // query permission succeeds with perm_addr's viewing key
    // i) set_viewing_key
    info.sender = addr1.clone();
    let msg_vk2 = ExecuteMsg::SetViewingKey {
        key: "vkey2".to_string(),
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info, msg_vk2)?;
    // ii) query permissions
    let msg_q2 = QueryMsg::Permission {
        owner: addr0,
        allowed_address: addr1,
        key: "vkey2".to_string(),
        token_id: "0".to_string(),
    };
    let q_result = query(deps.as_ref(), mock_env(), msg_q2);
    let q_answer = from_binary::<QueryAnswer>(&q_result?)?;
    match q_answer {
        QueryAnswer::Permission(perm) => assert_eq!(
            perm.unwrap_or_default(),
            Permission {
                view_balance_perm: true,
                view_balance_exp: Expiration::default(),
                view_pr_metadata_perm: false,
                view_pr_metadata_exp: Expiration::default(),
                trfer_allowance_perm: Uint256::from(10u128),
                trfer_allowance_exp: Expiration::default(),
            }
        ),
        _ => panic!("query error"),
    }

    Ok(())
}

#[test]
fn test_query_all_permissions() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // generate vks
    let info = mock_info(addr.a().as_str(), &[]);
    let vks = generate_viewing_keys(&mut deps, mock_env(), info.clone(), addr.all())?;

    // curate additional tokens
    mint_addtl_default(&mut deps, mock_env(), info.clone())?;

    // give permission to transfer: addr.a grants addr.b
    let msg0_perm_b = ExecuteMsg::GivePermission {
        allowed_address: addr.b(),
        token_id: "0".to_string(),
        view_balance: Some(true),
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: None,
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg0_perm_b)?;

    // give permission to transfer: addr.a grants addr.c
    let msg0_perm_c = ExecuteMsg::GivePermission {
        allowed_address: addr.c(),
        token_id: "0".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: Some(true),
        view_private_metadata_expiry: None,
        transfer: None,
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg0_perm_c)?;

    // give permission to transfer: addr.a grants addr.d
    let msg0_perm_d = ExecuteMsg::GivePermission {
        allowed_address: addr.d(),
        token_id: "0a".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(100u128)),
        transfer_expiry: Some(Expiration::AtHeight(100u64)),
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info, msg0_perm_d)?;

    // addr.a() query AllPermissions
    let msg_q_allperm_a = QueryMsg::AllPermissions {
        address: addr.a(),
        key: vks.a(),
        page: None,
        page_size: 10u32,
    };
    let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_q_allperm_a)?)?;
    if let QueryAnswer::AllPermissions {
        permission_keys,
        permissions,
        total,
    } = q_answer
    {
        assert_eq!(
            permission_keys
                .into_iter()
                .rev()
                .map(|key| key.allowed_addr)
                .collect::<Vec<Addr>>(),
            vec![addr.b(), addr.c(), addr.d()]
        );
        assert_eq!(
            permissions
                .iter()
                .rev()
                .map(|perm| perm.view_balance_perm)
                .collect::<Vec<bool>>(),
            vec![true, false, false]
        );
        assert_eq!(
            permissions
                .iter()
                .rev()
                .map(|perm| perm.view_balance_exp)
                .collect::<Vec<Expiration>>(),
            vec![Expiration::Never; 3]
        );
        assert_eq!(
            permissions
                .iter()
                .rev()
                .map(|perm| perm.view_pr_metadata_perm)
                .collect::<Vec<bool>>(),
            vec![false, true, false]
        );
        assert_eq!(
            permissions
                .iter()
                .rev()
                .map(|perm| perm.view_pr_metadata_exp)
                .collect::<Vec<Expiration>>(),
            vec![Expiration::Never; 3]
        );
        assert_eq!(
            permissions
                .iter()
                .rev()
                .map(|perm| perm.trfer_allowance_perm)
                .collect::<Vec<Uint256>>(),
            vec![
                Uint256::from(0u128),
                Uint256::from(0u128),
                Uint256::from(100u128)
            ]
        );
        assert_eq!(
            permissions
                .iter()
                .rev()
                .map(|perm| perm.trfer_allowance_exp)
                .collect::<Vec<Expiration>>(),
            vec![
                Expiration::Never,
                Expiration::Never,
                Expiration::AtHeight(100u64)
            ]
        );
        assert_eq!(total, 3u64);
    }

    // addr.b() query AllPermissions -> nothing because can only see list of all permissions as granter
    let msg_q_allperm_a = QueryMsg::AllPermissions {
        address: addr.b(),
        key: vks.b(),
        page: None,
        page_size: 10u32,
    };
    let q_answer = from_binary::<QueryAnswer>(&query(deps.as_ref(), mock_env(), msg_q_allperm_a)?)?;
    if let QueryAnswer::AllPermissions {
        permission_keys,
        permissions,
        total,
    } = q_answer
    {
        assert_eq!(permission_keys, vec![]);
        assert_eq!(permissions, vec![]);
        assert_eq!(total, 0u64);
    }

    Ok(())
}

#[test]
fn test_query_tokenid_private_info_sanity() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // generate viewing keys
    let info = mock_info(addr.a().as_str(), &[]);
    let vks = generate_viewing_keys(&mut deps, mock_env(), info, vec![addr.a()])?;

    // view private info of fungible token
    let msg = QueryMsg::TokenIdPrivateInfo {
        address: addr.a(),
        key: vks.a(),
        token_id: "0".to_string(),
    };
    let q_result = query(deps.as_ref(), mock_env(), msg);
    let q_answer = from_binary::<QueryAnswer>(&q_result?)?;
    match q_answer {
        QueryAnswer::TokenIdPrivateInfo {
            token_id_info,
            total_supply,
            owner,
        } => {
            assert!(serde_json::to_string(&token_id_info)
                .unwrap()
                .contains("\"public_metadata\":{\"token_uri\":\"public uri\""));
            assert!(serde_json::to_string(&token_id_info)
                .unwrap()
                .contains("\"private_metadata\":{\"token_uri\":\"private uri\""));
            assert_eq!(token_id_info.curator, addr.a());
            assert_eq!(total_supply, Some(Uint256::from(1000u128)));
            assert!(owner.is_none());
        }
        _ => panic!("query error"),
    }

    Ok(())
}
