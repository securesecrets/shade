use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, CosmosMsg, CanonicalAddr, WasmMsg};

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg, OracleCall};
use crate::state::{config, config_read, assets, assets_read, Config, Native_Coin, Assets, Asset};
use secret_toolkit::snip20::{mint_msg, register_receive_msg};
use secret_toolkit::utils::space_pad;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        count: msg.count,
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

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
            msg } => try_receive(deps, env, sender, from, amount, msg),
    }
}

pub fn try_update_config<s: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: CanonicalAddr,
    silk_contract: CanonicalAddr,
    silk_contract_code_hash: String,
    oracle_contract: CanonicalAddr,
    oracle_contract_code_hash: String,
) -> StdResult<HandleResponse> {
    let mut config = config(deps.storage);

    // Check if admin
    assert_eq!(deps.api.human_address(*config.load()?.owner), env.message.sender);

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

pub fn try_register_asset<s: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    contract: CanonicalAddr,
    code_hash: String,
) -> StdResult<HandleResponse> {
    let config = config_read(deps.storage);

    let contract_str = contract.to_string();

    // Check if admin
    assert_eq!(deps.api.human_address(*config.load()?.owner), env.message.sender);

    let mut assets = assets(deps.storage);

    // Check if asset already exists
    if assets.load()?.coins.contains_key(&*contract_str) {
        //TODO: actually handle the panic
        panic!("Asset already exists")
    }

    assets.update(|mut state| {
        state.coins.insert(contract_str, Asset{
            contract,
            code_hash,
            burned_tokens: 0,
        });
        Ok(state)
    })?;

    //TODO: log info

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None
    })
}

pub fn try_update_asset<s: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: CanonicalAddr,
    contract: CanonicalAddr,
    code_hash: String,
) -> StdResult<HandleResponse> {
    let config = config_read(deps.storage);

    let contract_str = asset.to_string();

    // Check if admin
    assert_eq!(deps.api.human_address(*config.load()?.owner), env.message.sender);

    let mut assets = assets(deps.storage);

    // Check if asset already exists
    if assets.load()?.coins.contains_key(&*contract_str) {
        //TODO: actually handle the panic
        panic!("Asset already exists")
    }

    assets.update(|mut state| {
        let mut token = state.coins.get_mut(&*contract_str).unwrap();

        token.contract = contract;
        token.code_hash = code_hash;

        Ok(state)
    })?;

    //TODO: log info

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None
    })
}

pub fn try_receive<s: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: CanonicalAddr,
    from: CanonicalAddr,
    amount: uint128,
    msg: Option<CosmosMsg>
) -> StdResult<HandleResponse> {
    // Check if the contract is still allowed
    let assets = assets_read(&deps.storage).load()?;

    // Check that the contract sender is the same as the message sender
    assert_eq!(sender, env.message.sender);

    match assets.coins.get(*sender) {
        None => { return  Err(StdError::generic_err("SNIP20 contract does not match list of supported contracts."))}
        Some(foundAsset) => {}
    }

    //TODO: add burned token amount

    // First get the current shade value per coin
    let token_value = call_oracle(deps, env, sender)?;

    // Calculate shade amount to mint
    let value_to_mint = amount * token_value;

    let msg = mint_shade(deps, from, value_to_mint)?;

    //TODO: log info

    Ok(HandleResponse {
        messages: vec![msg],
        log: vec![],
        data: None,
    })
}

// Helper functions

fn register_receive<s: Storage, A: Api, Q: Querier>(
    deps: Extern<S, A, Q>,
    env: Env,
    contract: CanonicalAddr,
    code_hash: String,
) -> StdResult<CosmosMsg> {
    let cosmos_msg = register_receive_msg(
        env.contract_code_hash,
        None,
        256,
        code_hash,
        deps.api.human_address(*contract),
    );

    cosmos_msg
}

fn mint_shade<s: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    sender: CanonicalAddr,
    amount: uint128,
) -> StdResult<CosmosMsg> {
    let config = config_read(&deps.storage).load()?;

    let cosmos_msg = mint_msg(
        deps.api.human_address(*sender),
        amount,
        None,
        256,
        config.silk_contract_code_hash,
        deps.api.human_address(*config.silk_contract),
    );

    cosmos_msg
}

fn call_oracle<s: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    contract: CanonicalAddr,
) -> StdResult<uint128> {
    // Call contract
    let block_size = 1; //update this later
    let config = config_read(&deps.storage).load()?;
    let mut msg = to_binary(&&OracleCall{ contract })?;
    space_pad(&mut msg.0, block_size);
    let execute = WasmMsg::Execute {
        contract_addr: deps.api.human_address(*config.oracle_contract),
        callback_code_hash: config.oracle_contract_code_hash,
        msg,
        send: vec![]
    };
    // somehow handle execute and get a uint128 value
    let value: uint28 = 1;
    OK(value)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(CountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can increment
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Increment {};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // not anyone can reset
        let unauth_env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_env = mock_env("creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_env, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
