use cosmwasm_std::{Api, Extern, HumanAddr, Querier, QueryResult, StdResult, Storage, to_binary};
use cosmwasm_math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20_test::manager::{Allowance, Balance, CoinInfo, Config, ContractStatusLevel, Minters, TotalSupply};
use shade_protocol::contract_interfaces::snip20_test::QueryAnswer;
use shade_protocol::contract_interfaces::snip20_test::transaction_history::{get_transfers, get_txs};
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};

pub fn token_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> QueryResult {

    let info = CoinInfo::load(&deps.storage)?;

    let total_supply = match Config::public_total_supply(&deps.storage)? {
        true => Some(TotalSupply::load(&deps.storage)?.0),
        false => None
    };

    to_binary(&QueryAnswer::TokenInfo {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply
    })
}

pub fn token_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> QueryResult {
    to_binary(&QueryAnswer::TokenConfig {
        // TODO: show the other addrd config items
        public_total_supply: Config::public_total_supply(&deps.storage)?,
        deposit_enabled: Config::deposit_enabled(&deps.storage)?,
        redeem_enabled: Config::redeem_enabled(&deps.storage)?,
        mint_enabled: Config::mint_enabled(&deps.storage)?,
        burn_enabled: Config::burn_enabled(&deps.storage)?
    })
}

pub fn contract_status<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> QueryResult {
    to_binary(&QueryAnswer::ContractStatus {
        status: ContractStatusLevel::load(&deps.storage)?
    })
}

pub fn exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> QueryResult {
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
        return to_binary(&QueryAnswer::ExchangeRate { rate, denom });
    }
    to_binary(&QueryAnswer::ExchangeRate {
        rate: Uint128::new(0),
        denom: String::new(),
    })
}

pub fn minters<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> QueryResult {
    to_binary(&QueryAnswer::Minters {
        minters: Minters::load(&deps.storage)?.0
    })
}

pub fn allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    spender: HumanAddr
) -> QueryResult {
    let allowance = Allowance::load(&deps.storage, (owner.clone(), spender.clone()))?;

    to_binary(&QueryAnswer::Allowance {
        spender,
        owner,
        allowance: allowance.amount,
        expiration: allowance.expiration
    })
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, account: HumanAddr) -> QueryResult {
    to_binary(&QueryAnswer::Balance {
        amount: Balance::load(&deps.storage, account)?.0
    })
}

pub fn transfer_history<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: HumanAddr,
    page: u32,
    page_size: u32,
) -> QueryResult {
    let transfer = get_transfers(&deps.storage, &account, page, page_size)?;
    to_binary(&QueryAnswer::TransferHistory {
        txs: transfer.0,
        total: Some(transfer.1)
    })
}

pub fn transaction_history<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: HumanAddr,
    page: u32,
    page_size: u32,
) -> QueryResult {
    let transfer = get_txs(&deps.storage, &account, page, page_size)?;
    to_binary(&QueryAnswer::TransactionHistory {
        txs: transfer.0,
        total: Some(transfer.1)
    })
}