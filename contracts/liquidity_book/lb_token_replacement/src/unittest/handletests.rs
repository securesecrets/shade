use super::testhelpers::*;

use super::super::{
    handles::*,
    // msg::*,
    queries::*,
    receiver::{ReceiverHandleMsg, Snip1155ReceiveMsg},
    state::*,
    // state::{expiration::*, metadata::*, permissions::*, state_structs::*},
};

use shade_protocol::lb_libraries::lb_token::{
    expiration::*, metadata::*, permissions::*, state_structs::*,
};
use shade_protocol::liquidity_book::lb_token::*;

use cosmwasm_std::{from_binary, testing::*, to_binary, Addr, Response, StdResult, Uint256};
use secret_toolkit::{crypto::sha_256, permit::RevokedPermits};

/////////////////////////////////////////////////////////////////////////////////
// Tests
/////////////////////////////////////////////////////////////////////////////////

#[test]
fn init_sanity() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());

    // instantiate
    let (init_result, deps) = init_helper_default();
    assert_eq!(init_result.unwrap(), Response::default());

    // check contract config
    let contr_conf = contr_conf_r(&deps.storage).load()?;
    assert_eq!(contr_conf.admin.unwrap(), addr0);
    assert_eq!(contr_conf.curators, vec![addr0.clone()]);
    // 1 minting could have happened, so tx_cnt should == 1:
    assert_eq!(contr_conf.tx_cnt, 1u64);
    let token_id = "0".to_string();

    // check initial balances
    let balance = balances_r(&deps.storage, &token_id).load(to_binary(&addr0)?.as_slice())?;
    assert_eq!(balance, Uint256::from(1000u128));

    Ok(())
}

#[test]
fn curate_token_id_sanity() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());
    let addr1 = Addr::unchecked("addr1".to_string());
    let addr2 = Addr::unchecked("addr2".to_string());

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // curate additional token_ids
    let info = mock_info("addr0", &[]);
    curate_addtl_default(&mut deps, mock_env(), info)?;

    // check balances
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(1000u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "0a", &addr0).unwrap(),
        Uint256::from(800u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "1", &addr1).unwrap(),
        Uint256::from(500u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(1u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2a", &addr2).unwrap(),
        Uint256::from(1u128)
    );
    // 1 initial balance, 4 curate_token_id
    assert_eq!(contr_conf_r(&deps.storage).load()?.tx_cnt, 5u64);

    // initial balance comprehensive check
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(1000u128)
    );
    assert_eq!(chk_bal(&deps.storage, "0", &addr1), None);
    assert_eq!(chk_bal(&deps.storage, "0", &addr2), None);
    assert_eq!(
        chk_bal(&deps.storage, "0a", &addr0).unwrap(),
        Uint256::from(800u128)
    );
    assert_eq!(chk_bal(&deps.storage, "0a", &addr1), None);
    assert_eq!(chk_bal(&deps.storage, "0a", &addr2), None);
    assert_eq!(chk_bal(&deps.storage, "1", &addr0), None);
    assert_eq!(
        chk_bal(&deps.storage, "1", &addr1).unwrap(),
        Uint256::from(500u128)
    );
    assert_eq!(chk_bal(&deps.storage, "1", &addr2), None);
    assert_eq!(chk_bal(&deps.storage, "2", &addr1), None);
    assert_eq!(chk_bal(&deps.storage, "2", &addr1), None);
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(1u128)
    );
    assert_eq!(chk_bal(&deps.storage, "2a", &addr1), None);
    assert_eq!(chk_bal(&deps.storage, "2a", &addr1), None);
    assert_eq!(
        chk_bal(&deps.storage, "2a", &addr2).unwrap(),
        Uint256::from(1u128)
    );

    Ok(())
}

#[test]
fn test_curate_token_id() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());
    let addr1 = Addr::unchecked("addr1".to_string());
    let addr2 = Addr::unchecked("addr2".to_string());

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // curate additional token_ids
    let mut info = mock_info("addr0", &[]);
    curate_addtl_default(&mut deps, mock_env(), info.clone())?;

    // cannot mint more than 1 nft; address != 1
    let mut curate = default_curate_value();
    curate.token_info.token_id = "testa".to_string();
    curate.token_info.token_config = default_token_config_nft();
    curate.balances = vec![
        TokenIdBalance {
            address: addr0.clone(),
            amount: Uint256::from(1u128),
        },
        TokenIdBalance {
            address: addr1.clone(),
            amount: Uint256::from(1u128),
        },
    ];
    let mut msg = ExecuteMsg::CurateTokenIds {
        initial_tokens: vec![curate],
        memo: None,
        padding: None,
    };
    let mut result = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    assert!(extract_error_msg(&result)
        .contains("is an NFT; there can only be one NFT. Balances should only have one address"));

    // cannot mint more than 1 nft; amount != 1
    let mut curate = default_curate_value();
    curate.token_info.token_id = "testb".to_string();
    curate.token_info.token_config = default_token_config_nft();
    curate.balances[0].amount = Uint256::from(2u128);
    msg = ExecuteMsg::CurateTokenIds {
        initial_tokens: vec![curate],
        memo: None,
        padding: None,
    };
    result = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    assert!(extract_error_msg(&result)
        .contains("is an NFT; there can only be one NFT. Balances.amount must == 1"));

    // non-curator cannot curate
    info.sender = addr1.clone();
    let mut curate = default_curate_value();
    curate.token_info.token_id = "testc".to_string();
    msg = ExecuteMsg::CurateTokenIds {
        initial_tokens: vec![curate],
        memo: None,
        padding: None,
    };
    result = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(extract_error_msg(&result).contains("Only curators are allowed to curate"));

    // check balances
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(1000u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "1", &addr1).unwrap(),
        Uint256::from(500u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(1u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2a", &addr2).unwrap(),
        Uint256::from(1u128)
    );
    assert_eq!(chk_bal(&deps.storage, "testa", &addr0), None);
    assert_eq!(chk_bal(&deps.storage, "4", &addr1), None);
    assert_eq!(chk_bal(&deps.storage, "testb", &addr0), None);
    assert_eq!(chk_bal(&deps.storage, "testc", &addr0), None);
    // 1 initial balance, 4 curate_token_id, 0 additional
    assert_eq!(contr_conf_r(&deps.storage).load()?.tx_cnt, 5u64);

    Ok(())
}

#[test]
fn test_mint_tokens() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // curate additional token_ids
    let mut info = mock_info(addr.a().as_str(), &[]);
    curate_addtl_default(&mut deps, mock_env(), info.clone())?;

    // can mint non-existent token_id if you're the curator
    let mint_non_exist = TokenAmount {
        token_id: "test0".to_string(),
        balances: vec![TokenIdBalance {
            address: addr.a(),
            amount: Uint256::from(100u128),
        }],
    };
    let msg = ExecuteMsg::MintTokens {
        mint_tokens: vec![mint_non_exist],
        memo: None,
        padding: None,
    };
    let _result = execute(deps.as_mut(), mock_env(), info.clone(), msg)?;
    assert_eq!(
        chk_bal(&deps.storage, "test0", &addr.a()),
        Some(Uint256::from(100u128))
    );

    // success: mint more fungible tokens to multiple addresses
    let mint = TokenAmount {
        token_id: "0".to_string(),
        balances: vec![
            TokenIdBalance {
                address: addr.a(),
                amount: Uint256::from(10u128),
            },
            TokenIdBalance {
                address: addr.b(),
                amount: Uint256::from(10u128),
            },
        ],
    };
    let msg = ExecuteMsg::MintTokens {
        mint_tokens: vec![mint],
        memo: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg.clone())?;
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr.a()).unwrap(),
        Uint256::from(1010u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr.b()).unwrap(),
        Uint256::from(10u128)
    );
    // 1 initial balance, 4 curate_token_id, 3 mint_token
    assert_eq!(contr_conf_r(&deps.storage).load()?.tx_cnt, 8u64);

    // non-minter cannot mint
    info.sender = addr.b();
    let result = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    // assert!(extract_error_msg(&result).contains("Only minters are allowed to mint"));

    // cannot mint additional nfts
    info.sender = addr.a();
    let mint = TokenAmount {
        token_id: "2".to_string(),
        balances: vec![TokenIdBalance {
            address: addr.a(),
            amount: Uint256::from(1u128),
        }],
    };
    let msg = ExecuteMsg::MintTokens {
        mint_tokens: vec![mint],
        memo: None,
        padding: None,
    };
    let result = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(extract_error_msg(&result).contains("minting is not enabled for this token_id"));
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr.a()).unwrap(),
        Uint256::from(1010u128)
    );
    // 1 initial balance, 4 curate_token_id, 3 mint_token
    assert_eq!(contr_conf_r(&deps.storage).load()?.tx_cnt, 8u64);

    Ok(())
}

