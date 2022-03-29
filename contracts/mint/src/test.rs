#[cfg(test)]
pub mod tests {
    use cosmwasm_std::{
        coins, from_binary,
        testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
        Extern, StdError, Uint128, HumanAddr,
    };
    use mockall_double::double;
    use shade_protocol::{
        mint::{HandleMsg, InitMsg, QueryAnswer, QueryMsg},
        utils::price::{normalize_price, translate_price},
    };

    use crate::{
        contract::{handle, init, query},
        handle::{calculate_portion, calculate_mint, try_burn},
    };

    mod mock_secret_toolkit {

        use cosmwasm_std::{HumanAddr, Querier, StdResult, Uint128};
        use secret_toolkit::snip20::TokenInfo;

        pub fn mock_token_info_query<Q: Querier>(
            _querier: &Q,
            _block_size: usize,
            _callback_code_hash: String,
            _contract_addr: HumanAddr,
        ) -> StdResult<TokenInfo> {
            Ok(TokenInfo {
                name: "Token".to_string(),
                symbol: "TKN".to_string(),
                decimals: 6,
                total_supply: Option::from(Uint128(150)),
            })
        }
    }

    #[double]
    use mock_secret_toolkit::token_info_query;
    use shade_protocol::utils::asset::Contract;

    fn create_contract(address: &str, code_hash: &str) -> Contract {
        let env = mock_env(address.to_string(), &[]);
        return Contract {
            address: env.message.sender,
            code_hash: code_hash.to_string(),
        };
    }

