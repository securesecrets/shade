use cosmwasm_std::{
    Storage, Api, Querier, Extern, StdResult, Uint128, StdError, debug_print,
};
use secret_toolkit::utils::Query;
use crate::state::{config_r, viewing_key_r, self_address_r};
use shade_protocol::contract_interfaces::{
    sky::{QueryAnswer, Config},
    mint::{QueryMsg, self},
    sienna::{PairInfoResponse, PairQuery, TokenType, PairInfo},
    utils::{math::{div, mult}},
    dex::pool_take_amount,
    snip20,
};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn market_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<QueryAnswer> {
    let config: Config = config_r(&deps.storage).load()?;

    //Query mint contract
    let mint_info: mint::QueryAnswer = QueryMsg::Mint{
        offer_asset: config.shd_token.contract.address.clone(),
        amount: Uint128(100000000), //1 SHD
    }.query(
        &deps.querier,
        config.mint_addr.code_hash.clone(),
        config.mint_addr.address.clone(),
    )?;
    let mut mint_price: Uint128 = Uint128(0); // SILK/SHD
    match mint_info{
        mint::QueryAnswer::Mint {
            asset: _,
            amount,
        } => {
            mint_price = mult(amount, Uint128(100)); // times 100 to make it have 8 decimals
        },
        _ => {
            mint_price = Uint128(0);
        },
    };

    //TODO Query Pool Amount
    let pool_info: PairInfoResponse = PairQuery::PairInfo.query(
        &deps.querier,
        config.market_swap_addr.code_hash.clone(),
        config.market_swap_addr.address.clone(),
    )?;

    Ok(QueryAnswer::GetMarketRate { 
        mint_rate: mint_price,
        pair: pool_info,
    })
}

pub fn trade_profitability<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint128,
) -> StdResult<QueryAnswer> {
    let config: Config = config_r(&deps.storage).load()?;

    let market_query = market_rate(&deps)?;
    let mint_price: Uint128;
    let pool_info: PairInfoResponse;
    
    match market_query {
        QueryAnswer::GetMarketRate { 
            mint_rate, 
            pair
         } => {
            mint_price = mint_rate;
            pool_info = pair;
         },
         _ => {
            return Err(StdError::generic_err("failed."));
        }
    };

    let mut shd_amount: Uint128 = Uint128(1);
    let mut silk_amount: Uint128 = Uint128(1);
    let mut silk_8d: Uint128 = Uint128(1);

    match pool_info.pair_info.pair.token_0{
        TokenType::CustomToken {
            contract_addr,
            token_code_hash: _,
        } => {
            if contract_addr.eq(&config.shd_token.contract.address) {
                shd_amount = pool_info.pair_info.amount_0;
                silk_amount = pool_info.pair_info.amount_1;
                silk_8d = mult(silk_amount, Uint128(100));
            } else {
                shd_amount = pool_info.pair_info.amount_1;
                silk_amount = pool_info.pair_info.amount_0;
                silk_8d = mult(silk_amount, Uint128(100));
            }
        }
        _ => {
            ;
        }
    }

    let dex_price: Uint128 = div(
        mult(silk_8d.clone(),Uint128(100000000)),
        shd_amount.clone(),
    ).unwrap();    


    let mut first_swap_amount: Uint128 = Uint128(0);
    let mut second_swap_amount: Uint128 = Uint128(0);
    let mut mint_first: bool = false;

    if mint_price.gt(&dex_price) {
        mint_first = true;
        first_swap_amount = div(
            mult(mint_price, amount),
            Uint128(100000000),
        ).unwrap();
        let mut first_swap_less_fee = div(
            first_swap_amount.clone(),
            Uint128(325)
        ).unwrap();
        first_swap_less_fee = Uint128(first_swap_amount.u128() - first_swap_less_fee.u128());
        second_swap_amount = pool_take_amount(
            amount, 
            silk_8d, 
            shd_amount,
        );
    } else {
        mint_first = false;
        let mut amount_less_fee: Uint128 = div(
            amount.clone(),
            Uint128(325)
        ).unwrap();
        amount_less_fee = Uint128(amount.u128() - amount_less_fee.u128());
        first_swap_amount = pool_take_amount(
            amount_less_fee, 
            shd_amount,
            silk_8d,
        );
        second_swap_amount = div(
            mult(first_swap_amount, Uint128(100000000)),
            mint_price
        ).unwrap();
    }

    let is_profitable = second_swap_amount.gt(&amount);

    Ok(QueryAnswer::TestProfitability { 
        is_profitable, 
        mint_first, 
        shd_amount,
        silk_amount,
        first_swap_amount, 
        second_swap_amount, 
    })
}

pub fn get_balances<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<QueryAnswer> {

    let viewing_key = viewing_key_r(&deps.storage).load()?;
    let self_addr = self_address_r(&deps.storage).load()?;
    let config = config_r(&deps.storage).load()?;
    let mut is_error = false;

    let mut res = snip20::QueryMsg::Balance {
        address: self_addr.clone(),
        key: viewing_key.clone()
    }.query(
        &deps.querier,
        config.shd_token.contract.code_hash.clone(),
        config.shd_token.contract.address.clone(),
    )?;

    debug_print!("{}", viewing_key);

    let mut shd_bal = Uint128(0);

    match res {
        snip20::QueryAnswer::Balance {amount } => {
            shd_bal = amount.clone();
        }, 
        _ => is_error = true,
    }

    res = snip20::QueryMsg::Balance {
        address: self_addr.clone(),
        key: viewing_key.clone(),
    }.query(
        &deps.querier,
        config.silk_token.contract.code_hash.clone(),
        config.silk_token.contract.address.clone()
    )?;

    let mut silk_bal = Uint128(0);

    match res {
        snip20::QueryAnswer::Balance { amount  } => {
            silk_bal = amount;
        },
        _ => is_error = true,
    }

    Ok(QueryAnswer::Balance {
        error_status: is_error.clone(),
        shd_bal,
        silk_bal
    })
}