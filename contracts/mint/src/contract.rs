use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128};

use crate::msg::{HandleMsg, HandleAnswer, InitMsg, QueryMsg, QueryAnswer, OracleCall, ResponseStatus};
use crate::state::{config, config_read, assets_w, assets_r, asset_list, asset_list_read, Config, Asset};
use secret_toolkit::snip20::{mint_msg, register_receive_msg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        owner: env.message.sender.clone(),
        silk_contract: msg.silk_contract,
        silk_contract_code_hash: msg.silk_contract_code_hash,
        oracle_contract: msg.oracle_contract,
        oracle_contract_code_hash: msg.oracle_contract_code_hash,
    };

    config(&mut deps.storage).save(&state)?;

    let empty_assets_list: Vec<String> = Vec::new();
    asset_list(&mut deps.storage).save(&empty_assets_list)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig {
            owner,
            silk_contract,
            silk_contract_code_hash,
            oracle_contract,
            oracle_contract_code_hash
        } => try_update_config(deps, env, owner, silk_contract, silk_contract_code_hash,
                                oracle_contract, oracle_contract_code_hash),
        HandleMsg::RegisterAsset {
            contract,
            code_hash
        } => try_register_asset(deps, env, contract, code_hash),
        HandleMsg::UpdateAsset {
            asset,
            contract,
            code_hash
        } => try_update_asset(deps, env, asset, contract, code_hash),
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..} => try_burn(deps, env, sender, from, amount, msg),
    }
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    silk_contract: Option<HumanAddr>,
    silk_contract_code_hash: Option<String>,
    oracle_contract: Option<HumanAddr>,
    oracle_contract_code_hash: Option<String>,
) -> StdResult<HandleResponse> {
    let mut config = config(&mut deps.storage);

    // Check if admin
    if env.message.sender != config.load()?.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    config.update(|mut state| {
        if let Some(owner) = owner {
            state.owner = owner;
        }
        if let Some(silk_contract) = silk_contract {
            state.silk_contract = silk_contract;
        }
        if let Some(silk_contract_code_hash) = silk_contract_code_hash {
            state.oracle_contract_code_hash = silk_contract_code_hash;
        }
        if let Some(oracle_contract) = oracle_contract {
            state.oracle_contract = oracle_contract;
        }
        if let Some(oracle_contract_code_hash) = oracle_contract_code_hash {
            state.oracle_contract_code_hash = oracle_contract_code_hash;
        }
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateAsset {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    contract: HumanAddr,
    code_hash: String,
) -> StdResult<HandleResponse> {
    let config = config_read(&deps.storage);

    let contract_str = contract.to_string();

    // Check if admin
    if env.message.sender != config.load()?.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut assets = assets_w(&mut deps.storage);

    let mut messages = vec![];

    // Check if asset already exists
    match assets.may_load(contract_str.as_bytes())? {
        Some(_) => return Err(StdError::generic_err("Asset already exists")),

        None => {
            // Add the new asset
            assets.save(contract_str.as_bytes(), &Asset {
                contract: contract.clone(),
                code_hash: code_hash.clone(),
                burned_tokens: Uint128(0),
            })?;
            // Add asset to list
            asset_list(&mut deps.storage).update(|mut state| {
                state.push(contract_str);
                Ok(state)
            })?;
            // Register contract in asset
            let register_msg = register_receive(&deps, env, contract, code_hash)?;
            messages.push(register_msg);
        }
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_update_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: HumanAddr,
    contract: HumanAddr,
    code_hash: String,
) -> StdResult<HandleResponse> {
    let config = config_read(&deps.storage);

    let asset_str = asset.to_string();

    // Check if admin
    if env.message.sender != config.load()?.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut assets = assets_w(&mut deps.storage);

    let mut messages = vec![];

    // Check if asset already exists
    match assets.may_load(asset_str.as_bytes())? {
        Some(loaded_asset) => {
            // Remove the old asset
            assets.remove(asset_str.as_bytes());
            // Add the new asset
            assets.save(contract.to_string().as_bytes(), &Asset {
                contract: contract.clone(),
                code_hash: code_hash.clone(),
                burned_tokens: loaded_asset.burned_tokens
            })?;
            // Remove old asset from list
            asset_list(&mut deps.storage).update(|mut state| {
                for (i, asset) in state.iter().enumerate() {
                    if asset == &asset_str {
                        state.remove(i);
                        state.push(asset_str.clone());
                        break;
                    }
                }
                Ok(state)
            })?;
            // Register contract in asset
            let register_msg = register_receive(&deps, env, contract, code_hash)?;
            messages.push(register_msg)
        },

        None => return Err(StdError::NotFound { kind: asset_str, backtrace: None }),
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateAsset {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_burn<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    _msg: Option<CosmosMsg>
) -> StdResult<HandleResponse> {
    // Check that the asset is supported
    let mut assets = assets_w(&mut deps.storage);

    // Check if asset already exists
    match assets.may_load(env.message.sender.to_string().as_bytes())? {
        Some(_) => {
            assets.update(env.message.sender.to_string().as_bytes(), |item| {
                let mut asset: Asset = item.unwrap();

                asset.burned_tokens += amount;
                Ok(asset)
            })?;
        },

        None => return Err(StdError::NotFound { kind: env.message.sender.to_string(), backtrace: None }),
    }

    // First get the current value per coin
    let token_value:u128 = call_oracle(deps, env.clone(), env.message.sender.clone())?.into();
    let amount_converted:u128 = amount.into();

    // // Calculate amount to mint
    let value_to_mint = Uint128::from(amount_converted * token_value);

    let mut messages = vec![];

    let mint_msg = mint_silk(deps, from, value_to_mint)?;
    messages.push(mint_msg);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Burn {
            status: ResponseStatus::Success,
            mint_amount: value_to_mint
        } )? ),
    })
}

// Helper functions

fn register_receive<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    env: Env,
    contract: HumanAddr,
    code_hash: String,
) -> StdResult<CosmosMsg> {
    let cosmos_msg = register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        code_hash,
        contract,
    );

    cosmos_msg
}

