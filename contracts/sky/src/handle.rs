use std::ptr::null;

use cosmwasm_std::{
    Storage, Api, Querier, Extern, Env, StdResult, HandleResponse, to_binary, 
    Uint128, StdError, HumanAddr, CosmosMsg, Binary, WasmMsg
};
use shade_protocol::{
    sky::{
        Config, HandleAnswer, self
    },
    sienna::{PairQuery, TokenTypeAmount, PairInfoResponse, TokenType, Swap, SwapOffer, CallbackMsg, CallbackSwap},
    mint::{QueryAnswer, QueryMsg, QueryAnswer::Mint, HandleMsg::Receive, self}, 
    utils::{math::div, asset::Contract}, 
    snip20::Snip20Asset,
};
use secret_toolkit::utils::Query;
use crate::{state::config_r, query::trade_profitability};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig{
            status: true,
        })?),
    })
}

pub fn try_arbitrage_event<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config: Config = config_r(&deps.storage).load()?;
    let pool_info: PairInfoResponse = PairQuery::PairInfo.query(
        &deps.querier,
        env.contract_code_hash.clone(),
        config.market_swap_addr.address.clone(),
    )?;
    let test_amount: u128 = 100000000;
    let mint_info: QueryAnswer = QueryMsg::Mint{
        offer_asset: config.shd_token.contract.address.clone(),
        amount: Uint128(test_amount),
    }.query(
        &deps.querier,
        env.contract_code_hash.clone(),
        config.mint_addr.address.clone(),
    )?;
    let mut mint_price: Uint128 = Uint128(0);
    match mint_info{
        QueryAnswer::Mint {
            asset,
            amount,
        } => {
            mint_price = amount;
        },
        _ => {
            mint_price = Uint128(0);
        },
    };
    let mut nom = Uint128(0);
    let mut denom = Uint128(0);
    if pool_info.pair_info.amount_0.u128().lt(&pool_info.pair_info.amount_1.u128()) {
        nom = Uint128(pool_info.pair_info.amount_1.u128().clone() * 100000000);
        denom = pool_info.pair_info.amount_0.clone();
    } else {
        nom = Uint128(pool_info.pair_info.amount_0.u128().clone() * 100000000);
        denom = pool_info.pair_info.amount_1.clone();
    }
    let mut market_price: Uint128 = div(nom, denom)?; // silk/shd
    

    let mut messages = vec![];
    if mint_price.lt(&market_price) { //swap then mint
        //take out swap fees here
        let first_swap = constant_product(amount.clone(), div(nom.clone(), Uint128(100000000)).unwrap(), denom.clone()).unwrap();
        let second_swap = div(first_swap.clone(), mint_price.clone()).unwrap();
        let mut msg = Swap{
            send: SwapOffer{
                recipient: config.market_swap_addr.address.clone(),
                amount,
                msg: to_binary(&{}).unwrap()
            }
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute{ //swap
            contract_addr: config.shd_token.contract.address.clone(),
            callback_code_hash: env.contract_code_hash.clone(),
            msg: to_binary(&msg).unwrap(),
            send: vec![],
        }));
        //let expected = {
        //    expected_amount: second_swap.clone(),
        //};
        let msg = Receive{
            amount: first_swap.clone(),
            from: config.silk_token.contract.address.clone(),
            memo: Some(to_binary("").unwrap()),
            sender:  env.contract.address.clone(),
            msg: Some(to_binary(&"TODO".to_string()).unwrap()),
        };
        let data = to_binary(&msg).unwrap();
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
}

pub fn try_execute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let config: Config = config_r(&deps.storage).load()?;

    //if amount.gt(env.)
    
    let res = trade_profitability( deps, amount ).unwrap();

    let mut profitable = false;
    let mut is_mint_first = false;
    let mut pool_shd_amount = Uint128(0);
    let mut pool_silk_amount = Uint128(0);
    let mut first_swap_min_expected = Uint128(0);
    let mut second_swap_min_expected = Uint128(0);
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
        mint_msg = mint::HandleMsg::Receive{
            sender: env.contract.address.clone(),
            from: config.shd_token.contract.address.clone(),
            amount: amount.clone(),
            memo: Some(to_binary(&"".to_string()).unwrap()),
            msg: Some(to_binary(&mint::MintMsgHook{
                minimum_expected_amount: first_swap_min_expected
            }).unwrap())
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute{
            contract_addr: config.mint_addr.address.clone(),
            callback_code_hash: config.mint_addr.code_hash.clone(),
            msg: to_binary(&mint_msg).unwrap(),
            send: vec![],
        }));

        sienna_msg = Swap{
            send: SwapOffer {
                recipient: config.market_swap_addr.address.clone(),
                amount: first_swap_min_expected.clone(),
                msg: to_binary(&CallbackSwap{
                    expected_return: second_swap_min_expected.clone(),
                }).unwrap(),
            },
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute { 
            contract_addr: config.silk_token.contract.address.clone(),
            callback_code_hash: config.silk_token.contract.code_hash.clone(),
            msg: to_binary(&sienna_msg).unwrap(), 
            send: vec![] 
        }));
    }
    else {
        sienna_msg = Swap{
            send: SwapOffer { 
                recipient: config.market_swap_addr.address.clone(),
                amount, 
                msg: to_binary(&CallbackSwap{
                    expected_return: first_swap_min_expected
                }).unwrap()
            }
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute { 
            contract_addr: config.shd_token.contract.address.clone(), 
            callback_code_hash: config.shd_token.contract.code_hash.clone(), 
            msg: to_binary(&sienna_msg).unwrap(), 
            send: vec![]
        }));

        mint_msg = mint::HandleMsg::Receive { 
            sender: env.contract.address.clone(), 
            from: config.silk_token.contract.address.clone(), 
            amount: first_swap_min_expected, 
            memo: Some(to_binary(&"".to_string()).unwrap()),
            msg: Some(to_binary(&mint::MintMsgHook{
                minimum_expected_amount: second_swap_min_expected
            }).unwrap()) 
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute{
            contract_addr: config.mint_addr.address.clone(),
            callback_code_hash: config.mint_addr.code_hash.clone(),
            msg: to_binary(&mint_msg).unwrap(),
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
}

pub fn constant_product(swap_amount: Uint128, pool_buy: Uint128, pool_sell: Uint128) -> StdResult<Uint128> {
    let cp = pool_buy.u128().clone() * pool_sell.u128().clone();
    let lpb = pool_sell.u128().clone() + swap_amount.u128().clone();
    let ncp = div(Uint128(cp.clone()), Uint128(lpb.clone())).unwrap();
    let result = pool_buy.u128().clone() - ncp.u128().clone();
    Ok(Uint128(result))
}