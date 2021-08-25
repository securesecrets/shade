use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, from_binary};
use crate::state::{config, config_read, assets_w, assets_r, asset_list, asset_list_read};
use secret_toolkit::{
    snip20::{mint_msg, burn_msg, register_receive_msg, token_info_query, minters_query},
};
use shade_protocol::{
    mint::{InitMsg, HandleMsg, HandleAnswer, QueryMsg, QueryAnswer, MintConfig, SupportedAsset, SnipMsgHook},
    oracle::{
        QueryMsg::GetPrice,
    },
    band::ReferenceData,
    asset::{Contract},
    msg_traits::{Init, Query},
    generic_response::ResponseStatus,
};
use std::convert::TryFrom;

// TODO: add remove asset
// TODO: add spacepad padding
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
        oracle: msg.oracle,
        activated: true,
    };

    config(&mut deps.storage).save(&state)?;
    let mut messages = vec![];

    let empty_assets_list: Vec<String> = Vec::new();
    asset_list(&mut deps.storage).save(&empty_assets_list)?;

    if let Some(assets) = msg.initial_assets {
        for asset in assets {
            messages.push(save_asset(deps, &env, asset)?);
        }
    }
    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse {
        messages,
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
            oracle
        } => try_update_config(deps, env, owner, oracle),
        HandleMsg::RegisterAsset {
            name,
            contract,
            burnable,
            total_burned,
        } => try_register_asset(deps, &env, name, contract, burnable, total_burned),
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

    // Disable contract
    let mut config = config(&mut deps.storage);
    config.update(|mut state| {
        state.activated = false;
        Ok(state)
    })?;

    // Move all registered assets
    let mut initial_assets: Vec<SupportedAsset> = vec![];
    let config_read = config.load()?;
    let assets = assets_r(&deps.storage);
    for asset_addr in asset_list_read(&deps.storage).load()? {
        if let Some(item) = assets.may_load(asset_addr.as_bytes())? {
            initial_assets.push(item)
        }
    };

    // Move config
    let init_msg = InitMsg {
        admin: Option::from(config_read.owner),
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
    name: Option<String>,
    contract: Contract,
    burnable: Option<bool>,
    total_burned: Option<Uint128>,
) -> StdResult<HandleResponse> {

    let asset = SupportedAsset {
        name: match name {
            None => { token_info_query(&deps.querier, 1, contract.code_hash.clone(), contract.address.clone())?.symbol }
            Some(x) => x,
        },
        contract,
        burnable: match burnable {
            None => false,
            Some(x) => x
        },
        total_burned: match total_burned {
            None => Uint128(0),
            Some(amount) => amount,
        }
    };

    if !authorized(deps, &env, AllowedAccess::Admin)? {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut messages = vec![];
    messages.push(save_asset(deps, env, asset)?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::RegisterAsset {
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

    // Setup msgs
    let msgs: SnipMsgHook = match msg {
        Some(x) => from_binary(&x)?,
        None => return Err(StdError::generic_err("data cannot be empty")),
    };

    // Check that the assets are supported
    let assets = assets_r(&deps.storage);
    let burning_asset = match assets.may_load(env.message.sender.to_string().as_bytes())? {
        Some(asset) => asset,
        None => return Err(StdError::NotFound { kind: env.message.sender.to_string(), backtrace: None }),
    };
    let minting_asset = match assets.may_load(msgs.to_mint.to_string().as_bytes())? {
        Some(asset) => asset,
        None => return Err(StdError::NotFound { kind: msgs.to_mint.to_string(), backtrace: None }),
    };

    // Check that requested snip20 is supported and mint address is inside the mintable array
    let mintable = minters_query(&deps.querier, 1,
                                 minting_asset.contract.code_hash.clone(),
                                 minting_asset.contract.address.clone())?.minters;

    if !mintable.contains(&env.contract.address) {
        return Err(StdError::generic_err("Asset does allow mint contract to mint"))
    }

    // Query prices
    let in_price = call_oracle(&deps, burning_asset.name)?;
    let target_price = call_oracle(&deps, minting_asset.name)?;

    // Get asset decimals
    // Load the decimal information for both coins
    let in_decimals = token_info_query(&deps.querier, 1,
                                       burning_asset.contract.code_hash.clone(),
                                       burning_asset.contract.address.clone())?.decimals as u32;
    let target_decimals = token_info_query(&deps.querier, 1,
                                           minting_asset.contract.code_hash.clone(),
                                           minting_asset.contract.address.clone())?.decimals as u32;

    // Calculate value to mint
    let amount_to_mint = calculate_mint(in_price, target_price, amount, in_decimals, target_decimals);

    // If minimum amount is greater then ignore the process
    if msgs.minimum_expected_amount > amount_to_mint {
        return Err(StdError::generic_err("did not exceed expected amount"))
    }

    // if burnable then burn if not ignore
    if burning_asset.burnable {
        messages.push(burn_msg(amount, None, 256,
                               burning_asset.contract.code_hash,
                               burning_asset.contract.address)?);
    }

    // Set burned amount
    let mut mut_assets = assets_w(&mut deps.storage);
    mut_assets.update(env.message.sender.to_string().as_bytes(), |item| {
        let mut asset: SupportedAsset = item.unwrap();
        asset.total_burned += amount;
        Ok(asset)
    })?;

    // Mint
    messages.push(mint_msg(from, amount_to_mint, None, 256,
                           minting_asset.contract.code_hash,
                           minting_asset.contract.address)?);

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

fn register_receive (
    env: &Env,
    contract: Contract
) -> StdResult<CosmosMsg> {
    let cosmos_msg = register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        contract.code_hash,
        contract.address,
    );

    cosmos_msg
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
        in_total.multiply_ratio(1u128, 10u128.pow(u32::try_from(difference.abs()).unwrap()))
    }
    else if difference > 0 {
        Uint128(in_total.u128() * 10u128.pow(u32::try_from(difference).unwrap()))
    }
    else {
        in_total
    }
}

fn save_asset<S: Storage, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: SupportedAsset,
) -> StdResult<CosmosMsg> {

    let mut assets = assets_w(&mut deps.storage);

    // Save the asset
    let key = asset.contract.address.to_string();
    assets.save(key.as_bytes(), &asset)?;

    // Add the asset to list
    asset_list(&mut deps.storage).update(|mut state| {
        state.push(key);
        Ok(state)
    })?;

    // Register contract in asset
    let register_msg = register_receive(env, asset.contract)?;

    Ok(register_msg)
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

    fn dummy_init(admin: String, oracle: Contract) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
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
        let some_contract = create_contract("contract", "hash");
        let msg = InitMsg {
            admin: None,
            oracle: create_contract("", ""),
            initial_assets: Some(vec![SupportedAsset{
                name: "some_asset".to_string(),
                contract: some_contract,
                burnable: false,
                total_burned: Uint128(0)
            }])
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        // We should receive two registered messages
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn config_update() {
        let oracle_contract = create_contract("oracle_contract", "oracle_hash");
        let mut deps = dummy_init("admin".to_string(), oracle_contract);

        // Check config is properly updated
        let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        let oracle_contract = create_contract("oracle_contract", "oracle_hash");
        match value {
            QueryAnswer::Config { config } => {
                assert_eq!(config.oracle, oracle_contract);
            }
            _ => { panic!("Received wrong answer") }
        }

        // Update config
        let user_env = mock_env("admin", &coins(1000, "earth"));
        let new_oracle_contract = create_contract("new_oracle_contract", "oracle_hash");
        let msg = HandleMsg::UpdateConfig {
            owner: None,
            oracle: Option::from(new_oracle_contract),
        };
        let _res = handle(&mut deps, user_env, msg);

        // Check config is properly updated
        let res = query(&deps, QueryMsg::GetConfig {}).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        let new_oracle_contract = create_contract("new_oracle_contract", "oracle_hash");
        match value {
            QueryAnswer::Config { config } => {
                assert_eq!(config.oracle, new_oracle_contract);

            }
            _ => { panic!("Received wrong answer") }
        }

    }

    #[test]
    fn user_register_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""));

        // User should not be allowed to add an item
        let user_env = mock_env("user", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            name: Some("asset".to_string()),
            contract: dummy_contract,
            burnable: None,
            total_burned: None
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
            _ => { panic!("Expected empty array") }
        }
    }

    #[test]
    fn admin_register_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""));

        // Admin should be allowed to add an item
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            name: Some("asset".to_string()),
            contract: dummy_contract,
            burnable: None,
            total_burned: None
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
    fn admin_update_asset() {
        let mut deps = dummy_init("admin".to_string(),
                                  create_contract("", ""));

        // Add a supported asset
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            name: Some("old_asset".to_string()),
            contract: dummy_contract,
            burnable: None,
            total_burned: None
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // admins can update assets
        let env = mock_env("admin", &coins(1000, "earth"));
        let dummy_contract = create_contract("some_contract", "some_hash");
        let msg = HandleMsg::RegisterAsset {
            name: Some("new_asset".to_string()),
            contract: dummy_contract,
            burnable: None,
            total_burned: None
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // Response should be new dummy contract
        let res = query(&deps, QueryMsg::GetAsset { contract: "some_contract".to_string() }).unwrap();
        let value: QueryAnswer = from_binary(&res).unwrap();
        match value {
            QueryAnswer::Asset { asset } => { assert_eq!("new_asset".to_string(), asset.name) }
            _ => { panic!("Received wrong answer") }
        };
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