#[test]
fn test_burn() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());
    let addr1 = Addr::unchecked("addr1".to_string());
    let addr2 = Addr::unchecked("addr2".to_string());

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // mint additional token_ids
    let mut info = mock_info("addr0", &[]);
    curate_addtl_default(&mut deps, mock_env(), info.clone())?;

    // initial balance check
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(1000u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "1", &addr1).unwrap(),
        Uint256::from(500u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(1u128)
    );

    // burn tokens of another address => should fail
    let burn = TokenAmount {
        token_id: "1".to_string(),
        balances: vec![TokenIdBalance {
            address: addr1.clone(),
            amount: Uint256::from(501u128),
        }],
    };
    let msg = ExecuteMsg::BurnTokens {
        burn_tokens: vec![burn],
        memo: None,
        padding: None,
    };
    let mut result = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    assert!(extract_error_msg(&result).contains("you do not have permission to burn "));

    // burn more tokens than available => should fail
    info.sender = addr1.clone();
    result = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    assert!(extract_error_msg(&result).contains("insufficient funds"));

    // burn fungible tokens should work
    let burn = TokenAmount {
        token_id: "1".to_string(),
        balances: vec![TokenIdBalance {
            address: addr1.clone(),
            amount: Uint256::from(300u128),
        }],
    };
    let msg = ExecuteMsg::BurnTokens {
        burn_tokens: vec![burn],
        memo: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg)?;
    assert_eq!(
        chk_bal(&deps.storage, "1", &addr1).unwrap(),
        Uint256::from(200u128)
    );

    // burn nft should work
    info.sender = addr2.clone();
    let burn = TokenAmount {
        token_id: "2".to_string(),
        balances: vec![TokenIdBalance {
            address: addr2.clone(),
            amount: Uint256::from(1u128),
        }],
    };
    let msg = ExecuteMsg::BurnTokens {
        burn_tokens: vec![burn],
        memo: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info, msg)?;
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(0u128)
    );

    // 1 initial balance, 4 curate_token_id, 2 burns
    assert_eq!(contr_conf_r(&deps.storage).load()?.tx_cnt, 7u64);

    Ok(())
}

#[test]
fn test_change_metadata_nft() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();
    // addr.a() = admin;
    // addr.b() = curator;
    // addr.c() = owner;
    // addr.d() = new owner for testnft0; minter for testnft2;

    // custom instantiate
    let mut deps = mock_dependencies();
    let mut info = mock_info(addr.a().as_str(), &[]);

    let init_msg = InstantiateMsg {
        has_admin: true,
        admin: None, // None -> sender defaults as admin
        curators: vec![addr.b()],
        entropy: "seedentropy".to_string(),
        initial_tokens: vec![default_curate_value()],
        lb_pair_info: LbPair {
            name: String::new(),
            symbol: String::new(),
            lb_pair_address: Addr::unchecked("address"),
            decimals: 18,
        },
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)?;

    // curate three nfts: one which owner can change metadata...
    let mut curate0 = default_curate_value();
    curate0.token_info.token_id = "testnft0".to_string();
    curate0.token_info.token_config = default_token_config_nft();
    curate0.balances = vec![TokenIdBalance {
        address: addr.c(),
        amount: Uint256::from(1u128),
    }];

    // ... one which owner cannot change metadata...
    let mut curate1 = default_curate_value();
    curate1.token_info.token_id = "testnft1".to_string();
    curate1.token_info.token_config = default_token_config_nft();
    let mut flat_config = curate1.token_info.token_config.flatten();
    flat_config.owner_may_update_metadata = false;
    curate1.token_info.token_config = flat_config.to_enum();
    curate1.balances = vec![TokenIdBalance {
        address: addr.c(),
        amount: Uint256::from(1u128),
    }];

    // ... and one where minter can change metadata (and owner cannot)
    let mut curate2 = default_curate_value();
    curate2.token_info.token_id = "testnft2".to_string();
    curate2.token_info.token_config = default_token_config_nft();
    let mut flat_config = curate2.token_info.token_config.flatten();
    flat_config.minters = vec![addr.d()];
    flat_config.owner_may_update_metadata = false;
    curate2.token_info.token_config = flat_config.to_enum();
    curate2.balances = vec![TokenIdBalance {
        address: addr.c(),
        amount: Uint256::from(1u128),
    }];

    let msg_curate = ExecuteMsg::CurateTokenIds {
        initial_tokens: vec![curate0, curate1, curate2],
        memo: None,
        padding: None,
    };
    info.sender = addr.b();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_curate)?;

    // error: admin cannot change nft metadata if not owner
    let msg_change_metadata = ExecuteMsg::ChangeMetadata {
        token_id: "testnft0".to_string(),
        public_metadata: Box::new(Some(Metadata {
            token_uri: Some("new public uri for testnft0".to_string()),
            extension: Some(Extension::default()),
        })),
        private_metadata: Box::new(None),
    };
    info.sender = addr.a();
    let mut result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    );
    assert!(
        extract_error_msg(&result).contains("unable to change the metadata for token_id testnft0")
    );

    // error: curator cannot change nft metadata if not owner
    info.sender = addr.b();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    );
    assert!(
        extract_error_msg(&result).contains("unable to change the metadata for token_id testnft0")
    );

    // error: random non-owner cannot change metadata
    info.sender = addr.d();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    );
    assert!(
        extract_error_msg(&result).contains("unable to change the metadata for token_id testnft0")
    );

    // error: nft owner cannot change metadata if config doesn't allow
    let msg_change_metadata_nft1 = ExecuteMsg::ChangeMetadata {
        token_id: "testnft1".to_string(),
        public_metadata: Box::new(None),
        private_metadata: Box::new(None),
    };
    info.sender = addr.c();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata_nft1,
    );
    assert!(
        extract_error_msg(&result).contains("unable to change the metadata for token_id testnft1")
    );

    // success: nft owner can change metadata if config allows...
    info.sender = addr.c();
    execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    )?;
    // check public metadata has changed
    let tkn_info = tkn_info_r(&deps.storage).load("testnft0".to_string().as_bytes())?;
    assert_eq!(
        tkn_info.public_metadata,
        Some(Metadata {
            token_uri: Some("new public uri for testnft0".to_string()),
            extension: Some(Extension::default()),
        })
    );
    // check private metadata unchanged because input is None
    assert_eq!(
        tkn_info.private_metadata,
        Some(Metadata {
            token_uri: Some("private uri".to_string()),
            extension: Some(Extension::default()),
        })
    );
    // transfer nft to a different owner...
    let msg_trans = ExecuteMsg::Transfer {
        token_id: "testnft0".to_string(),
        from: addr.c(),
        recipient: addr.d(),
        amount: Uint256::from(1u128),
        memo: None,
        padding: None,
    };
    info.sender = addr.c();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_trans)?;

    // ...error: old nft owner cannot change metadata
    info.sender = addr.c();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    );
    assert!(
        extract_error_msg(&result).contains("unable to change the metadata for token_id testnft0")
    );

    // success: new nft owner can change metadata
    info.sender = addr.d();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_change_metadata)?;

    // additional nft tests:
    // testnft2 token, where minter can change metadata, but owner cannot
    let msg_change_metadata_nft2 = ExecuteMsg::ChangeMetadata {
        token_id: "testnft2".to_string(),
        public_metadata: Box::new(None),
        private_metadata: Box::new(Some(Metadata {
            token_uri: Some("new private uri for testnft2".to_string()),
            extension: Some(Extension::default()),
        })),
    };
    // admin cannot change metadata
    info.sender = addr.a();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata_nft2.clone(),
    );
    assert!(
        extract_error_msg(&result).contains("unable to change the metadata for token_id testnft2")
    );

    // token_id curator cannot change metadata
    info.sender = addr.b();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata_nft2.clone(),
    );
    assert!(
        extract_error_msg(&result).contains("unable to change the metadata for token_id testnft2")
    );

    // owner cannot change metadata
    info.sender = addr.c();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata_nft2.clone(),
    );
    assert!(
        extract_error_msg(&result).contains("unable to change the metadata for token_id testnft2")
    );

    // success: minter can change metadata
    info.sender = addr.d();
    execute(deps.as_mut(), mock_env(), info, msg_change_metadata_nft2)?;
    // check public metadata unchanged because input is None
    let tkn_info = tkn_info_r(&deps.storage).load("testnft2".to_string().as_bytes())?;
    assert_eq!(
        tkn_info.public_metadata,
        Some(Metadata {
            token_uri: Some("public uri".to_string()),
            extension: Some(Extension::default()),
        })
    );
    // check public metadata has changed
    assert_eq!(
        tkn_info.private_metadata,
        Some(Metadata {
            token_uri: Some("new private uri for testnft2".to_string()),
            extension: Some(Extension::default()),
        })
    );

    Ok(())
}

