use crate::contract_interfaces::dex::sienna::{PairInfoResponse, PairQuery, TokenType};
use crate::{utils::asset::Contract, contract_interfaces::snip20::Snip20Asset};
use crate::utils::generic_response::ResponseStatus;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{Binary, HumanAddr, StdResult, Env, Extern, Querier, Api, Storage};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[cfg(feature = "sky-impl")]
use crate::utils::storage::SingletonStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub mint_addr: Contract,
    pub market_swap_addr: Contract,
    pub shd_token: Snip20Asset,
    pub silk_token: Snip20Asset,
    pub treasury: HumanAddr,
    pub limit: Option<String>,
}

#[cfg(feature = "sky-impl")]
impl SingletonStorage for Config {
    const NAMESPACE: &'static [u8] = b"config-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg{
    pub admin: Option<HumanAddr>,
    pub mint_addr: Contract,
    pub market_swap_addr: Contract,
    pub shd_token: Snip20Asset,
    pub silk_token: Snip20Asset,
    pub treasury: HumanAddr,
    pub viewing_key: String,
    pub limit: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        config: Config,
    },
    ArbPeg {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetMarketRate {},
    IsProfitable {
        amount: Uint128,
    },
    Balance{},
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    GetMarketRate {
        mint_rate: Uint128,
        pair: PairInfoResponse,
    },
    TestProfitability {
        is_profitable: bool,
        mint_first: bool,
        shd_amount: Uint128,
        silk_amount: Uint128,
        first_swap_amount: Uint128,
        second_swap_amount: Uint128,
    },
    Balance{
        error_status: bool,
        shd_bal: Uint128,
        silk_bal: Uint128,
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init {
        status: bool,
    },
    UpdateConfig {
        status: bool,
    },
    ExecuteArb {
        status: bool,
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArbPair {
    pair_address: HumanAddr,
    dex_id: String, //sienna, scrtswap, shdswap
    token1_address: HumanAddr,
    token1_amount: Uint128,
    token2_address: HumanAddr,
    token2_amount: Uint128,
}

/*impl ArbPair {
    fn init<S: Storage, A: Api, Q: Querier>(&mut self, deps: &mut Extern<S, A, Q>,env: Env) -> StdResult<bool> {
        if self.dex_id.eq(&"sienna".to_string()) {
            let pool_info: PairInfoResponse = PairQuery::PairInfo.query(
                &deps.querier,
                env.contract_code_hash.clone(),
                self.pair_address.clone(),                
            )?;
            match pool_info.pair_info.pair.token_0 {
                TokenType::CustomToken { contract_addr, token_code_hash } => self.token1_address = contract_addr.clone(),
                _ => self.token1_address = HumanAddr("".to_string()),
            }
            match pool_info.pair_info.pair.token_1 {
                TokenType::CustomToken { contract_addr, token_code_hash } => self.token2_address = contract_addr.clone(),
                _ => self.token2_address = HumanAddr("".to_string()),
            }
            self.token1_amount = pool_info.pair_info.amount_0.clone();
            self.token2_amount = pool_info.pair_info.amount_1.clone();
        } else if self.dex_id.eq(&"sswap".to_string()) {
            todo!() 
        } else { //shd swap
            todo!()
        }  

        Ok(true)
    }
    fn expected_amount(&self, swap_amount: Uint128, buy_token1: bool) -> StdResult<Uint128>{
        if buy_token1 {
            let out = self.token1_amount.u128() - (self.token1_amount.u128() * self.token2_amount.u128())/
                (self.token2_amount.u128() + swap_amount.u128());
            Ok(Uint128(out))
        } else {
            let out = self.token2_amount.u128() - (self.token2_amount.u128() * self.token1_amount.u128())/
                (self.token1_amount.u128() + swap_amount.u128());
            Ok(Uint128(out))

        }

    }
}*/