fn mint_silk<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    sender: HumanAddr,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    let config = config_read(&deps.storage).load()?;

    let cosmos_msg = mint_msg(
        sender,
        amount,
        None,
        256,
        config.silk_contract_code_hash,
        config.silk_contract,
    );

    cosmos_msg
}

fn call_oracle<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _contract: HumanAddr,
) -> StdResult<Uint128> {
    // Call contract
    // let block_size = 1; //update this later
    // let config = config_read(&deps.storage).load()?;
    // let mut msg = to_binary(&&OracleCall{ contract })?;
    // space_pad(&mut msg.0, block_size);
    // let _execute = WasmMsg::Execute {
    //     contract_addr: config.oracle_contract,
    //     callback_code_hash: config.oracle_contract_code_hash,
    //     msg,
    //     send: vec![]
    // };
    // somehow handle execute and get a Uint128 value
    let value = Uint128(1);
    Ok(value)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSupportedAssets {} => to_binary(&query_supported_assets(deps)?),
        QueryMsg::GetAsset { contract } => to_binary(&query_asset(deps, contract)?),
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
    }
}

fn query_supported_assets<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::SupportedAssets { assets: asset_list_read(&deps.storage).load()? })
}

fn query_asset<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, contract: String) -> StdResult<QueryAnswer> {
    let assets = assets_r(&deps.storage);

    return match assets.may_load(contract.as_bytes())? {
        Some(asset) => Ok(QueryAnswer::Asset { asset }),
        None => Err(StdError::NotFound { kind: contract, backtrace: None }),
    };
}