#[test]
fn test_change_metadata_fungible() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();
    // addr.a() = admin;
    // addr.b() = curator;
    // addr.c() = minter;
    // addr.d() = owner / new minter;

    // custom instantiate
    let mut deps = mock_dependencies();
    let mut info = mock_info(addr.a().as_str(), &[]);

    let init_msg = InstantiateMsg {
        has_admin: true,
        admin: None, // None -> sender defaults as admin
        curators: vec![addr.b()],
        entropy: "seedentropy".to_string(),
        initial_tokens: vec![default_curate_value()],
        lb_pair_info: LbPair {
            name: String::new(),
            symbol: String::new(),
            lb_pair_address: Addr::unchecked("address"),
            decimals: 18,
        },
    };
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)?;

    // curate two fungible tokens: one which owner can change metadata, and one which owner cannot
    let mut curate0 = default_curate_value();
    curate0.token_info.token_id = "test0".to_string();
    curate0.token_info.token_config = default_token_config_fungible();
    let mut flat_config = curate0.token_info.token_config.flatten();
    flat_config.minters = vec![addr.c()];
    curate0.token_info.token_config = flat_config.to_enum();
    curate0.balances = vec![TokenIdBalance {
        address: addr.d(),
        amount: Uint256::from(1000u128),
    }];

    let mut curate1 = default_curate_value();
    curate1.token_info.token_id = "test1".to_string();
    curate1.token_info.token_config = default_token_config_fungible();
    let mut flat_config = curate1.token_info.token_config.flatten();
    flat_config.minters = vec![addr.c()];
    flat_config.minter_may_update_metadata = false;
    curate1.token_info.token_config = flat_config.to_enum();
    curate1.balances = vec![TokenIdBalance {
        address: addr.d(),
        amount: Uint256::from(1000u128),
    }];

    let msg_curate = ExecuteMsg::CurateTokenIds {
        initial_tokens: vec![curate0, curate1],
        memo: None,
        padding: None,
    };
    info.sender = addr.b();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_curate)?;

    // error: admin cannot change metadata if not minter
    let msg_change_metadata = ExecuteMsg::ChangeMetadata {
        token_id: "test0".to_string(),
        public_metadata: Box::new(Some(Metadata {
            token_uri: Some("new public uri".to_string()),
            extension: Some(Extension::default()),
        })),
        private_metadata: Box::new(None),
    };
    info.sender = addr.a();
    let mut result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    );
    assert!(extract_error_msg(&result).contains("unable to change the metadata for token_id test0"));

    // error: curator cannot change nft metadata if not minter
    info.sender = addr.b();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    );
    assert!(extract_error_msg(&result).contains("unable to change the metadata for token_id test0"));

    // error: owner cannot change metadata
    info.sender = addr.d();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    );
    assert!(extract_error_msg(&result).contains("unable to change the metadata for token_id test0"));

    // error: minter cannot change metadata if config doesn't allow
    let msg_change_metadata_test1 = ExecuteMsg::ChangeMetadata {
        token_id: "test1".to_string(),
        public_metadata: Box::new(None),
        private_metadata: Box::new(None),
    };
    info.sender = addr.c();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata_test1,
    );
    assert!(extract_error_msg(&result).contains("unable to change the metadata for token_id test1"));

    // success: minter can change metadata if config allows
    info.sender = addr.c();
    execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_metadata.clone(),
    )?;
    // check public metadata has changed
    let tkn_info = tkn_info_r(&deps.storage).load("test0".to_string().as_bytes())?;
    assert_eq!(
        tkn_info.public_metadata,
        Some(Metadata {
            token_uri: Some("new public uri".to_string()),
            extension: Some(Extension::default()),
        })
    );
    // check private metadata unchanged because input is None
    assert_eq!(
        tkn_info.private_metadata,
        Some(Metadata {
            token_uri: Some("private uri".to_string()),
            extension: Some(Extension::default()),
        })
    );

    // // admin can add minter...
    // let msg_add_minter = ExecuteMsg::AddMinters {
    //     token_id: "test0".to_string(),
    //     add_minters: vec![addr.d()],
    //     padding: None,
    // };
    // info.sender = addr.a();
    // execute(deps.as_mut(), mock_env(), info.clone(), msg_add_minter)?;

    // // ...admin can remove minter
    // let msg_remove_minter = ExecuteMsg::RemoveMinters {
    //     token_id: "test0".to_string(),
    //     remove_minters: vec![addr.c()],
    //     padding: None,
    // };
    // info.sender = addr.a();
    // execute(deps.as_mut(), mock_env(), info.clone(), msg_remove_minter)?;

    // // ...error: old minter cannot change metadata
    // info.sender = addr.c();
    // result = execute(
    //     deps.as_mut(),
    //     mock_env(),
    //     info.clone(),
    //     msg_change_metadata.clone(),
    // );
    // assert!(extract_error_msg(&result).contains("unable to change the metadata for token_id test0"));

    // // success: new minter can change metadata
    // info.sender = addr.d();
    // execute(deps.as_mut(), mock_env(), info, msg_change_metadata)?;

    Ok(())
}

#[test]
fn test_transfer() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());
    let addr1 = Addr::unchecked("addr1".to_string());
    let addr2 = Addr::unchecked("addr2".to_string());

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // mint additional token_ids
    let mut info = mock_info("addr0", &[]);
    curate_addtl_default(&mut deps, mock_env(), info.clone())?;

    // initial balance check
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(1u128)
    );
    assert_eq!(chk_bal(&deps.storage, "2", &addr1), None);
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(1000u128)
    );
    assert_eq!(chk_bal(&deps.storage, "0", &addr1), None);

    // transfer fungible token "tkn0"
    let msg = ExecuteMsg::Transfer {
        token_id: "0".to_string(),
        from: addr0.clone(),
        recipient: addr1.clone(),
        amount: Uint256::from(800u128),
        memo: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg.clone())?;
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(200u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr1).unwrap(),
        Uint256::from(800u128)
    );

    // cannot transfer if not owner
    info.sender = addr2.clone();
    let result = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    assert!(extract_error_msg(&result)
        .contains("These tokens do not exist or you have no permission to transfer"));

    // transfer NFT "tkn2"; amount != 1
    info.sender = addr2.clone();
    let msg = ExecuteMsg::Transfer {
        token_id: "2".to_string(),
        from: addr2.clone(),
        recipient: addr1.clone(),
        amount: Uint256::from(0u128),
        memo: None,
        padding: None,
    };
    let result = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    assert!(extract_error_msg(&result).contains("NFT amount must == 1"));

    // transfer NFT "tkn2"; should succeed
    let msg = ExecuteMsg::Transfer {
        token_id: "2".to_string(),
        from: addr2.clone(),
        recipient: addr1.clone(),
        amount: Uint256::from(1u128),
        memo: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info, msg)?;

    // final balance check
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(0u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr1).unwrap(),
        Uint256::from(1u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(200u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr1).unwrap(),
        Uint256::from(800u128)
    );
    // 1 initial balance, 4 curate_token_id, 2 transfers
    assert_eq!(contr_conf_r(&deps.storage).load()?.tx_cnt, 7u64);

    Ok(())
}