    fn dummy_init(
        admin: String,
        native_asset: Contract,
        oracle: Contract,
        peg: Option<String>,
        treasury: HumanAddr,
        capture: Option<Uint128>,
    ) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
            native_asset,
            oracle,
            peg,
            treasury,
            secondary_burn: None,
            limit: None,
        };
        let env = mock_env(admin, &coins(1000, "earth"));
        let _res = init(&mut deps, env, msg).unwrap();

        return deps;
    }

    #[test]
    /*
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
            native_asset: create_contract("", ""),
            oracle: create_contract("", ""),
            peg: Option::from("TKN".to_string()),
            treasury: Option::from(create_contract("", "")),
            // 1%
            capture: Option::from(Uint128(100)),
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
    */

    /*
    #[test]
    fn config_update() {
        let native_asset = create_contract("snip20", "hash");
        let oracle = create_contract("oracle", "hash");
        let treasury = create_contract("treasury", "hash");
        let capture = Uint128(100);

        let admin_env = mock_env("admin", &coins(1000, "earth"));
        let mut deps = dummy_init("admin".to_string(),
                                  native_asset,
                                  oracle,
                                  None,
                                  Option::from(treasury),
                                  Option::from(capture));

        // new config vars
        let new_oracle = Option::from(create_contract("new_oracle", "hash"));
        let new_treasury = Option::from(create_contract("new_treasury", "hash"));
        let new_capture = Option::from(Uint128(200));

        // Update config
        let update_msg = HandleMsg::UpdateConfig {
            owner: None,
            oracle: new_oracle.clone(),
            treasury: new_treasury.clone(),
            // 2%
            capture: new_capture.clone(),
        };
        let update_res = handle(&mut deps, admin_env, update_msg);

        let config_res = query(&deps, QueryMsg::GetConfig {}).unwrap();
        let value: QueryAnswer = from_binary(&config_res).unwrap();
        match value {
            QueryAnswer::Config { config } => {
                assert_eq!(config.oracle, new_oracle.unwrap());
                assert_eq!(config.treasury, new_treasury);
                assert_eq!(config.capture, new_capture);
            }
            _ => { panic!("Received wrong answer") }
        }
    }
    */

    /*
    #[test]
    fn user_register_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""),
                                  create_contract("", ""),
                                  None, None, None);

        // User should not be allowed to add an item
        let user_env = mock_env("user", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
        };
        let res = handle(&mut deps, user_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Response should be an empty array
        let res = query(&deps, QueryMsg::GetSupportedAssets {}).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        match value {
            QueryAnswer::SupportedAssets { assets } => { assert_eq!(0, assets.len()) }
            _ => { panic!("Received wrong answer") }
        }
    }

    #[test]
    fn admin_register_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""),
                                  create_contract("", ""),
                                  None,
                                  None,
                                  None);

        // Admin should be allowed to add an item
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Response should be an array of size 1
        let res = query(&deps, QueryMsg::GetSupportedAssets {}).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        match value {
            QueryAnswer::SupportedAssets { assets } => { assert_eq!(1, assets.len()) }
            _ => { panic!("Received wrong answer") }
        }
    }

    #[test]
    fn duplicate_register_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""),
                                  create_contract("", ""),
                                  None,
                                  None,
                                  None);

        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Should not be allowed to add an existing asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "other_hash");
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::GenericErr { .. }) => {}
            _ => panic!("Must return not found error"),
        };
    }

    /*
    #[test]
    fn user_update_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""),
                                  create_contract("", ""));

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // users should not be allowed to update assets
        let user_env = mock_env("user", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let new_dummy_contract = create_contract("some_other_contract", "some_hash");
        let msg = HandleMsg::UpdateAsset {
            asset: dummy_contract.address,
            contract: new_dummy_contract,
        };
        let res = handle(&mut deps, user_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        };
    }
    */

    /*
    #[test]
    fn admin_update_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""),
                                  create_contract("", ""));

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // admins can update assets
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let new_dummy_contract = create_contract("some_other_contract", "some_hash");
        let msg = HandleMsg::UpdateAsset {
            asset: dummy_contract.address,
            contract: new_dummy_contract,
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Response should be new dummy contract
        let res = query(&deps, QueryMsg::GetAsset { contract: "some_other_contract".to_string() }).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        match value {
            QueryAnswer::Asset { asset, burned } => { assert_eq!("some_other_contract".to_string(), asset.contract.address.to_string()) }
            _ => { panic!("Received wrong answer") }
        };
    }
    */

    #[test]
    fn receiving_an_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""),
                                  create_contract("", ""),
                                  None, None, None);

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Contract tries to send funds
        let env = mock_env("some_contract", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_owner", "some_hash");

        let msg = HandleMsg::Receive {
            sender: dummy_contract.address,
            from: Default::default(),
            amount: Uint128(100),
            msg: None,
            memo: None
        };

        let res = handle(&mut deps, env, msg);
        match res {
            Err(err) => {
                match err {
                    StdError::NotFound { .. } => {panic!("Not found");}
                    StdError::Unauthorized { .. } => {panic!("Unauthorized");}
                    _ => {}
                }
            }
            _ => {}
        }
    }

    #[test]
    fn receiving_an_asset_from_non_supported_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""),
                                  create_contract("", ""),
                                  None,
                                  None,
                                  None,
                                  );

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Contract tries to send funds
        let env = mock_env("some_other_contract", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_owner", "some_hash");
        let msg = HandleMsg::Receive {
            sender: dummy_contract.address,
            from: Default::default(),
            amount: Uint128(100),
            msg: None,
            memo: None
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::NotFound { .. }) => {}
            _ => {panic!("Must return not found error")},
        }
    }
    */

    #[test]
    fn capture_calc() {
        let amount = Uint128(1_000_000_000_000_000_000);
        //10%
        let capture = Uint128(100_000_000_000_000_000);
        let expected = Uint128(100_000_000_000_000_000);
        let value = calculate_portion(amount, capture);
        assert_eq!(value, expected);
    }


    /**
    #[test]
    fn mint_algorithm_simple() {
        // In this example the "sent" value is 1 with 6 decimal places
        // The mint value will be 1 with 3 decimal places
        let price = Uint128(1_000_000_000_000_000_000);
        let in_amount = Uint128(1_000_000);
        let expected_value = Uint128(1_000);
        let value = calculate_mint(price, in_amount, 6, price, 3);

        assert_eq!(value, expected_value);
    }

    #[test]
    fn mint_algorithm_complex_1() {
        // In this example the "sent" value is 1.8 with 6 decimal places
        // The mint value will be 3.6 with 12 decimal places
        let in_price = Uint128(2_000_000_000_000_000_000);
        let target_price = Uint128(1_000_000_000_000_000_000);
        let in_amount = Uint128(1_800_000);
        let expected_value = Uint128(3_600_000_000_000);
        let value = calculate_mint(in_price, in_amount, 6, target_price, 12);

        assert_eq!(value, expected_value);
    }

    #[test]
    fn mint_algorithm_complex_2() {
        // In amount is 50.000 valued at 20
        // target price is 100$ with 6 decimals
        let in_price = Uint128(20_000_000_000_000_000_000);
        let target_price = Uint128(100_000_000_000_000_000_000);
        let in_amount = Uint128(50_000);
        let expected_value = Uint128(10_000_000);
        let value = calculate_mint(in_price, in_amount, 3, target_price, 6);

        assert_eq!(value, expected_value);
    }
    **/
    macro_rules! mint_algorithm_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (in_price, in_amount, in_decimals, target_price, target_decimals, expected_value) = $value;
                    assert_eq!(calculate_mint(in_price, in_amount, in_decimals, target_price, target_decimals), expected_value);
                }
            )*
        }
    }

    mint_algorithm_tests!{
        mint_simple_0: (
            // In this example the "sent" value is 1 with 6 decimal places
            // The mint value will be 1 with 3 decimal places
            Uint128(1_000_000_000_000_000_000), //Burn price
            Uint128(1_000_000),                 //Burn amount
            6u8,                                //Burn decimals
            Uint128(1_000_000_000_000_000_000), //Mint price
            3u8,                                //Mint decimals
            Uint128(1_000),                     //Expected value
        ),
        mint_complex_0: (
            // In this example the "sent" value is 1.8 with 6 decimal places
            // The mint value will be 3.6 with 12 decimal places
            Uint128(2_000_000_000_000_000_000),
            Uint128(1_800_000),
            6u8,
            Uint128(1_000_000_000_000_000_000),
            12u8,
            Uint128(3_600_000_000_000),
        ),
        mint_complex_1: (
            // In amount is 50.000 valued at 20
            // target price is 100$ with 6 decimals
            Uint128(20_000_000_000_000_000_000),
            Uint128(50_000),
            3u8,
            Uint128(100_000_000_000_000_000_000),
            6u8,
            Uint128(10_000_000),
        ),
        mint_complex_2: (
            // In amount is 10,000,000 valued at 100
            // Target price is $10 with 6 decimals
            Uint128(1_000_000_000_000_000_000_000),
            Uint128(100_000_000_000_000),
            8u8,
            Uint128(10_000_000_000_000_000_000),
            6u8,
            Uint128(100_000_000_000_000),
        ),
    }
        /*
        mint_overflow_0: (
            // In amount is 1,000,000,000,000,000,000,000,000 valued at 1,000
            // Target price is $5 with 6 decimals
            Uint128(1_000_000_000_000_000_000_000),
            Uint128(100_000_000_000_000_000_000_000_000_000_000),
            8u8,
            Uint128(5_000_000_000_000_000_000),
            6u8,
            Uint128(500_000_000_000_000_000_000_000_000_000_000_000),
        ),
        */
}