fn query_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config { config: config_read(&deps.storage).load()? })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{coins, from_binary, StdError};
    use crate::msg::QueryAnswer;

    fn create_contract(str: String) -> HumanAddr {
        let env = mock_env(str, &[]);
        return env.message.sender
    }

    fn dummy_init(admin: String, silk: HumanAddr, silk_hash: String, oracle: HumanAddr,
                  oracle_hash: String) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            silk_contract: silk,
            silk_contract_code_hash: silk_hash,
            oracle_contract: oracle,
            oracle_contract_code_hash: oracle_hash };
        let env = mock_env(admin, &coins(1000, "earth"));
        let _res = init(&mut deps, env, msg).unwrap();

        return deps
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            silk_contract: Default::default(),
            silk_contract_code_hash: "".to_string(),
            oracle_contract: Default::default(),
            oracle_contract_code_hash: "".to_string() };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn config_update() {
        let silk_contract = create_contract("silk_contract".to_string());
        let oracle_contract = create_contract("oracle_contract".to_string());
        let mut deps = dummy_init("admin".to_string(), silk_contract,
                                  "silk_hash".to_string(), oracle_contract,
                                  "oracle_hash".to_string());

        // Check config is properly updated
        let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        let silk_contract = create_contract("silk_contract".to_string());
        let oracle_contract = create_contract("oracle_contract".to_string());
        match value {
            QueryAnswer::Config { config } => {
                assert_eq!(config.silk_contract, silk_contract);
                assert_eq!(config.silk_contract_code_hash, "silk_hash".to_string());
                assert_eq!(config.oracle_contract, oracle_contract);
                assert_eq!(config.oracle_contract_code_hash, "oracle_hash".to_string());

            }
            _ => { panic!("Received wrong answer") }
        }

        // Update config
        let user_env = mock_env("admin", &coins(1000, "earth"));
        let new_silk_contract = create_contract("new_silk_contract".to_string());
        let new_oracle_contract = create_contract("new_oracle_contract".to_string());
        let msg = HandleMsg::UpdateConfig {
            owner: None,
            silk_contract: Option::from(new_silk_contract),
            silk_contract_code_hash: None,
            oracle_contract: Option::from(new_oracle_contract),
            oracle_contract_code_hash: None
        };
        let res = handle(&mut deps, user_env, msg);

        // Check config is properly updated
        let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        let new_silk_contract = create_contract("new_silk_contract".to_string());
        let new_oracle_contract = create_contract("new_oracle_contract".to_string());
        match value {
            QueryAnswer::Config { config } => {
                assert_eq!(config.silk_contract, new_silk_contract);
                assert_eq!(config.silk_contract_code_hash, "silk_hash".to_string());
                assert_eq!(config.oracle_contract, new_oracle_contract);
                assert_eq!(config.oracle_contract_code_hash, "oracle_hash".to_string());

            }
            _ => { panic!("Received wrong answer") }
        }

    }

    #[test]
    fn user_register_asset() {
        let mut deps = dummy_init("admin".to_string(), Default::default(),
                                  "silk_hash".to_string(), Default::default(),
                                  "oracle_hash".to_string());

        // User should not be allowed to add an item
        let user_env = mock_env("user", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "some_hash".to_string()
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
        let mut deps = dummy_init("admin".to_string(), Default::default(),
                                  "silk_hash".to_string(), Default::default(),
                                  "oracle_hash".to_string());

        // Admin should be allowed to add an item
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "some_hash".to_string()
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
        let mut deps = dummy_init("admin".to_string(), Default::default(),
                                  "silk_hash".to_string(), Default::default(),
                                  "oracle_hash".to_string());

        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Should not be allowed to add an existing asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "other_hash".to_string()
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::GenericErr { .. }) => {}
            _ => panic!("Must return not found error"),
        };
    }

    #[test]
    fn user_update_asset() {
        let mut deps = dummy_init("admin".to_string(), Default::default(),
                                  "silk_hash".to_string(), Default::default(),
                                  "oracle_hash".to_string());

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // users should not be allowed to update assets
        let user_env = mock_env("user", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let new_dummy_contract = create_contract("some_other_contract".to_string());
        let msg = HandleMsg::UpdateAsset {
            asset: dummy_contract,
            contract: new_dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let res = handle(&mut deps, user_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        };
    }

    #[test]
    fn admin_update_asset() {
        let mut deps = dummy_init("admin".to_string(), Default::default(),
                                  "silk_hash".to_string(), Default::default(),
                                  "oracle_hash".to_string());

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // admins can update assets
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let new_dummy_contract = create_contract("some_other_contract".to_string());
        let msg = HandleMsg::UpdateAsset {
            asset: dummy_contract,
            contract: new_dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Response should be new dummy contract
        let res = query(&deps, QueryMsg::GetAsset { contract: "some_other_contract".to_string() }).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        match value {
            QueryAnswer::Asset { asset } => { assert_eq!("some_other_contract".to_string(), asset.contract.to_string()) }
            _ => { panic!("Received wrong answer") }
        };
    }

    #[test]
    fn nonexisting_update_asset() {
        let mut deps = dummy_init("admin".to_string(), Default::default(),
                                  "silk_hash".to_string(), Default::default(),
                                  "oracle_hash".to_string());

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Should now be able to update non existing asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let bad_dummy_contract = create_contract("some_non_existing_contract".to_string());
        let new_dummy_contract = create_contract("some_other_contract".to_string());
        let msg = HandleMsg::UpdateAsset {
            asset: bad_dummy_contract,
            contract: new_dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::NotFound { .. }) => {}
            _ => panic!("Must return not found error"),
        }
    }

    #[test]
    fn receiving_an_asset() {
        let mut deps = dummy_init("admin".to_string(), Default::default(),
                                  "silk_hash".to_string(), Default::default(),
                                  "oracle_hash".to_string());

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Contract tries to send funds
        let env = mock_env("some_contract", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_owner".to_string());
        let msg = HandleMsg::Receive {
            sender: dummy_contract,
            from: Default::default(),
            amount: Uint128(100),
            msg: None,
            memo: None
        };
        let res = handle(&mut deps, env, msg);
        if res.is_err() {
            panic!("Must not return error");
        }
    }

    #[test]
    fn receiving_an_asset_from_non_supported_asset() {
        let mut deps = dummy_init("admin".to_string(), Default::default(),
                                  "silk_hash".to_string(), Default::default(),
                                  "oracle_hash".to_string());

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract".to_string());
        let msg = HandleMsg::RegisterAsset {
            contract: dummy_contract,
            code_hash: "some_hash".to_string()
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Contract tries to send funds
        let env = mock_env("some_other_contract", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_owner".to_string());
        let msg = HandleMsg::Receive {
            sender: dummy_contract,
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
}