#[test]
fn test_send() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // initial balance check
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr.a()).unwrap(),
        Uint256::from(1000u128)
    );

    // `send` token_id "0" with msg
    let info = mock_info(addr.a().as_str(), &[]);
    let msg = ExecuteMsg::Send {
        token_id: "0".to_string(),
        from: addr.a(),
        recipient: addr.b(),
        recipient_code_hash: Some(addr.b_hash()),
        amount: Uint256::from(800u128),
        msg: Some(to_binary(&"msg_str")?),
        memo: None,
        padding: None,
    };
    let response = execute(deps.as_mut(), mock_env(), info, msg)?;
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr.a()).unwrap(),
        Uint256::from(200u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr.b()).unwrap(),
        Uint256::from(800u128)
    );
    let (receiver_msg, receiver_addr, receiver_hash) =
        extract_cosmos_msg::<ReceiverHandleMsg>(&response.messages[0].msg)?;
    assert_eq!(receiver_addr, Some(addr.b()));
    assert_eq!(receiver_hash, &addr.b_hash());
    let exp_receive_msg = Snip1155ReceiveMsg {
        sender: addr.a(),
        token_id: "0".to_string(),
        from: addr.a(),
        amount: Uint256::from(800u128),
        memo: None,
        msg: Some(to_binary(&"msg_str")?),
    };
    match receiver_msg {
        ReceiverHandleMsg::Snip1155Receive(i) => assert_eq!(i, exp_receive_msg),
    }

    Ok(())
}

/// note: tested more extensively in integration tests
#[test]
fn test_batch_transfer_and_send_sanity() -> StdResult<()> {
    //init addresses
    let addr = init_addrs();

    //instantiate
    let (_init_result, mut deps) = init_helper_default();

    // curate new tokens
    let info = mock_info("addr0", &[]);
    curate_addtl_default(&mut deps, mock_env(), info.clone())?;

    // initial balances
    assert_eq!(chk_bal(&deps.storage, "0", &addr.c()), None);
    assert_eq!(chk_bal(&deps.storage, "0a", &addr.c()), None);

    // can batch transfer
    let msg_batch_trans = ExecuteMsg::BatchTransfer {
        actions: vec![
            TransferAction {
                token_id: "0".to_string(),
                from: addr.a(),
                recipient: addr.b(),
                amount: Uint256::from(10u128),
                memo: None,
            },
            TransferAction {
                token_id: "0a".to_string(),
                from: addr.a(),
                recipient: addr.c(),
                amount: Uint256::from(20u128),
                memo: None,
            },
        ],
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg_batch_trans)?;

    assert_eq!(
        chk_bal(&deps.storage, "0", &addr.b()),
        Some(Uint256::from(10u128))
    );
    assert_eq!(
        chk_bal(&deps.storage, "0a", &addr.c()),
        Some(Uint256::from(20u128))
    );

    // can batch send
    let msg_batch_send = ExecuteMsg::BatchSend {
        actions: vec![
            SendAction {
                token_id: "0".to_string(),
                from: addr.a(),
                recipient: addr.b(),
                recipient_code_hash: Some(addr.b_hash()),
                amount: Uint256::from(20u128),
                msg: Some(to_binary(&"test message to b")?),
                memo: None,
            },
            SendAction {
                token_id: "0a".to_string(),
                from: addr.a(),
                recipient: addr.c(),
                recipient_code_hash: Some(addr.c_hash()),
                amount: Uint256::from(30u128),
                msg: Some(to_binary(&"test message to c")?),
                memo: None,
            },
        ],
        padding: None,
    };
    let response = execute(deps.as_mut(), mock_env(), info, msg_batch_send)?;

    // check balances
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr.b()),
        Some(Uint256::from(30u128))
    );
    assert_eq!(
        chk_bal(&deps.storage, "0a", &addr.c()),
        Some(Uint256::from(50u128))
    );

    // check inter-contract messages
    let (receiver_msg_b, receiver_addr_b, receiver_hash_b) =
        extract_cosmos_msg::<ReceiverHandleMsg>(&response.messages[0].msg)?;
    assert_eq!(receiver_addr_b, Some(addr.b()));
    assert_eq!(receiver_hash_b, &addr.b_hash());
    let exp_receive_msg_b = Snip1155ReceiveMsg {
        sender: addr.a(),
        token_id: "0".to_string(),
        from: addr.a(),
        amount: Uint256::from(20u128),
        memo: None,
        msg: Some(to_binary(&"test message to b")?),
    };
    match receiver_msg_b {
        ReceiverHandleMsg::Snip1155Receive(i) => assert_eq!(i, exp_receive_msg_b),
    }

    let (receiver_msg_c, receiver_addr_c, receiver_hash_c) =
        extract_cosmos_msg::<ReceiverHandleMsg>(&response.messages[1].msg)?;
    assert_eq!(receiver_addr_c, Some(addr.c()));
    assert_eq!(receiver_hash_c, &addr.c_hash());
    let exp_receive_msg_c = Snip1155ReceiveMsg {
        sender: addr.a(),
        token_id: "0a".to_string(),
        from: addr.a(),
        amount: Uint256::from(30u128),
        memo: None,
        msg: Some(to_binary(&"test message to c")?),
    };
    match receiver_msg_c {
        ReceiverHandleMsg::Snip1155Receive(i) => assert_eq!(i, exp_receive_msg_c),
    }

    Ok(())
}

/// note: tested more extensively in integration tests
#[test]
fn test_batch_transfer_and_send_errors() -> StdResult<()> {
    //init addresses
    let addr = init_addrs();

    //instantiate
    let (_init_result, mut deps) = init_helper_default();

    // cannot batch transfer 0a because it does not exist
    let msg_batch_trans = ExecuteMsg::BatchTransfer {
        actions: vec![
            TransferAction {
                token_id: "0".to_string(),
                from: addr.a(),
                recipient: addr.b(),
                amount: Uint256::from(10u128),
                memo: None,
            },
            TransferAction {
                token_id: "0a".to_string(),
                from: addr.a(),
                recipient: addr.c(),
                amount: Uint256::from(20u128),
                memo: None,
            },
        ],
        padding: None,
    };
    let info = mock_info("addr0", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg_batch_trans);
    assert!(extract_error_msg(&result)
        .contains("These tokens do not exist or you have no permission to transfer"));

    Ok(())
}

