use cosmwasm_std::{
    Storage, Api, Querier, Extern, Env, StdResult, HandleResponse, to_binary, 
    StdError, HumanAddr, CosmosMsg, Binary, WasmMsg
};
use fadroma::scrt::to_cosmos_msg;
use cosmwasm_math_compat::Uint128;
use shade_protocol::{
    utils::{asset::Contract, storage::plus::ItemStorage},
    contract_interfaces::{
    sky::sky::{
        Config, HandleAnswer, self, ViewingKeys, Cycle, Cycles
    },
    dex::{
        self,
        sienna::{PairQuery, TokenTypeAmount, PairInfoResponse, Swap, SwapOffer, CallbackMsg, CallbackSwap},
        shadeswap::{TokenType},
    },
    mint::mint::{QueryAnswer, QueryMsg, QueryAnswer::Mint, HandleMsg::Receive, self},  
    snip20::helpers::Snip20Asset,
}};
use secret_toolkit::{utils::Query, snip20::set_viewing_key_msg};
use secret_toolkit::snip20::send_msg;
use crate::{query::trade_profitability};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    if env.message.sender != Config::load(&deps.storage)?.admin {
        return Err(StdError::unauthorized())
    }
    config.save(&mut deps.storage)?;
    let view_key = ViewingKeys::load(&deps.storage)?.0;
    let mut messages = vec![
        set_viewing_key_msg(
            view_key.clone(), 
            None, 
            1, 
            config.shd_token.contract.code_hash.clone(), 
            config.shd_token.contract.address.clone(),    
        )?,
        set_viewing_key_msg(
            view_key.clone(), 
            None, 
            1, 
            config.silk_token.contract.code_hash.clone(), 
            config.silk_token.contract.address.clone()
        )?
    ];
    Ok(HandleResponse{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig{
            status: true,
        })?),
    })
}

pub fn try_set_cycles<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cycles_to_set: Vec<Cycle>,
) -> StdResult<HandleResponse> {
    if env.message.sender != Config::load(&deps.storage)?.admin {
        return Err(StdError::unauthorized())
    }

    let new_cycles = Cycles ( cycles_to_set );
    new_cycles.save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetCycles{
            status: true,
        })?),
    })
}

pub fn try_append_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cycles_to_add: Vec<Cycle>,
) -> StdResult<HandleResponse> {
    if env.message.sender != Config::load(&deps.storage)?.admin {
        return Err(StdError::unauthorized())
    }

    let mut cycles = Cycles::load(&deps.storage)?;

    cycles.0.append(&mut cycles_to_add.clone());

    cycles.save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AppendCycles{
            status: true,
        })?),
    })
}

/*pub fn try_arbitrage_event<S: Storage, A: Api, Q: Querier>( //DEPRECIATED
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config: Config = Config::load(&deps.storage)?;
    let pool_info: PairInfoResponse = PairQuery::PairInfo.query(
        &deps.querier,
        env.contract_code_hash.clone(),//TODO
        config.market_swap_addr.address.clone(),
    )?;
    let test_amount: u128 = 100000000;
    let mint_info: QueryAnswer = QueryMsg::Mint{
        offer_asset: config.shd_token.contract.address.clone(),
        amount: Uint128::new(test_amount),
    }.query(
        &deps.querier,
        env.contract_code_hash.clone(),//TODO
        config.mint_addr.address.clone(),
    )?;
    let mut mint_price: Uint128 = Uint128::zero();
    match mint_info{
        QueryAnswer::Mint {
            asset,
            amount,
        } => {
            mint_price = amount;
        },
        _ => {
            return Err(StdError::GenericErr { 
                msg: "Query returned with unexpected result".to_string(), 
                backtrace: None 
            });
        },
    };
    let mut nom = Uint128::zero();
    let mut denom = Uint128::zero();
    if pool_info.pair_info.amount_0.u128().lt(&pool_info.pair_info.amount_1.u128()) {
        nom = pool_info.pair_info.amount_1.checked_mul(Uint128::new(100000000))?;
        denom = pool_info.pair_info.amount_0.clone();
    } else {
        nom = pool_info.pair_info.amount_0.checked_mul(Uint128::new(100000000))?;
        denom = pool_info.pair_info.amount_1.clone();
    }
    let mut market_price: Uint128 = nom.checked_mul(denom)?; // silk/shd
    

    let mut messages = vec![];
    if mint_price.lt(&market_price) { //swap then mint
        //take out swap fees here
        let first_swap = constant_product(
            amount.clone(), 
            nom.checked_div(Uint128::new(100000000))?, 
            denom.clone()
        )?;
        let second_swap = first_swap.checked_div(mint_price)?;
        let mut msg = Swap{
            send: SwapOffer{
                recipient: config.market_swap_addr.address.clone(),
                amount,
                msg: to_binary(&{})?
            }
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute{ //swap
            contract_addr: config.shd_token.contract.address.clone(),
            callback_code_hash: env.contract_code_hash.clone(),
            msg: to_binary(&msg)?,
            send: vec![],
        }));
        //let expected = {
        //    expected_amount: second_swap.clone(),
        //};
        let msg = Receive{
            amount: first_swap.clone(),
            from: config.silk_token.contract.address.clone(),
            memo: Some(to_binary("")?),
            sender:  env.contract.address.clone(),
            msg: Some(to_binary(&"TODO".to_string())?),
        };
        let data = to_binary(&msg)?;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute{ //mint
            contract_addr: config.mint_addr.address.clone(),
            callback_code_hash: "".to_string(),
            msg: data,
            send: vec![],
        }));
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute{ //swap
            contract_addr: config.shd_token.contract.address.clone(),
            callback_code_hash: "".to_string(),
            msg: Binary(vec![]),
            send: vec![],
        }));
    }else{ //mint then swap
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute{ //swap
            contract_addr: config.shd_token.contract.address.clone(),
            callback_code_hash: "".to_string(),
            msg: Binary(vec![]),
            send: vec![],
        }));
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute{ //mint
            contract_addr: config.shd_token.contract.address.clone(),
            callback_code_hash: "".to_string(),
            msg: Binary(vec![]),
            send: vec![],
        }));
    }

    Ok(HandleResponse{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArb{
            status: true,
        })?)
    })
}*/

