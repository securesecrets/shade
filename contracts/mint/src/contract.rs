use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, OracleCall, SupportedAssetsResponse, AssetResponse};
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
            ..} => try_receive(deps, env, sender, from, amount, msg),
    }
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
    silk_contract: HumanAddr,
    silk_contract_code_hash: String,
    oracle_contract: HumanAddr,
    oracle_contract_code_hash: String,
) -> StdResult<HandleResponse> {
    let mut config = config(&mut deps.storage);

    // Check if admin
    if env.message.sender != config.load()?.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Resave new info
    config.update(|mut state| {
        state.owner = owner;
        state.silk_contract = silk_contract;
        state.oracle_contract_code_hash = silk_contract_code_hash;
        state.oracle_contract = oracle_contract;
        state.oracle_contract_code_hash = oracle_contract_code_hash;
        Ok(state)
    })?;

    //TODO: log update information
    //TODO: make state info optionals

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None
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
            match assets.save(contract_str.as_bytes(), &Asset {
                contract: contract.clone(),
                code_hash: code_hash.clone(),
                burned_tokens: Uint128(0),
            }) {
                Err(err) => return Err(err),
                _ => {}
            };

            match asset_list(&mut deps.storage).update(|mut state| {
                state.push(contract_str);
                Ok(state)
            }) {
                Err(err) => return Err(err),
                _ => {}
            };

            match register_receive(&deps, env, contract, code_hash) {
                Err(err) => return Err(err),
                Ok(register_msg) => { messages.push(register_msg); },
            };
        }
    }

    //TODO: log info

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None
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
            let register_msg = register_receive(&deps, env, contract, code_hash)?;
            messages.push(register_msg)
        },

        None => return Err(StdError::NotFound { kind: asset_str, backtrace: None }),
    }

    //TODO: log info

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None
    })
}

pub fn try_receive<S: Storage, A: Api, Q: Querier>(
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

    // First get the current shade value per coin
    let token_value:u128 = call_oracle(deps, env.clone(), env.message.sender.clone())?.into();
    let amount_converted:u128 = amount.into();

    // // Calculate shade amount to mint
    let value_to_mint = amount_converted * token_value;

    let mut messages = vec![];

    match mint_shade(deps, from, Uint128::from(value_to_mint)) {
        Err(err) => { return Err(err) },
        Ok(msg) => { messages.push(msg) }
    };
    //TODO: log info

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None,
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

fn mint_shade<S: Storage, A: Api, Q: Querier>(
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
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    contract: HumanAddr,
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
    }
}

fn query_supported_assets<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<SupportedAssetsResponse> {
    Ok(SupportedAssetsResponse { assets: asset_list_read(&deps.storage).load()? })
}

fn query_asset<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, contract: String) -> StdResult<AssetResponse> {
    let assets = assets_r(&deps.storage);

    return match assets.may_load(contract.as_bytes())? {
        Some(asset) => Ok(AssetResponse { asset }),
        None => Err(StdError::NotFound { kind: contract, backtrace: None }),
    };


}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{coins, from_binary, StdError};
    use crate::msg::AssetResponse;

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
        let value: SupportedAssetsResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.assets.len());
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
        let value: SupportedAssetsResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.assets.len());
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
        }
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
        }
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
        let value: AssetResponse = from_binary(&res).unwrap();
        assert_eq!("some_other_contract".to_string(), value.asset.contract.to_string());
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
        match res {
            Err(err) => {panic!("Must not return error")}
            _ => {},
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