#[test]
fn test_transfer_permissions_fungible() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());
    let addr0_bin = to_binary(&addr0)?;
    let _addr0_u8 = addr0_bin.as_slice();
    let addr1 = Addr::unchecked("addr1".to_string());
    let addr1_bin = to_binary(&addr1)?;
    let addr1_u8 = addr1_bin.as_slice();
    let addr2 = Addr::unchecked("addr2".to_string());
    let addr2_bin = to_binary(&addr2)?;
    let addr2_u8 = addr2_bin.as_slice();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // initial balance check
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(1000u128)
    );

    // cannot transfer without allowance
    let mut info = mock_info("addr1", &[]);
    let msg_trnsf_0 = ExecuteMsg::Transfer {
        token_id: "0".to_string(),
        from: addr0.clone(),
        recipient: addr1.clone(),
        amount: Uint256::from(10u128),
        memo: None,
        padding: None,
    };
    let mut result = execute(deps.as_mut(), mock_env(), info.clone(), msg_trnsf_0.clone());
    assert!(extract_error_msg(&result)
        .contains("These tokens do not exist or you have no permission to transfer"));

    // cannot transfer with insufficient allowance
    info.sender = addr0.clone();
    let msg0_perm_1 = ExecuteMsg::GivePermission {
        allowed_address: addr1.clone(),
        token_id: "0".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(11u128)),
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg0_perm_1)?;
    // check that old permission gets replaced if a new one is granted
    let msg0_perm_1 = ExecuteMsg::GivePermission {
        allowed_address: addr1.clone(),
        token_id: "0".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(9u128)),
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg0_perm_1)?;

    info.sender = addr1.clone();
    result = execute(deps.as_mut(), mock_env(), info.clone(), msg_trnsf_0.clone());
    assert!(extract_error_msg(&result).contains("Insufficient transfer allowance: 9"));

    // cannot transfer with wrong allowances: wrong spender address: addr2 has the transfer permission
    info.sender = addr0.clone();
    let msg0_perm_2 = ExecuteMsg::GivePermission {
        allowed_address: addr2.clone(),
        token_id: "0".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(15u128)),
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg0_perm_2)?;

    info.sender = addr1.clone();
    result = execute(deps.as_mut(), mock_env(), info.clone(), msg_trnsf_0.clone());
    assert!(extract_error_msg(&result).contains("Insufficient transfer allowance: "));

    // cannot transfer with wrong allowances: wrong owner address: addr1 giving permission
    info.sender = addr1.clone();
    let msg1_perm_1 = ExecuteMsg::GivePermission {
        allowed_address: addr1.clone(),
        token_id: "0".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(10u128)),
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg1_perm_1)?;
    result = execute(deps.as_mut(), mock_env(), info.clone(), msg_trnsf_0.clone());
    assert!(extract_error_msg(&result).contains("Insufficient transfer allowance: "));

    // can transfer with enough allowance: addr2 has enough allowance
    info.sender = addr2;
    execute(deps.as_mut(), mock_env(), info.clone(), msg_trnsf_0.clone())?;
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(990u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr1).unwrap(),
        Uint256::from(10u128)
    );

    // allowance gets consumed: cannot exceed allowance with a second tx
    result = execute(deps.as_mut(), mock_env(), info.clone(), msg_trnsf_0.clone());
    assert!(extract_error_msg(&result).contains("Insufficient transfer allowance: 5"));

    // allowance for different address does not get consumed
    assert_eq!(
        perm_r(&deps.storage, &addr0, "0")
            .load(addr1_u8)?
            .trfer_allowance_perm,
        Uint256::from(9u128)
    );
    assert_eq!(
        perm_r(&deps.storage, &addr0, "0")
            .load(addr2_u8)?
            .trfer_allowance_perm,
        Uint256::from(5u128)
    );
    assert_eq!(
        perm_r(&deps.storage, &addr1, "0")
            .load(addr1_u8)?
            .trfer_allowance_perm,
        Uint256::from(10u128)
    );

    // owner can transfer regardless of allowance
    info.sender = addr0.clone();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_trnsf_0.clone())?;
    execute(deps.as_mut(), mock_env(), info, msg_trnsf_0)?;
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr0).unwrap(),
        Uint256::from(970u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "0", &addr1).unwrap(),
        Uint256::from(30u128)
    );
    // 1 initial balance, 3 transfers
    assert_eq!(contr_conf_r(&deps.storage).load()?.tx_cnt, 4u64);

    Ok(())
}

#[test]
fn test_transfer_permissions_nft() -> StdResult<()> {
    // init addresses
    let addr0 = Addr::unchecked("addr0".to_string());
    let addr0_bin = to_binary(&addr0)?;
    let _addr0_u8 = addr0_bin.as_slice();
    let addr1 = Addr::unchecked("addr1".to_string());
    let addr1_bin = to_binary(&addr1)?;
    let addr1_u8 = addr1_bin.as_slice();
    let addr2 = Addr::unchecked("addr2".to_string());
    let addr2_bin = to_binary(&addr2)?;
    let _addr2_u8 = addr2_bin.as_slice();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // curate additional token_ids
    let mut info = mock_info("addr0", &[]);
    curate_addtl_default(&mut deps, mock_env(), info.clone())?;

    // cannot transfer: no permission
    info.sender = addr1.clone();
    let msg1_trnsf_0 = ExecuteMsg::Transfer {
        token_id: "2".to_string(),
        from: addr2.clone(),
        recipient: addr0.clone(),
        amount: Uint256::from(1u128),
        memo: None,
        padding: None,
    };
    let mut result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg1_trnsf_0.clone(),
    );
    assert!(extract_error_msg(&result)
        .contains("These tokens do not exist or you have no permission to transfer"));
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(1u128)
    );
    assert_eq!(chk_bal(&deps.storage, "2", &addr0), None);

    // give permission to transfer
    info.sender = addr2.clone();
    let msg2_perm_1 = ExecuteMsg::GivePermission {
        allowed_address: addr1.clone(),
        token_id: "2".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(10u128)),
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg2_perm_1)?;

    // addr1 can now transfer addr2's nft to addr0
    info.sender = addr1.clone();
    execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg1_trnsf_0.clone(),
    )?;
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(0u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr0).unwrap(),
        Uint256::from(1u128)
    );

    // cannot transfer again: insufficient funds
    result = execute(deps.as_mut(), mock_env(), info.clone(), msg1_trnsf_0);
    assert!(extract_error_msg(&result).contains("insufficient funds"));
    // balance is unchanged
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr2).unwrap(),
        Uint256::from(0u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2", &addr0).unwrap(),
        Uint256::from(1u128)
    );

    // give permission to transfern token 2a
    info.sender = addr2.clone();
    let msg2_perm_1 = ExecuteMsg::GivePermission {
        allowed_address: addr1.clone(),
        token_id: "2a".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(1u128)),
        transfer_expiry: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg2_perm_1)?;
    // double check that addr1 has permission to transfer token 2a
    assert_eq!(
        perm_r(&deps.storage, &addr2, "2a").load(addr1_u8)?,
        Permission {
            view_balance_perm: false,
            view_balance_exp: Expiration::default(),
            view_pr_metadata_perm: false,
            view_pr_metadata_exp: Expiration::default(),
            trfer_allowance_perm: Uint256::from(1u128),
            trfer_allowance_exp: Expiration::default(),
        }
    );

    // addr2 transfers away token 2a
    info.sender = addr2.clone();
    let msg = ExecuteMsg::Transfer {
        token_id: "2a".to_string(),
        from: addr2.clone(),
        recipient: addr0.clone(),
        amount: Uint256::from(1u128),
        memo: None,
        padding: None,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), msg)?;
    assert_eq!(
        chk_bal(&deps.storage, "2a", &addr2).unwrap(),
        Uint256::from(0u128)
    );
    assert_eq!(
        chk_bal(&deps.storage, "2a", &addr0).unwrap(),
        Uint256::from(1u128)
    );

    // user1 cannot transfer nft (now owned by user0), even though it previously had allowance
    // when it was owned by user2
    info.sender = addr1.clone();
    let msg = ExecuteMsg::Transfer {
        token_id: "2a".to_string(),
        from: addr0.clone(),
        recipient: addr1.clone(),
        amount: Uint256::from(1u128),
        memo: None,
        padding: None,
    };
    result = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(extract_error_msg(&result)
        .contains("These tokens do not exist or you have no permission to transfer"));
    assert_eq!(chk_bal(&deps.storage, "2a", &addr1), None);
    assert_eq!(
        chk_bal(&deps.storage, "2a", &addr0).unwrap(),
        Uint256::from(1u128)
    );

    Ok(())
}

#[test]
fn test_revoke_permission_sanity() -> StdResult<()> {
    //init addresses
    let addr = init_addrs();

    //instantiate
    let (_init_result, mut deps) = init_helper_default();

    // give permission
    let msg0_perm_b = ExecuteMsg::GivePermission {
        allowed_address: addr.b(),
        token_id: "0".to_string(),
        view_balance: None,
        view_balance_expiry: None,
        view_private_metadata: None,
        view_private_metadata_expiry: None,
        transfer: Some(Uint256::from(10u128)),
        transfer_expiry: None,
        padding: None,
    };
    let mut info = mock_info("addr0", &[]);
    execute(deps.as_mut(), mock_env(), info.clone(), msg0_perm_b)?;

    let vks = generate_viewing_keys(
        &mut deps,
        mock_env(),
        info.clone(),
        vec![addr.a(), addr.b()],
    )?;

    let q_answer = from_binary::<QueryAnswer>(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Permission {
            owner: addr.a(),
            allowed_address: addr.b(),
            key: vks.a(),
            token_id: "0".to_string(),
        },
    )?)?;
    match q_answer {
        QueryAnswer::Permission(perm) => {
            assert_eq!(perm.unwrap().trfer_allowance_perm, Uint256::from(10u128))
        }
        _ => panic!("query error"),
    }

    // addr.b can revoke (renounce) permission it has been given
    let msg_revoke = ExecuteMsg::RevokePermission {
        token_id: "0".to_string(),
        owner: addr.a(),
        allowed_address: addr.b(),
        padding: None,
    };
    info.sender = addr.b();
    execute(deps.as_mut(), mock_env(), info, msg_revoke)?;
    let q_answer = from_binary::<QueryAnswer>(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Permission {
            owner: addr.a(),
            allowed_address: addr.b(),
            key: vks.a(),
            token_id: "0".to_string(),
        },
    )?)?;
    match q_answer {
        QueryAnswer::Permission(perm) => {
            assert_eq!(perm.unwrap().trfer_allowance_perm, Uint256::from(0u128))
        }
        _ => panic!("query error"),
    }

    Ok(())
}

