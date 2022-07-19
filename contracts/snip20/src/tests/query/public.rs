use shade_protocol::contract_interfaces::snip20::{InitConfig, QueryAnswer, QueryMsg};
use shade_protocol::utils::{ExecuteCallback, InstantiateCallback, Query, MultiTestable};
use crate::tests::init_snip20_with_config;

#[test]
fn token_info() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();
    let answer: QueryAnswer = QueryMsg::TokenInfo {  }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::TokenInfo { name, symbol, decimals, total_supply} => {
            assert_eq!(name, "Token");
            assert_eq!(symbol, "TKN");
            assert_eq!(decimals, 8);
            assert_eq!(total_supply, None);
        },
        _ => assert!(false)
    }
}

#[test]
fn token_config() {
    let (mut chain, snip) = init_snip20_with_config(None, None).unwrap();
    let answer: QueryAnswer = QueryMsg::TokenConfig {  }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::TokenConfig {
            public_total_supply,
            deposit_enabled,
            redeem_enabled,
            mint_enabled,
            burn_enabled,
            transfer_enabled
        } => {
            assert_eq!(public_total_supply, false);
            assert_eq!(deposit_enabled, false);
            assert_eq!(redeem_enabled, false);
            assert_eq!(mint_enabled, false);
            assert_eq!(burn_enabled, false);
        },
        _ => assert!(false)
    }

    let (mut chain, snip) = init_snip20_with_config(None, Some(InitConfig{
        public_total_supply: Some(true),
        enable_deposit: Some(true),
        enable_redeem: Some(true),
        enable_mint: None,
        enable_burn: None,
        enable_transfer: None
    })).unwrap();
    let answer: QueryAnswer = QueryMsg::TokenConfig {  }.test_query(&snip, &chain).unwrap();

    match answer {
        QueryAnswer::TokenConfig {
            public_total_supply,
            deposit_enabled,
            redeem_enabled,
            mint_enabled,
            burn_enabled,
            transfer_enabled
        } => {
            assert_eq!(public_total_supply, true);
            assert_eq!(deposit_enabled, true);
            assert_eq!(redeem_enabled, true);
            assert_eq!(mint_enabled, false);
            assert_eq!(burn_enabled, false);
        },
        _ => assert!(false)
    }
}

// TODO: add exchange rate after IBC is added