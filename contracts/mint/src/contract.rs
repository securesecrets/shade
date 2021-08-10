use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, from_binary};
use crate::state::{config, config_read, assets_w, assets_r, asset_list, asset_list_read};
use secret_toolkit::{
    snip20::{mint_msg, register_receive_msg, token_info_query},
};
use shade_protocol::{
    mint::{InitMsg, HandleMsg, HandleAnswer, QueryMsg, QueryAnswer, AssetMsg, MintConfig, BurnableAsset},
    oracle::{
        ReferenceData,
        QueryMsg::GetPrice,
    },
    asset::{Contract},
    msg_traits::{Init, Query},
};
use shade_protocol::generic_response::ResponseStatus;
use shade_protocol::mint::{SnipMsgHook, MintType};
use std::convert::TryFrom;

// TODO: add remove asset
// TODO: add spacepad padding
// TODO: father contract must be snip20 contract owner
// TODO: father contract must change minters when migrating
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = MintConfig {
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        silk: msg.silk,
        shade: msg.shade,
        oracle: msg.oracle,
        activated: true,
    };

    config(&mut deps.storage).save(&state)?;

    let empty_assets_list: Vec<String> = Vec::new();
    asset_list(&mut deps.storage).save(&empty_assets_list)?;

    if let Some(assets) = msg.initial_assets {
        for asset in assets {
            let _response = try_register_asset(deps, &env, asset.contract, asset.burned_tokens);
        }
    }
    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse {
        messages: vec![],
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Migrate {
            code_id,
            code_hash,
            label
        } => try_migrate(deps, env, label, code_id, code_hash),
        HandleMsg::UpdateConfig {
            owner,
            silk,
            shade,
            oracle
        } => try_update_config(deps, env, owner, silk, shade, oracle),
        HandleMsg::RegisterAsset {
            contract,
        } => try_register_asset(deps, &env, contract, None),
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

pub fn try_migrate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    label: String,
    code_id: u64,
    code_hash: String,
) -> StdResult<HandleResponse> {
    if !authorized(deps, &env, AllowedAccess::Admin)? {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut config = config(&mut deps.storage);
    config.update(|mut state| {
        state.activated = false;
        Ok(state)
    })?;

    let config_read = config.load()?;
    let mut initial_assets: Vec<AssetMsg> = vec![];
    let assets = assets_r(&deps.storage);

    for asset_addr in asset_list_read(&deps.storage).load()? {
        if let Some(item) = assets.may_load(asset_addr.as_bytes())? {
            initial_assets.push(AssetMsg {
                contract: item.contract,
                burned_tokens: Some(item.burned_tokens),
            })
        }
    };

    let init_msg = InitMsg {
        admin: Option::from(config_read.owner),
        silk: config_read.silk,
        shade: config_read.shade,
        oracle: config_read.oracle,
        initial_assets: Some(initial_assets)
    };

    Ok(HandleResponse {
        messages: vec![init_msg.to_cosmos_msg(1, code_id, code_hash, label)?],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Migrate {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    silk: Option<Contract>,
    shade: Option<Contract>,
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
        if let Some(shade) = shade {
            state.shade = shade;
        }
        if let Some(oracle) = oracle {
            state.oracle = oracle;
        }
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: Contract,
    burned_amount: Option<Uint128>
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
            assets.save(contract_str.as_bytes(), &BurnableAsset {
                contract: contract.clone(),
                burned_tokens: match burned_amount {
                    None => { Uint128(0) }
                    Some(amount) => { amount }
                },
            })?;
            // Add asset to list
            asset_list(&mut deps.storage).update(|mut state| {
                state.push(contract_str);
                Ok(state)
            })?;
            // Register contract in asset
            let register_msg = register_receive(env, contract.address, contract.code_hash)?;
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
            assets.save(contract.address.to_string().as_bytes(), &BurnableAsset {
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
            let register_msg = register_receive(&env, contract.address, contract.code_hash)?;
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
    msg: Option<Binary>
) -> StdResult<HandleResponse> {
    if !authorized(deps, &env, AllowedAccess::User)? {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    let mut messages = vec![];

    // Get mint command
    let amount_to_mint: Uint128;
    let mint_response: StdResult<(CosmosMsg, Uint128)>;
    if let Some(msg) = msg {
        let mint_msg: SnipMsgHook = from_binary(&msg)?;
        mint_response = match mint_msg.mint_type {
            MintType::MintSilk {} => mint_with_asset(deps, &env, TargetCoin::Silk, from, amount, mint_msg.minimum_expected_amount),
            MintType::MintShade {} => mint_with_asset(deps, &env, TargetCoin::Shade, from, amount, mint_msg.minimum_expected_amount)
        }
    } else {
        return Err(StdError::generic_err("data cannot be empty"))
    }

    // Handle response
    match mint_response {
        Ok(tuple) => {
            let (cosmos_msg, amount_returned) = tuple;
            messages.push(cosmos_msg);
            amount_to_mint = amount_returned;
        }
        Err(err) => { return Err(err) }
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Burn {
            status: ResponseStatus::Success,
            mint_amount: amount_to_mint
        } )? ),
    })
}

// Helper functions

#[derive(PartialEq)]
pub enum AllowedAccess{
    Admin,
    User,
}

fn authorized<S: Storage, A: Api, Q: Querier>(
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

fn calculate_mint(in_price: Uint128, target_price: Uint128, in_amount: Uint128, in_decimals: u32, target_decimals: u32) -> Uint128 {
    // Math must only be made in integers
    // in_decimals  = x
    // target_decimals = y
    // in_price     = p1 * 10^18
    // target_price = p2 * 10^18
    // in_amount    = a1 * 10^x
    // return       = a2 * 10^y

    // (a1 * 10^x) * (p1 * 10^18) = (a2 * 10^y) * (p2 * 10^18)

    //                (p1 * 10^18)
    // (a1 * 10^x) * --------------  = (a2 * 10^y)
    //                (p2 * 10^18)

    let in_total = in_amount.multiply_ratio(in_price, target_price);

    // in_total * 10^(y - x) = (a2 * 10^y)
    let difference: i32 = target_decimals as i32 - in_decimals as i32;

    // To avoid a mess of different types doing math
    if difference < 0 {
        in_total.multiply_ratio(1 as u128, 10u128.pow(u32::try_from(difference.abs()).unwrap()))
    }
    else if difference > 0 {
        Uint128(in_total.u128() * 10u128.pow(u32::try_from(difference).unwrap()))
    }
    else {
        in_total
    }
}

fn register_receive (
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

enum TargetCoin {
    Silk,
    Shade,
}

fn mint_with_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    target: TargetCoin,
    sender: HumanAddr,
    amount: Uint128,
    minimum_expected_amount: Uint128,
) -> StdResult<(CosmosMsg, Uint128)> {

    // Check that the asset is supported
    let assets = assets_r(&deps.storage);
    let asset_contract: Contract;
    match assets.may_load(env.message.sender.to_string().as_bytes())? {
        Some(asset) => { asset_contract = asset.contract; },
        None => return Err(StdError::NotFound { kind: env.message.sender.to_string(), backtrace: None }),
    }

    // Query the coin values
    let (in_price, target_price) = match target {
        TargetCoin::Silk => (call_oracle(&deps, "SCRT".to_string())?, Uint128(10u128.pow(18))),
        TargetCoin::Shade => (call_oracle(&deps, "SCRT".to_string())?, call_oracle(&deps, "SHD".to_string())?)
    };

    // Get the target coin information
    let config = config_read(&deps.storage).load()?;

    let target_coin = match target {
        TargetCoin::Silk => config.silk,
        TargetCoin::Shade => config.shade
    };

    // Load the decimal information for both coins
    let in_decimals = token_info_query(&deps.querier, 1, asset_contract.code_hash,
                                       asset_contract.address)?.decimals as u32;
    let target_decimals = token_info_query(&deps.querier, 1,
                                           target_coin.code_hash.clone(),
                                           target_coin.address.clone())?.decimals as u32;

    // This will calculate the total mind value
    let amount_to_mint = calculate_mint(in_price, target_price, amount, in_decimals, target_decimals);

    // If minimum amount is greater then ignore the process
    if minimum_expected_amount > amount_to_mint {
        return Err(StdError::generic_err("did not exceed expected amount"))
    }

    let mint_msg = mint_msg(sender, amount_to_mint, None,
                            256, target_coin.code_hash,
                            target_coin.address)?;

    // Set burned amount
    let mut mut_assets = assets_w(&mut deps.storage);
    mut_assets.update(env.message.sender.to_string().as_bytes(), |item| {
        let mut asset: BurnableAsset = item.unwrap();
        asset.burned_tokens += amount;
        Ok(asset)
    })?;

    Ok((mint_msg, amount_to_mint))
}

fn call_oracle<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<Uint128> {
    let block_size = 1; //update this later
    let config = config_read(&deps.storage).load()?;
    let query_msg = GetPrice { symbol };
    let answer: ReferenceData = query_msg.query(&deps.querier, block_size,
                                 config.oracle.code_hash,
                                 config.oracle.address)?;
    Ok(answer.rate)
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
    use shade_protocol::mint::QueryAnswer;

    fn create_contract(address: &str, code_hash: &str) -> Contract {
        let env = mock_env(address.to_string(), &[]);
        return Contract{
            address: env.message.sender,
            code_hash: code_hash.to_string()
        }
    }

    fn dummy_init(admin: String, silk: Contract, shade: Contract, oracle: Contract) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
            silk,
            shade,
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
            shade: create_contract("", ""),
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
        let shade_contract = create_contract("shade_contract", "shade_hash");
        let oracle_contract = create_contract("oracle_contract", "oracle_hash");
        let mut deps = dummy_init("admin".to_string(), silk_contract, shade_contract, oracle_contract);

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
            shade: None,
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
            msg: Option::from(to_binary(&SnipMsgHook {
                minimum_expected_amount: Uint128(1),
                mint_type: MintType::MintSilk {}
            }).unwrap()),
            memo: None
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::NotFound { .. }) => {}
            _ => {panic!("Must return not found error")},
        }
    }

    #[test]
    fn mint_algorithm_simple() {
        // In this example the "sent" value is 1 with 6 decimal places
        // The mint value will be 1 with 3 decimal places
        let price = Uint128(1_000_000_000_000_000_000);
        let in_amount = Uint128(1_000_000);
        let expected_value = Uint128(1_000);
        let value = calculate_mint(price, price, in_amount, 6, 3);

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
        let value = calculate_mint(in_price, target_price, in_amount, 6, 12);

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
        let value = calculate_mint(in_price, target_price, in_amount, 3, 6);

        assert_eq!(value, expected_value);
    }
}