pub fn try_execute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let config: Config = Config::load(&deps.storage)?;

    //if amount.gt(env.)
    
    let res = trade_profitability( deps, amount )?;

    let mut profitable = false;
    let mut is_mint_first = false;
    let mut pool_shd_amount = Uint128::zero();
    let mut pool_silk_amount = Uint128::zero();
    let mut first_swap_min_expected = Uint128::zero();
    let mut second_swap_min_expected = Uint128::zero();
    match res {
        sky::QueryAnswer::TestProfitability{
            is_profitable, 
            mint_first, 
            shd_amount,
            silk_amount,
            first_swap_amount, 
            second_swap_amount,
        } => {
            profitable = is_profitable;
            is_mint_first = mint_first;
            pool_shd_amount = shd_amount;
            pool_silk_amount = silk_amount;
            first_swap_min_expected = first_swap_amount;
            second_swap_min_expected = second_swap_amount;
        }
        _ => {}
    }
    
    let mut messages = vec![];
    let mut mint_msg: mint::HandleMsg;
    let mut sienna_msg: Swap;

    if is_mint_first {
        messages.push(to_cosmos_msg(
            config.mint_addr_silk.address.clone(),
            config.mint_addr_silk.code_hash.clone(),
            &mint::HandleMsg::Receive{
                sender: env.contract.address.clone(),
                from: config.shd_token.contract.address.clone(),
                amount: amount.clone(),
                memo: None,
                msg: Some(to_binary(&mint::MintMsgHook{
                    minimum_expected_amount: first_swap_min_expected
                })?)
            },
        )?);

        messages.push(send_msg(
            config.market_swap_addr.address.clone(),
            cosmwasm_std::Uint128(first_swap_min_expected.clone().u128()),
            Some(to_binary(&CallbackSwap{
                expected_return: second_swap_min_expected.clone(),
            })?),
            None,
            None,
            256,
            config.silk_token.contract.code_hash.clone(),
            config.silk_token.contract.address.clone(),
        )?);
    }
    else {
        messages.push(send_msg(
            config.market_swap_addr.address.clone(),
            cosmwasm_std::Uint128(amount.u128()),
            Some(to_binary(&CallbackSwap{
                expected_return: first_swap_min_expected,
            })?),
            None,
            None,
            256,
            config.shd_token.contract.code_hash.clone(),
            config.shd_token.contract.address.clone(),
        )?);

        messages.push(to_cosmos_msg(
            config.mint_addr_shd.address.clone(),
            config.mint_addr_shd.code_hash.clone(),
            &mint::HandleMsg::Receive{
                sender: env.contract.address.clone(),
                from: config.silk_token.contract.address.clone(),
                amount: first_swap_min_expected,
                memo: None,
                msg: Some(to_binary(&mint::MintMsgHook{
                    minimum_expected_amount: second_swap_min_expected
                })?)
            },
        )?);
    }

    Ok(HandleResponse{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArb{
            status: true,
        })?)
    })
}

pub fn try_arb_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    index: Uint128,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    Ok(HandleResponse{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArbCycle{
            status: true,
        })?)
    })
}

pub fn try_arb_all_cycles<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    Ok(HandleResponse{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArbAllCycles{
            status: true,
        })?)
    })
}

pub fn constant_product(swap_amount: Uint128, pool_buy: Uint128, pool_sell: Uint128) -> StdResult<Uint128> {
    //let cp = pool_buy.u128().clone() * pool_sell.u128().clone();
    //let lpb = pool_sell.u128().clone() + swap_amount.u128().clone();
    //let ncp = div(Uint128::new(cp.clone()), Uint128::new(lpb.clone()))?;
    //let result = pool_buy.u128().clone() - ncp.u128().clone();
    let cp = pool_buy.checked_mul(pool_sell)?;
    let lpb = pool_sell.checked_add(swap_amount)?;
    let ncp = cp.checked_div(lpb)?;
    let result = pool_buy.checked_sub(ncp)?;

    Ok(result)
}