#[test]
fn test_create_and_set_viewing_keys_sanity() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // create vk
    let msg_create_vk = ExecuteMsg::CreateViewingKey {
        entropy: "foobar".to_string(),
        padding: None,
    };
    let mut info = mock_info(addr.a().as_str(), &[]);
    let response = execute(deps.as_mut(), mock_env(), info.clone(), msg_create_vk)?;
    let response_data = from_binary::<ExecuteAnswer>(&response.data.unwrap())?;
    match response_data {
        ExecuteAnswer::CreateViewingKey { key } => assert!(key.contains("api_key_")),
        _ => panic!("expected HandleAnswer:CreateViewingKey variant"),
    }

    // set vk
    let msg_set_vk = ExecuteMsg::SetViewingKey {
        key: "foobar".to_string(),
        padding: None,
    };
    info.sender = addr.b();
    execute(deps.as_mut(), mock_env(), info, msg_set_vk)?;
    let vk = read_viewing_key_hash(&deps.storage, addr.b().as_str()).unwrap_or_default();
    let exp_vk = sha_256("foobar".as_bytes());
    assert_eq!(vk.as_slice(), exp_vk);

    Ok(())
}

/// permit queries tested more extensively in integration test. Because in current Secret Network API (at the time of writing),
/// permits will always pass in unit tests
#[test]
fn test_revoke_permit_sanity() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // revoke permit
    let msg = ExecuteMsg::RevokePermit {
        permit_name: "testpermit".to_string(),
        padding: None,
    };
    let info = mock_info(addr.a().as_str(), &[]);
    execute(deps.as_mut(), mock_env(), info, msg)?;

    // check that permit is revoked
    assert!(RevokedPermits::is_permit_revoked(
        &deps.storage,
        PREFIX_REVOKED_PERMITS,
        addr.a().as_str(),
        "testpermit"
    ));

    Ok(())
}

#[test]
fn test_add_remove_curators() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // non-curator cannot curate new token_ids
    let mut info = mock_info(addr.b().as_str(), &[]);
    let mut curate0 = default_curate_value();
    curate0.token_info.token_id = "test0".to_string();
    let msg_curate = ExecuteMsg::CurateTokenIds {
        initial_tokens: vec![curate0],
        memo: None,
        padding: None,
    };
    let mut result = execute(deps.as_mut(), mock_env(), info.clone(), msg_curate.clone());
    assert!(extract_error_msg(&result).contains("Only curators are allowed to curate token_ids"));

    // admin adds 2 curators...
    info.sender = addr.a();
    let msg_add_curators = ExecuteMsg::AddCurators {
        add_curators: vec![addr.b(), addr.c()],
        padding: None,
    };
    execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_add_curators.clone(),
    )?;
    assert_eq!(chk_bal(&deps.storage, "test0", &addr.a()), None);

    // ...then new curator addr.b can curate new token_id
    info.sender = addr.b();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_curate)?;
    assert_eq!(
        chk_bal(&deps.storage, "test0", &addr.a()),
        Some(Uint256::from(1000u128))
    );

    // addr.b is curator, but because not admin => cannot add curators
    info.sender = addr.b();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_add_curators.clone(),
    );
    assert!(extract_error_msg(&result).contains("This is an admin function"));

    // admin can remove curator addr.b with just one operation, even though addr.b was added as curator multiple times
    // admin can also remove itself as curator
    // i) add addr.b (and addr.c) as curator a few more times
    info.sender = addr.a();
    for _ in 0..2 {
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            msg_add_curators.clone(),
        )?;
    }
    let q_answer = from_binary::<QueryAnswer>(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ContractInfo {},
    )?)?;
    match q_answer {
        QueryAnswer::ContractInfo { curators, .. } => {
            assert_eq!(
                curators,
                vec![
                    addr.a(),
                    addr.b(),
                    addr.c(),
                    addr.b(),
                    addr.c(),
                    addr.b(),
                    addr.c()
                ]
            )
        }
        _ => panic!("query error"),
    }

    // ii) remove addr.a and addr.b as curators
    let msg_remove_curators = ExecuteMsg::RemoveCurators {
        remove_curators: vec![addr.a(), addr.b()],
        padding: None,
    };
    info.sender = addr.a();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_remove_curators)?;
    let q_answer = from_binary::<QueryAnswer>(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ContractInfo {},
    )?)?;
    match q_answer {
        QueryAnswer::ContractInfo { curators, .. } => {
            assert_eq!(curators, vec![addr.c(), addr.c(), addr.c()])
        }
        _ => panic!("query error"),
    }

    // now curator addr.b cannot curate new tokens anymore
    let mut curate1 = default_curate_value();
    curate1.token_info.token_id = "test1".to_string();
    let msg_curate_1 = ExecuteMsg::CurateTokenIds {
        initial_tokens: vec![curate1],
        memo: None,
        padding: None,
    };
    info.sender = addr.b();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_curate_1.clone(),
    );
    assert!(extract_error_msg(&result).contains("Only curators are allowed to curate token_ids"));
    assert_eq!(chk_bal(&deps.storage, "test1", &addr.a()), None);

    // addr.a (which is admin) cannot curate new tokens either, since it is no longer a curator
    info.sender = addr.a();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_curate_1.clone(),
    );
    assert!(extract_error_msg(&result).contains("Only curators are allowed to curate token_ids"));
    assert_eq!(chk_bal(&deps.storage, "test1", &addr.a()), None);

    // addr.c (still a curator), can still curate new tokens
    info.sender = addr.c();
    execute(deps.as_mut(), mock_env(), info, msg_curate_1)?;
    assert_eq!(
        chk_bal(&deps.storage, "test1", &addr.a()),
        Some(Uint256::from(1000u128))
    );

    Ok(())
}

// #[test]
// fn test_add_remove_minters() -> StdResult<()> {
//     // init addresses
//     let addr = init_addrs();

//     // instantiate
//     let (_init_result, mut deps) = init_helper_default();

//     // admin adds 2 curators, addr.b and addr.c ...
//     let mut info = mock_info(addr.a().as_str(), &[]);
//     let msg_add_curators = ExecuteMsg::AddCurators {
//         add_curators: vec![addr.b(), addr.c()],
//         padding: None,
//     };
//     execute(deps.as_mut(), mock_env(), info.clone(), msg_add_curators)?;

//     // ...then new curator (addr.b) curates new token_id
//     let mut curate0 = default_curate_value();
//     curate0.token_info.token_id = "test0".to_string();
//     let msg_curate = ExecuteMsg::CurateTokenIds {
//         initial_tokens: vec![curate0],
//         memo: None,
//         padding: None,
//     };
//     info.sender = addr.b();
//     execute(deps.as_mut(), mock_env(), info.clone(), msg_curate)?;
//     assert_eq!(
//         chk_bal(&deps.storage, "test0", &addr.a()),
//         Some(Uint256::from(1000u128))
//     );

//     // addr.b cannot mint new tokens because it is not a minter despite creating the token_id
//     let msg_mint = ExecuteMsg::MintTokens {
//         mint_tokens: vec![TokenAmount {
//             token_id: "test0".to_string(),
//             balances: vec![TokenIdBalance {
//                 address: addr.a(),
//                 amount: Uint256::from(100u128),
//             }],
//         }],
//         memo: None,
//         padding: None,
//     };
//     let mut result = execute(deps.as_mut(), mock_env(), info.clone(), msg_mint.clone());
//     assert!(extract_error_msg(&result)
//         .contains("Only minters are allowed to mint additional tokens for token_id test0"));

