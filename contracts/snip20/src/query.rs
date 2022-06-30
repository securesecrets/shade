use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{to_binary, Api, Extern, HumanAddr, Querier, QueryResult, StdResult, Storage};
use shade_protocol::{
    contract_interfaces::snip20::{
        manager::{
            Allowance,
            Balance,
            CoinInfo,
            Config,
            ContractStatusLevel,
            Minters,
            TotalSupply,
        },
        transaction_history::{RichTx, Tx},
        QueryAnswer,
    },
    utils::storage::plus::{ItemStorage, MapStorage},
};

pub fn token_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let info = CoinInfo::load(&deps.storage)?;

    let total_supply = match Config::public_total_supply(&deps.storage)? {
        true => Some(TotalSupply::load(&deps.storage)?.0),
        false => None,
    };

    Ok(QueryAnswer::TokenInfo {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply,
    })
}

pub fn token_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::TokenConfig {
        // TODO: show the other addrd config items
        public_total_supply: Config::public_total_supply(&deps.storage)?,
        deposit_enabled: Config::deposit_enabled(&deps.storage)?,
        redeem_enabled: Config::redeem_enabled(&deps.storage)?,
        mint_enabled: Config::mint_enabled(&deps.storage)?,
        burn_enabled: Config::burn_enabled(&deps.storage)?,
        transfer_enabled: Config::transfer_enabled(&deps.storage)?,
    })
}

pub fn contract_status<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::ContractStatus {
        status: ContractStatusLevel::load(&deps.storage)?,
    })
}

pub fn exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let decimals = CoinInfo::load(&deps.storage)?.decimals;
    if Config::deposit_enabled(&deps.storage)? || Config::redeem_enabled(&deps.storage)? {
        let rate: Uint128;
        let denom: String;
        // if token has more decimals than SCRT, you get magnitudes of SCRT per token
        if decimals >= 6 {
            rate = Uint128::new(10u128.pow(decimals as u32 - 6));
            denom = "SCRT".to_string();
            // if token has less decimals, you get magnitudes token for SCRT
        } else {
            rate = Uint128::new(10u128.pow(6 - decimals as u32));
            denom = CoinInfo::load(&deps.storage)?.symbol;
        }
        return Ok(QueryAnswer::ExchangeRate { rate, denom });
    }
    Ok(QueryAnswer::ExchangeRate {
        rate: Uint128::new(0),
        denom: String::new(),
    })
}

pub fn minters<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Minters {
        minters: Minters::load(&deps.storage)?.0,
    })
}

pub fn allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    spender: HumanAddr,
) -> StdResult<QueryAnswer> {
    let allowance = Allowance::may_load(
        &deps.storage,
        (owner.clone(), spender.clone())
    )?.unwrap_or_default();

    Ok(QueryAnswer::Allowance {
        spender,
        owner,
        allowance: allowance.amount,
        expiration: allowance.expiration,
    })
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: HumanAddr,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Balance {
        amount: Balance::may_load(&deps.storage, account)?.unwrap_or(Balance(Uint128::zero())).0,
    })
}

pub fn transfer_history<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: HumanAddr,
    page: u32,
    page_size: u32,
) -> StdResult<QueryAnswer> {
    let transfer = Tx::get(&deps.storage, &account, page, page_size)?;
    Ok(QueryAnswer::TransferHistory {
        txs: transfer.0,
        total: Some(transfer.1),
    })
}

pub fn transaction_history<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: HumanAddr,
    page: u32,
    page_size: u32,
) -> StdResult<QueryAnswer> {
    let transfer = RichTx::get(&deps.storage, &account, page, page_size)?;
    Ok(QueryAnswer::TransactionHistory {
        txs: transfer.0,
        total: Some(transfer.1),
    })
}
