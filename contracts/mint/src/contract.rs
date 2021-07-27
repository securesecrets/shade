use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128};

use crate::msg::{HandleMsg, HandleAnswer, InitMsg, QueryMsg, QueryAnswer, ResponseStatus};
use crate::state::{config, config_read, assets_w, assets_r, asset_list, asset_list_read, Config, Asset, Contract};
use secret_toolkit::snip20::{mint_msg, register_receive_msg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        silk: msg.silk,
        oracle: msg.oracle,
        activated: true,
    };

    config(&mut deps.storage).save(&state)?;

    let empty_assets_list: Vec<String> = Vec::new();
    asset_list(&mut deps.storage).save(&empty_assets_list)?;

    if let Some(assets) = msg.initial_assets {
        for asset in assets {
            let _response = try_register_asset(deps, &env, asset);
        }
    }
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
            silk,
            oracle
        } => try_update_config(deps, env, owner, silk, oracle),
        HandleMsg::RegisterAsset {
            contract,
        } => try_register_asset(deps, &env, contract),
        HandleMsg::UpdateAsset {
            asset,
            contract,
        } => try_update_asset(deps, env, asset, contract),
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
    silk: Option<Contract>,
    oracle: Option<Contract>,
) -> StdResult<HandleResponse> {
    if !authorized(deps, &env, AllowedAccess::Admin)? {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config(&mut deps.storage);
    config.update(|mut state| {
        if let Some(owner) = owner {
            state.owner = owner;
        }
        if let Some(silk) = silk {
            state.silk = silk;
        }
        if let Some(oracle) = oracle {
            state.oracle = oracle;
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
    env: &Env,
    contract: Contract,
) -> StdResult<HandleResponse> {
    if !authorized(deps, &env, AllowedAccess::Admin)? {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let contract_str = contract.address.to_string();
    let mut assets = assets_w(&mut deps.storage);
    let mut messages = vec![];

    // Check if asset already exists
    match assets.may_load(contract_str.as_bytes())? {
        Some(_) => return Err(StdError::generic_err("Asset already exists")),

        None => {
            // Add the new asset
            assets.save(contract_str.as_bytes(), &Asset {
                contract: contract.clone(),
                burned_tokens: Uint128(0),
            })?;
            // Add asset to list
            asset_list(&mut deps.storage).update(|mut state| {
                state.push(contract_str);
                Ok(state)
            })?;
            // Register contract in asset
            let register_msg = register_receive(&deps, env, contract.address, contract.code_hash)?;
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
    contract: Contract,
) -> StdResult<HandleResponse> {
    if !authorized(deps, &env, AllowedAccess::Admin)? {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let asset_str = asset.to_string();
    let mut assets = assets_w(&mut deps.storage);
    let mut messages = vec![];

    // Check if asset already exists
    match assets.may_load(asset_str.as_bytes())? {
        Some(loaded_asset) => {
            // Remove the old asset
            assets.remove(asset_str.as_bytes());
            // Add the new asset
            assets.save(contract.address.to_string().as_bytes(), &Asset {
                contract: contract.clone(),
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
            let register_msg = register_receive(&deps, &env, contract.address, contract.code_hash)?;
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
    if !authorized(deps, &env, AllowedAccess::User)? {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    
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

#[derive(PartialEq)]
pub enum AllowedAccess{
    Admin,
    User,
}

pub fn authorized<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    access: AllowedAccess,
) -> StdResult<bool> {
    let config = config_read(&deps.storage).load()?;
    // Check if contract is still activated
    if !config.activated {
        return Ok(false)
    }

    if access == AllowedAccess::Admin {
        // Check if admin
        if env.message.sender != config.owner {
            return Ok(false)
        }
    }
    return Ok(true)
}

fn register_receive<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    env: &Env,
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
        config.silk.code_hash,
        config.silk.address,
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

    fn create_contract(address: &str, code_hash: &str) -> Contract {
        let env = mock_env(address.to_string(), &[]);
        return Contract{
            address: env.message.sender,
            code_hash: code_hash.to_string()
        }
    }

    fn dummy_init(admin: String, silk: Contract, oracle: Contract) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
            silk,
            oracle,
            initial_assets: None
        };
        let env = mock_env(admin, &coins(1000, "earth"));
        let _res = init(&mut deps, env, msg).unwrap();

        return deps
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
            silk: create_contract("", ""),
            oracle: create_contract("", ""),
            initial_assets: None
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn config_update() {
        let silk_contract = create_contract("silk_contract", "silk_hash");
        let oracle_contract = create_contract("oracle_contract", "oracle_hash");
        let mut deps = dummy_init("admin".to_string(), silk_contract, oracle_contract);

        // Check config is properly updated
        let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        let silk_contract = create_contract("silk_contract", "silk_hash");
        let oracle_contract = create_contract("oracle_contract", "oracle_hash");
        match value {
            QueryAnswer::Config { config } => {
                assert_eq!(config.silk, silk_contract);
                assert_eq!(config.oracle, oracle_contract);

            }
            _ => { panic!("Received wrong answer") }
        }

        // Update config
        let user_env = mock_env("admin", &coins(1000, "earth"));
        let new_silk_contract = create_contract("new_silk_contract", "silk_hash");
        let new_oracle_contract = create_contract("new_oracle_contract", "oracle_hash");
        let msg = HandleMsg::UpdateConfig {
            owner: None,
            silk: Option::from(new_silk_contract),
            oracle: Option::from(new_oracle_contract),
        };
        let _res = handle(&mut deps, user_env, msg);

        // Check config is properly updated
        let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        let new_silk_contract = create_contract("new_silk_contract", "silk_hash");
        let new_oracle_contract = create_contract("new_oracle_contract", "oracle_hash");
        match value {
            QueryAnswer::Config { config } => {
                assert_eq!(config.silk, new_silk_contract);
                assert_eq!(config.oracle, new_oracle_contract);

            }
            _ => { panic!("Received wrong answer") }
        }

    }

    #[test]
    fn user_register_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""),
                                  create_contract("", ""));

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
                                  create_contract("", ""));

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
                                  create_contract("", ""));

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
            QueryAnswer::Asset { asset } => { assert_eq!("some_other_contract".to_string(), asset.contract.address.to_string()) }
            _ => { panic!("Received wrong answer") }
        };
    }

    #[test]
    fn nonexisting_update_asset() {
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

        // Should now be able to update non existing asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let bad_dummy_contract = create_contract("some_non_existing_contract", "some_hash");
        let new_dummy_contract = create_contract("some_other_contract", "some_hash");
        let msg = HandleMsg::UpdateAsset {
            asset: bad_dummy_contract.address,
            contract: new_dummy_contract,
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::NotFound { .. }) => {}
            _ => panic!("Must return not found error"),
        }
    }

    #[test]
    fn receiving_an_asset() {
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
        if res.is_err() {
            panic!("Must not return error");
        }
    }

    #[test]
    fn receiving_an_asset_from_non_supported_asset() {
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
}