//     // addr.c, is curator, but not token_id "test0"'s curator, so cannot add minters (in base spec, addr.c cannot add/remove minter in any event)
//     let msg_add_minter_c = ExecuteMsg::AddMinters {
//         token_id: "test0".to_string(),
//         add_minters: vec![addr.c()],
//         padding: None,
//     };
//     info.sender = addr.c();
//     result = execute(
//         deps.as_mut(),
//         mock_env(),
//         info.clone(),
//         msg_add_minter_c.clone(),
//     );
//     assert!(
//         extract_error_msg(&result).contains("You need to be the admin to add or remove minters")
//     );

//     // addr.b, as token_id's curator, but still cannot add minter addr.c (in additional specs, may be possible)
//     info.sender = addr.b();
//     result = execute(deps.as_mut(), mock_env(), info.clone(), msg_add_minter_c);
//     assert!(
//         extract_error_msg(&result).contains("You need to be the admin to add or remove minters")
//     );

//     // check minter list is unchanged
//     let q_answer = from_binary::<QueryAnswer>(&query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::TokenIdPublicInfo {
//             token_id: "test0".to_string(),
//         },
//     )?)?;
//     match q_answer {
//         QueryAnswer::TokenIdPublicInfo { token_id_info, .. } => {
//             assert_eq!(token_id_info.curator, addr.b());
//             assert_eq!(token_id_info.token_config.flatten().minters, vec![addr.a()]);
//         }
//         _ => panic!("query error"),
//     }

//     // admin addr.a can add minters addr.c and addr.d twice (in a single tx).
//     // Addr.d is added twice for test later that it can be removed in a single remove_minter msg
//     let msg_add_minter_cd = ExecuteMsg::AddMinters {
//         token_id: "test0".to_string(),
//         add_minters: vec![addr.c(), addr.d(), addr.d()],
//         padding: None,
//     };
//     info.sender = addr.a();
//     execute(deps.as_mut(), mock_env(), info.clone(), msg_add_minter_cd)?;

//     let mut q_answer = from_binary::<QueryAnswer>(&query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::TokenIdPublicInfo {
//             token_id: "test0".to_string(),
//         },
//     )?)?;
//     match q_answer {
//         QueryAnswer::TokenIdPublicInfo { token_id_info, .. } => {
//             assert_eq!(
//                 token_id_info.token_config.flatten().minters,
//                 vec![addr.a(), addr.c(), addr.d(), addr.d()]
//             );
//         }
//         _ => panic!("query error"),
//     }

//     // admin addr.a cannot add minters for a non-existent token_id
//     let msg_add_minter_nonexistent = ExecuteMsg::AddMinters {
//         token_id: "test-na".to_string(),
//         add_minters: vec![addr.d()],
//         padding: None,
//     };
//     info.sender = addr.a();
//     result = execute(
//         deps.as_mut(),
//         mock_env(),
//         info.clone(),
//         msg_add_minter_nonexistent,
//     );
//     assert!(extract_error_msg(&result).contains("token_id test-na does not exist"));

//     // both minters addr.c and addr.d can mint new tokens
//     info.sender = addr.c();
//     execute(deps.as_mut(), mock_env(), info.clone(), msg_mint.clone())?;
//     info.sender = addr.d();
//     execute(deps.as_mut(), mock_env(), info.clone(), msg_mint.clone())?;
//     assert_eq!(
//         chk_bal(&deps.storage, "test0", &addr.a()),
//         Some(Uint256::from(1200u128))
//     );

//     // minters cannot burn tokens
//     let msg_burn = ExecuteMsg::BurnTokens {
//         burn_tokens: vec![TokenAmount {
//             token_id: "test0".to_string(),
//             balances: vec![TokenIdBalance {
//                 address: addr.a(),
//                 amount: Uint256::from(500u128),
//             }],
//         }],
//         memo: None,
//         padding: None,
//     };
//     info.sender = addr.c();
//     result = execute(deps.as_mut(), mock_env(), info.clone(), msg_burn);
//     assert!(extract_error_msg(&result)
//         .contains("you do not have permission to burn 500 tokens from address addr0",));

//     // minters can change metadata (because of config allows)
//     let msg_change_metadata = ExecuteMsg::ChangeMetadata {
//         token_id: "test0".to_string(),
//         public_metadata: Box::new(Some(Metadata {
//             token_uri: Some("new public uri".to_string()),
//             extension: Some(Extension::default()),
//         })),
//         private_metadata: Box::new(None),
//     };
//     info.sender = addr.c();
//     execute(deps.as_mut(), mock_env(), info.clone(), msg_change_metadata)?;
//     q_answer = from_binary::<QueryAnswer>(&query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::TokenIdPublicInfo {
//             token_id: "test0".to_string(),
//         },
//     )?)?;
//     match q_answer {
//         QueryAnswer::TokenIdPublicInfo { token_id_info, .. } => {
//             assert_eq!(
//                 token_id_info.public_metadata.unwrap().token_uri,
//                 Some("new public uri".to_string())
//             )
//         }
//         _ => panic!("query error"),
//     }

//     // curator addr.c cannot remove minters addr.c (note: addr.c is not the curator that created this token_id, although this is irrelevant in base specs)
//     let msg_remove_minter_c = ExecuteMsg::RemoveMinters {
//         token_id: "test0".to_string(),
//         remove_minters: vec![addr.c()],
//         padding: None,
//     };
//     info.sender = addr.c();
//     result = execute(
//         deps.as_mut(),
//         mock_env(),
//         info.clone(),
//         msg_remove_minter_c.clone(),
//     );
//     assert!(
//         extract_error_msg(&result).contains("You need to be the admin to add or remove minters")
//     );

//     // token_id curator addr.b cannot remove minter addr.c, per the base specs
//     info.sender = addr.b();
//     result = execute(deps.as_mut(), mock_env(), info.clone(), msg_remove_minter_c);
//     assert!(
//         extract_error_msg(&result).contains("You need to be the admin to add or remove minters")
//     );
//     // check minter list is unchanged
//     q_answer = from_binary::<QueryAnswer>(&query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::TokenIdPublicInfo {
//             token_id: "test0".to_string(),
//         },
//     )?)?;
//     match q_answer {
//         QueryAnswer::TokenIdPublicInfo { token_id_info, .. } => {
//             assert_eq!(
//                 token_id_info.token_config.flatten().minters,
//                 vec![addr.a(), addr.c(), addr.d(), addr.d()]
//             );
//         }
//         _ => panic!("query error"),
//     }

//     // admin can remove all minters: minter addr.d; although added twice, just need one input remove
//     // addr.a (as admin) can also remove itself as minter
//     let msg_remove_minter_acd = ExecuteMsg::RemoveMinters {
//         token_id: "test0".to_string(),
//         remove_minters: vec![addr.a(), addr.c(), addr.d()],
//         padding: None,
//     };
//     info.sender = addr.a();
//     execute(
//         deps.as_mut(),
//         mock_env(),
//         info.clone(),
//         msg_remove_minter_acd,
//     )?;
//     q_answer = from_binary::<QueryAnswer>(&query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::TokenIdPublicInfo {
//             token_id: "test0".to_string(),
//         },
//     )?)?;
//     match q_answer {
//         QueryAnswer::TokenIdPublicInfo { token_id_info, .. } => {
//             assert_eq!(
//                 token_id_info.token_config.flatten().minters,
//                 Vec::<Addr>::new()
//             );
//         }
//         _ => panic!("query error"),
//     }

//     // check no one can mint tokens now
//     // admin (addr.a) not a minter anymore
//     // addr.b (curator of the token_id) was never a minter
//     // (addr.c and addr.d) no longer minters
//     for address in addr.all() {
//         info.sender = address;
//         result = execute(deps.as_mut(), mock_env(), info.clone(), msg_mint.clone());
//         assert!(extract_error_msg(&result)
//             .contains("Only minters are allowed to mint additional tokens for token_id test0"));
//     }

