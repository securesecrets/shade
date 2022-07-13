use shade_protocol::c_std::{
    Storage, Api, Querier, Extern, Env, StdResult, Response, to_binary,
    StdError, Addr, CosmosMsg, Binary, WasmMsg
};
use shade_protocol::fadroma::scrt::to_cosmos_msg;
use shade_protocol::c_std::Uint128;
use shade_protocol::{
    utils::{asset::Contract, storage::plus::ItemStorage},
    contract_interfaces::{
    sky::sky::{
        Config, HandleAnswer, self
    },
    dex::sienna::{PairQuery, TokenTypeAmount, PairInfoResponse, TokenType, Swap, SwapOffer, CallbackMsg, CallbackSwap},
    mint::mint::{QueryAnswer, QueryMsg, QueryAnswer::Mint, HandleMsg::Receive, self},  
    snip20::helpers::Snip20Asset,
}};
use shade_protocol::secret_toolkit::utils::Query;
use shade_protocol::snip20::helpers::send_msg;
use crate::{query::trade_profitability};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<Response> {
    if env.message.sender != Config::load(&deps.storage)?.admin {
        return Err(StdError::unauthorized())
    }
    config.save(&mut deps.storage)?;
    Ok(Response{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig{
            status: true,
        })?),
    })
}

/*pub fn try_arbitrage_event<S: Storage, A: Api, Q: Querier>( //DEPRECIATED
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<Response> {
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

    Ok(Response{
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
) -> StdResult<Response> {

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
            config.mint_addr.address.clone(),
            config.mint_addr.code_hash.clone(),
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
            Uint128::new(first_swap_min_expected.clone().u128()),
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
            Uint128::new(amount.u128()),
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
            config.mint_addr.address.clone(),
            config.mint_addr.code_hash.clone(),
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

    Ok(Response{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArb{
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