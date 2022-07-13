use std::marker::PhantomData;

use crate::contract_interfaces::dex::sienna::{PairInfoResponse, PairQuery, TokenType};
use crate::{utils::asset::Contract, contract_interfaces::snip20::helpers::Snip20Asset};
use crate::utils::generic_response::ResponseStatus;
use crate::c_std::{Uint128, Binary, Addr, StdResult, Env, Deps, DepsMut};

use secret_storage_plus::Item;
use crate::utils::{HandleCallback, InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TokenContract{
    pub contract: Contract,
    pub decimals: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub admin: Addr,
    pub mint_addr: Contract,
    pub market_swap_addr: Contract,
    pub shd_token: TokenContract,
    pub silk_token: TokenContract,
    pub treasury: Addr,
    pub limit: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ViewingKeys(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SelfAddr(pub Addr);

#[cfg(feature = "sky-impl")]
use crate::utils::storage::plus::ItemStorage;
impl ItemStorage for Config {
    const ITEM: Item<'static, Config> = Item::new("item_config");
}
#[cfg(feature = "sky-impl")]
impl ItemStorage for ViewingKeys{
    const ITEM: Item<'static, ViewingKeys> = Item::new("item_view_keys");
}
#[cfg(feature = "sky-impl")]
impl ItemStorage for SelfAddr{
    const ITEM: Item<'static, SelfAddr> = Item::new("item_self_addr");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitMsg{
    pub admin: Option<Addr>,
    pub mint_addr: Contract,
    pub market_swap_addr: Contract,
    pub shd_token: TokenContract,
    pub silk_token: TokenContract,
    pub treasury: Addr,
    pub viewing_key: String,
    pub limit: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        config: Config,
    },
    ArbPeg {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetMarketRate {},
    IsProfitable {
        amount: Uint128,
    },
    Balance{},
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ArbPair {
    pair_address: Addr,
    dex_id: String, //sienna, scrtswap, shdswap
    token1_address: Addr,
    token1_amount: Uint128,
    token2_address: Addr,
    token2_amount: Uint128,
}

/*impl ArbPair {
    fn init(&mut self, deps: DepsMut,env: Env) -> StdResult<bool> {
        if self.dex_id.eq(&"sienna".to_string()) {
            let pool_info: PairInfoResponse = PairQuery::PairInfo.query(
                &deps.querier,
                env.contract_code_hash.clone(),
                self.pair_address.clone(),                
            )?;
            match pool_info.pair_info.pair.token_0 {
                TokenType::CustomToken { contract_addr, token_code_hash } => self.token1_address = contract_addr.clone(),
                _ => self.token1_address = Addr::unchecked("".to_string()),
            }
            match pool_info.pair_info.pair.token_1 {
                TokenType::CustomToken { contract_addr, token_code_hash } => self.token2_address = contract_addr.clone(),
                _ => self.token2_address = Addr::unchecked("".to_string()),
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
            Ok(Uint128::new(out))
        } else {
            let out = self.token2_amount.u128() - (self.token2_amount.u128() * self.token1_amount.u128())/
                (self.token1_amount.u128() + swap_amount.u128());
            Ok(Uint128::new(out))

        }

    }
}*/