//     Ok(())
// }

#[test]
fn test_change_admin() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // check current admin
    let contract_info = contr_conf_r(&deps.storage).load()?;
    assert_eq!(contract_info.admin, Some(addr.a()));

    // error: non-admin cannot call this function
    let msg_change_admin = ExecuteMsg::ChangeAdmin {
        new_admin: addr.b(),
        padding: None,
    };
    let mut info = mock_info(addr.b().as_str(), &[]);
    let result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_change_admin.clone(),
    );
    assert!(extract_error_msg(&result).contains("This is an admin function"));

    // success: admin can change admin
    info.sender = addr.a();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_change_admin)?;
    let contract_info = contr_conf_r(&deps.storage).load()?;
    assert_eq!(contract_info.admin, Some(addr.b()));

    // old admin cannot call admin function (choice of function is arbitrary)
    let msg_add_curators = ExecuteMsg::AddCurators {
        add_curators: vec![addr.b()],
        padding: None,
    };
    let result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_add_curators.clone(),
    );
    assert!(extract_error_msg(&result).contains("This is an admin function"));

    // success: new admin can call admin function
    info.sender = addr.b();
    execute(deps.as_mut(), mock_env(), info, msg_add_curators)?;

    Ok(())
}

#[test]
fn test_remove_admin() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // check admin from contract_info
    let q_answer = from_binary::<QueryAnswer>(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ContractInfo {},
    )?)?;
    match q_answer {
        QueryAnswer::ContractInfo {
            admin,
            curators,
            all_token_ids,
        } => {
            assert_eq!(admin, Some(addr.a()));
            assert_eq!(curators, vec![addr.a()]);
            assert_eq!(all_token_ids, vec!["0".to_string()]);
        }
        _ => panic!("query error"),
    }

    // test admin can perform an admin function (choice of function is arbitrary)
    let msg_add_curators = ExecuteMsg::AddCurators {
        add_curators: vec![addr.b()],
        padding: None,
    };
    let mut info = mock_info(addr.a().as_str(), &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_add_curators.clone(),
    )?;

    // admin tries to remove admin: fail due to wrong current admin input
    info.sender = addr.a();
    let mut result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RemoveAdmin {
            current_admin: addr.b(),
            contract_address: Addr::unchecked("cosmos2contract".to_string()),
            padding: None,
        },
    );
    assert!(
        extract_error_msg(&result).contains("your inputs are incorrect to perform this function")
    );

    // error: admin tries to remove admin: fail due to wrong contract address
    info.sender = addr.a();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RemoveAdmin {
            current_admin: addr.a(),
            contract_address: Addr::unchecked("wronginput".to_string()),
            padding: None,
        },
    );
    assert!(
        extract_error_msg(&result).contains("your inputs are incorrect to perform this function")
    );

    // error: non-admin cannot remove admin
    let msg_remove_admin = ExecuteMsg::RemoveAdmin {
        current_admin: addr.a(),
        contract_address: Addr::unchecked("cosmos2contract".to_string()),
        padding: None,
    };
    info.sender = addr.b();
    result = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        msg_remove_admin.clone(),
    );
    assert!(extract_error_msg(&result).contains("This is an admin function"));

    // success: admin removes admin
    info.sender = addr.a();
    execute(deps.as_mut(), mock_env(), info.clone(), msg_remove_admin)?;

    // check that admin can no longer perform admin function
    result = execute(deps.as_mut(), mock_env(), info, msg_add_curators);
    assert!(extract_error_msg(&result).contains("This contract has no admin"));

    // check that contract_info shows no admin
    let q_answer = from_binary::<QueryAnswer>(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::ContractInfo {},
    )?)?;
    match q_answer {
        QueryAnswer::ContractInfo {
            admin,
            curators,
            all_token_ids,
        } => {
            assert_eq!(admin, None);
            assert_eq!(curators, vec![addr.a(), addr.b()]);
            assert_eq!(all_token_ids, vec!["0".to_string()]);
        }
        _ => panic!("query error"),
    }

    Ok(())
}

#[test]
fn test_instantiate_admin_inputs() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // case0: instantiate with has_admin = false && admin = None -> no admin
    let mut deps = mock_dependencies();
    let mut info = mock_info(addr.a().as_str(), &[]);

    let init_msg = InstantiateMsg {
        has_admin: false,
        admin: None,
        curators: vec![],
        entropy: "seedentropy".to_string(),
        initial_tokens: vec![default_curate_value()],

        lb_pair_info: LbPair {
            name: String::new(),
            symbol: String::new(),
            lb_pair_address: Addr::unchecked("address"),
            decimals: 18,
        },
    };

    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)?;
    assert_eq!(contr_conf_r(&deps.storage).load()?.admin, None);

    // case1: instantiate with has_admin = false && admin = Some(_) -> no admin
    let mut deps = mock_dependencies();

    let init_msg = InstantiateMsg {
        has_admin: false,
        admin: Some(addr.a()),
        curators: vec![],
        entropy: "seedentropy".to_string(),
        initial_tokens: vec![default_curate_value()],

        lb_pair_info: LbPair {
            name: String::new(),
            symbol: String::new(),
            lb_pair_address: Addr::unchecked("address"),
            decimals: 18,
        },
    };

    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)?;
    assert_eq!(contr_conf_r(&deps.storage).load()?.admin, None);

    // case2: instantiate with has_admin = true && admin = None -> defaults to sender as admin
    let mut deps = mock_dependencies();

    let init_msg = InstantiateMsg {
        has_admin: true,
        admin: None,
        curators: vec![],
        entropy: "seedentropy".to_string(),
        initial_tokens: vec![default_curate_value()],

        lb_pair_info: LbPair {
            name: String::new(),
            symbol: String::new(),
            lb_pair_address: Addr::unchecked("address"),
            decimals: 18,
        },
    };

    info.sender = addr.a();
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)?;
    assert_eq!(contr_conf_r(&deps.storage).load()?.admin, Some(addr.a()));

    // case3: instantiate with has_admin = true && admin = addr.b() -> admin is addr.b(), although addr.a() instantiated
    let mut deps = mock_dependencies();

    let init_msg = InstantiateMsg {
        has_admin: true,
        admin: Some(addr.b()),
        curators: vec![],
        entropy: "seedentropy".to_string(),
        initial_tokens: vec![default_curate_value()],
        lb_pair_info: LbPair {
            name: String::new(),
            symbol: String::new(),
            lb_pair_address: Addr::unchecked("address"),
            decimals: 18,
        },
    };

    info.sender = addr.a();
    instantiate(deps.as_mut(), mock_env(), info, init_msg)?;
    assert_eq!(contr_conf_r(&deps.storage).load()?.admin, Some(addr.b()));

    Ok(())
}

#[test]
fn test_receiver_sanity() -> StdResult<()> {
    // init addresses
    let addr = init_addrs();

    // instantiate
    let (_init_result, mut deps) = init_helper_default();

    // `send` with msg
    let info = mock_info(addr.a().as_str(), &[]);
    let msg = ExecuteMsg::Send {
        token_id: "0".to_string(),
        from: addr.a(),
        recipient: addr.b(),
        recipient_code_hash: Some(addr.b_hash()),
        amount: Uint256::from(800u128),
        msg: Some(to_binary(&"msg_str")?),
        memo: Some("some memo".to_string()),
        padding: None,
    };
    let response = execute(deps.as_mut(), mock_env(), info, msg)?;
    let (receiver_msg, receiver_addr, receiver_hash) =
        extract_cosmos_msg::<ReceiverHandleMsg>(&response.messages[0].msg)?;
    assert_eq!(receiver_addr, Some(addr.b()));
    assert_eq!(receiver_hash, &addr.b_hash());
    let exp_receive_msg = Snip1155ReceiveMsg {
        sender: addr.a(),
        token_id: "0".to_string(),
        from: addr.a(),
        amount: Uint256::from(800u128),
        memo: Some("some memo".to_string()),
        msg: Some(to_binary(&"msg_str")?),
    };
    match receiver_msg {
        ReceiverHandleMsg::Snip1155Receive(i) => assert_eq!(i, exp_receive_msg),
    }

    Ok(())
}
