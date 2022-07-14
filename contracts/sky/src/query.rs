use shade_protocol::c_std::{
    Storage, Api, Querier, DepsMut, StdResult, StdError, debug_print,
};
use shade_protocol::c_std::Uint128;
use shade_protocol::{
    contract_interfaces::{
        sky::sky::{QueryAnswer, Config, ViewingKeys, SelfAddr},
        mint::mint::{QueryMsg, self},
        dex::{dex::pool_take_amount, sienna::{PairInfoResponse, PairQuery, TokenType, PairInfo},},
    snip20,
    },
    utils::storage::plus::ItemStorage,
};

pub fn config(
    deps: Deps
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(&deps.storage)?,
    })
}

pub fn market_rate(
    deps: Deps
) -> StdResult<QueryAnswer> {
    let config: Config = Config::load(&deps.storage)?;

    //Query mint contract
    let mint_info: mint::QueryAnswer = QueryMsg::Mint{
        offer_asset: config.shd_token.contract.address.clone(),
        amount: Uint128::new(100000000), //1 SHD
    }.query(
        &deps.querier,
        config.mint_addr.code_hash.clone(),
        config.mint_addr.address.clone(),
    )?;
    let mut mint_price: Uint128 = Uint128::new(0); // SILK/SHD
    match mint_info{
        mint::QueryAnswer::Mint {
            asset: _,
            amount,
        } => {
            mint_price = amount.checked_mul(Uint128::new(100))?; // times 100 to make it have 8 decimals
        },
        _ => {
            mint_price = Uint128::new(0);
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

pub fn trade_profitability(
    deps: Deps,
    amount: Uint128,
) -> StdResult<QueryAnswer> {
    let config: Config = Config::load(&deps.storage)?;

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

    let mut shd_amount: Uint128 = Uint128::new(1);
    let mut silk_amount: Uint128 = Uint128::new(1);
    let mut silk_8d: Uint128 = Uint128::new(1);

    match pool_info.pair_info.pair.token_0{
        TokenType::CustomToken {
            contract_addr,
            token_code_hash: _,
        } => {
            if contract_addr.eq(&config.shd_token.contract.address) {
                shd_amount = pool_info.pair_info.amount_0;
                silk_amount = pool_info.pair_info.amount_1;
                silk_8d = silk_amount.checked_mul(Uint128::new(100))?;
            } else {
                shd_amount = pool_info.pair_info.amount_1;
                silk_amount = pool_info.pair_info.amount_0;
                silk_8d = silk_amount.checked_mul(Uint128::new(100))?;
            }
        }
        _ => {
            ;
        }
    }

    let div_silk_8d: Uint128 = silk_8d.checked_mul(Uint128::new(100000000))?;
    let dex_price: Uint128 = div_silk_8d.checked_div(shd_amount.clone())?;    


    let mut first_swap_amount: Uint128 = Uint128::new(0);
    let mut second_swap_amount: Uint128 = Uint128::new(0);
    let mut mint_first: bool = false;

    if mint_price.gt(&dex_price) {
        mint_first = true;
        let mul_mint_price: Uint128 = mint_price.checked_mul(amount)?;
        first_swap_amount = mul_mint_price.checked_div(Uint128::new(100000000))?;
        let mut first_swap_less_fee = first_swap_amount.checked_div(Uint128::new(325))?;
        first_swap_less_fee = first_swap_amount.checked_sub(first_swap_less_fee)?;
        second_swap_amount = pool_take_amount(
            amount, 
            silk_8d, 
            shd_amount,
        );
    } else {
        mint_first = false;
        let mut amount_less_fee: Uint128 = amount.checked_div(Uint128::new(325))?;
        amount_less_fee = amount.checked_sub(amount_less_fee)?;
        first_swap_amount = pool_take_amount(
            amount_less_fee, 
            shd_amount,
            silk_8d,
        );
        let mul_first_swap = first_swap_amount.checked_mul(Uint128::new(100000000))?;
        second_swap_amount = mul_first_swap.checked_div(mint_price)?;
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

pub fn get_balances(
    deps: Deps
) -> StdResult<QueryAnswer> {

    let viewing_key = ViewingKeys::load(&deps.storage)?.0;
    let self_addr = SelfAddr::load(&deps.storage)?.0;
    let config = Config::load(&deps.storage)?;
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

    let mut shd_bal = Uint128::new(0);

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

    let mut silk_bal = Uint128::new(0);

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