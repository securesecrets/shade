use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, 
    Env, Extern, HandleResponse, InitResponse, 
    Querier, StdResult, StdError, Storage, Uint128,
    HumanAddr,
};
use crate::state::{config, config_read};
use shade_protocol::{
    oracle::{
        InitMsg, HandleMsg, HandleAnswer,
        QueryMsg, QueryAnswer, OracleConfig, ReferenceData
    },
    asset::Contract,
    msg_traits::Query,
    generic_response::ResponseStatus,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = OracleConfig {
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        band: msg.band,
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
            band
        } => try_update_config(deps, env, owner, band),
    }
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    band: Contract,
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
        state.band = band;

        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig{
            status: ResponseStatus::Success } )? )
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetReferenceData { base_symbol, quote_symbol } => 
            to_binary(&query_reference_data(deps, base_symbol, quote_symbol)?),
        QueryMsg::GetPrice{ symbol } => 
            to_binary(&query_reference_data(deps, symbol, "USD".to_string())?),
    }
}

fn query_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config { config: config_read(&deps.storage).load()? })
}

// cross-contract query
fn query_reference_data<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    base_symbol: String,
    quote_symbol: String,
) -> StdResult<ReferenceData> {

    if base_symbol == "SHD" {
        // this can read from the local storage
        return Ok(ReferenceData {
            //11.47 * 10^18
            rate: Uint128(1147 * 10u128.pow(16)),
            last_updated_base: 0,
            last_updated_quote: 0
        });
    }

    let config_read = config_read(&deps.storage).load()?;

    // If band oracle is not defined it will return a default value
    Ok(QueryMsg::GetReferenceData {
            base_symbol,
            quote_symbol
    }.query(
        &deps.querier,
        1, //block_size
        config_read.band.code_hash,
        config_read.band.address)?)
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

    if access == AllowedAccess::Admin {
        // Check if admin
        if env.message.sender != config.owner {
            return Ok(false)
        }
    }
    return Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{coins, from_binary};
    use shade_protocol::asset::Contract;
    use mockall::{automock, predicate::*};

    fn create_contract(address: &str, code_hash: &str) -> Contract {
        let env = mock_env(address.to_string(), &[]);
        return Contract{
            address: env.message.sender,
            code_hash: code_hash.to_string()
        }
    }

    fn dummy_init(admin: &str, band: Contract) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
            band: band,
        };
        let env = mock_env(admin.to_string(), &coins(1000, "earth"));
        let _res = init(&mut deps, env, msg).unwrap();

        return deps
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            admin: None,
            band: create_contract("", ""),
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[cfg_attr(test, automock)]
    trait Query{
        fn query(&self,
            _querier: &QueryMsg,
            _block_size: usize,
            _callback_code_hash: String,
            _contract_addr: HumanAddr,
        ) -> StdResult<ReferenceData> {
            Ok(ReferenceData {
                //11.47 * 10^18
                rate: Uint128(1147 * 10u128.pow(16)),
                last_updated_base: 0,
                last_updated_quote: 0
            })
        }
    }

    #[test]
    fn price_query() {
        let mut deps = dummy_init(&"admin".to_string(),
                                  create_contract("", ""));
        let msg = QueryMsg::GetPrice{
            symbol: "SHD".to_string(),
        };
        let res = query(&mut deps, msg).unwrap();
        let value: ReferenceData = from_binary(&res).unwrap();
        assert_eq!(value.rate, Uint128(1147 * 10u128.pow(16)))
    }
}
