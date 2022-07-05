use crate::{
    contract_interfaces::{
        dao::adapter,
        dex::{dex::Dex, secretswap, shadeswap, sienna},
    },
    utils::asset::Contract,
};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{to_binary, Api, CosmosMsg, Extern, HumanAddr, Querier, StdError, Storage};
use schemars::JsonSchema;
use secret_storage_plus::Item;
use secret_toolkit::{
    snip20::send_msg,
    utils::{HandleCallback, InitCallback, Query},
};
use serde::{Deserialize, Serialize};

/*#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenContract {
    pub contract: Contract,
    pub decimals: Uint128,
}*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: HumanAddr,
    pub mint_contract_shd: Contract,
    pub mint_contract_silk: Contract,
    pub market_swap_contract: Contract,
    pub shd_token_contract: Contract,
    pub silk_token_contract: Contract,
    pub treasury: HumanAddr,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ViewingKeys(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SelfAddr(pub HumanAddr);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cycles(pub Vec<Cycle>);

#[cfg(feature = "sky-impl")]
use crate::utils::storage::plus::ItemStorage;
impl ItemStorage for Config {
    const ITEM: Item<'static, Config> = Item::new("item_config");
}
#[cfg(feature = "sky-impl")]
impl ItemStorage for ViewingKeys {
    const ITEM: Item<'static, ViewingKeys> = Item::new("item_view_keys");
}
#[cfg(feature = "sky-impl")]
impl ItemStorage for SelfAddr {
    const ITEM: Item<'static, SelfAddr> = Item::new("item_self_addr");
}
#[cfg(feature = "sky-impl")]
impl ItemStorage for Cycles {
    const ITEM: Item<'static, Cycles> = Item::new("item_cycles");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admin: Option<HumanAddr>, //TODO shade admins contract
    pub mint_contract_shd: Contract,
    pub mint_contract_silk: Contract,
    pub market_swap_contract: Contract,
    pub shd_token_contract: Contract,
    pub silk_token_contract: Contract,
    pub treasury: HumanAddr,
    pub viewing_key: String,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        config: Config,
        padding: Option<String>,
    },
    ArbPeg {
        amount: Uint128,
        padding: Option<String>,
    },
    SetCycles {
        cycles: Vec<Cycle>,
        padding: Option<String>,
    },
    AppendCycles {
        cycle: Vec<Cycle>,
        padding: Option<String>,
    },
    RemoveCycle {
        index: Uint128,
        padding: Option<String>,
    },
    ArbCycle {
        amount: Uint128,
        index: Uint128,
        padding: Option<String>,
    },
    Adapter(adapter::SubHandleMsg),
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    IsArbPegProfitable { amount: Uint128 },
    Balance {},
    GetCycles {},
    IsCycleProfitable { amount: Uint128, index: Uint128 },
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    ArbPegProfitability {
        is_profitable: bool,
        mint_first: bool,
        first_swap_result: Uint128,
        profit: Uint128,
    },
    Balance {
        shd_bal: Uint128,
        silk_bal: Uint128, //should be zero or close to
    },
    GetCycles {
        cycles: Vec<Cycle>,
    },
    IsCycleProfitable {
        is_profitable: bool,
        direction: Cycle,
        swap_amounts: Vec<Uint128>,
        profit: Uint128,
    },
    IsAnyCycleProfitable {
        is_profitable: Vec<bool>,
        direction: Vec<Cycle>,
        swap_amounts: Vec<Vec<Uint128>>,
        profit: Vec<Uint128>,
    },
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
        amount: Uint128,
        after_first_swap: Uint128,
        final_amount: Uint128,
    },
    SetCycles {
        status: bool,
    },
    AppendCycles {
        status: bool,
    },
    RemoveCycle {
        status: bool,
    },
    ExecuteArbCycle {
        status: bool,
        swap_amounts: Vec<Uint128>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArbPair {
    pub pair_contract: Contract,
    pub token0_contract: Contract,
    pub token1_contract: Contract,
    pub dex: Dex,
}

impl ArbPair {
    pub fn simulate_swap<S: Storage, A: Api, Q: Querier>(
        self,
        deps: &Extern<S, A, Q>,
        amount: Uint128,
        offer_token: Contract,
    ) -> Result<Uint128, StdError> {
        let mut swap_result = Uint128::zero();
        match self.dex {
            Dex::SecretSwap => {
                let res = secretswap::PairQuery::Simulation {
                    offer_asset: secretswap::Asset {
                        amount,
                        info: secretswap::AssetInfo {
                            token: secretswap::Token {
                                contract_addr: offer_token.address,
                                token_code_hash: offer_token.code_hash,
                                viewing_key: "".to_string(), //TODO will sky have to make viewing keys for every asset?
                            },
                        },
                    },
                }
                .query(
                    &deps.querier,
                    self.pair_contract.code_hash,
                    self.pair_contract.address,
                )?;
                match res {
                    secretswap::SimulationResponse { return_amount, .. } => {
                        swap_result = return_amount
                    }
                }
            }
            Dex::SiennaSwap => {
                let res = sienna::PairQuery::SwapSimulation {
                    offer: sienna::TokenTypeAmount {
                        token: sienna::TokenType::CustomToken {
                            token_code_hash: offer_token.code_hash.clone(),
                            contract_addr: offer_token.address.clone(),
                        },
                        amount,
                    },
                }
                .query(
                    &deps.querier,
                    self.pair_contract.code_hash,
                    self.pair_contract.address,
                )?;
                match res {
                    sienna::SimulationResponse { return_amount, .. } => swap_result = return_amount,
                }
            }
            Dex::ShadeSwap => {
                let res = shadeswap::PairQuery::GetEstimatedPrice {
                    offer: shadeswap::TokenAmount {
                        token: shadeswap::TokenType::CustomToken {
                            token_code_hash: offer_token.code_hash.clone(),
                            contract_addr: offer_token.address.clone(),
                        },
                        amount,
                    },
                }
                .query(
                    &deps.querier,
                    self.pair_contract.code_hash,
                    self.pair_contract.address,
                )?;
                match res {
                    shadeswap::QueryMsgResponse::EstimatedPrice { estimated_price } => {
                        swap_result = estimated_price
                    }
                    _ => {}
                }
            }
        }
        Ok(swap_result)
    }

    pub fn to_cosmos_msg(
        &self,
        recipient: HumanAddr,
        amount: Uint128,
        expected_return: Uint128,
        offer_asset: Contract,
    ) -> Result<CosmosMsg, StdError> {
        match self.dex {
            Dex::SiennaSwap => send_msg(
                recipient,
                cosmwasm_std::Uint128(amount.u128()),
                Some(to_binary(&sienna::CallbackMsg {
                    swap: sienna::CallbackSwap { expected_return },
                })?),
                None,
                None,
                1,
                offer_asset.code_hash,
                offer_asset.address,
            ),
            Dex::SecretSwap => send_msg(
                recipient,
                cosmwasm_std::Uint128(amount.u128()),
                Some(to_binary(&secretswap::CallbackMsg {
                    swap: secretswap::CallbackSwap { expected_return },
                })?),
                None,
                None,
                1,
                offer_asset.code_hash,
                offer_asset.address,
            ),
            Dex::ShadeSwap => send_msg(
                recipient,
                cosmwasm_std::Uint128(amount.u128()),
                Some(to_binary(&shadeswap::SwapTokens {
                    expected_return: Some(expected_return),
                    to: None,
                    router_link: None,
                    callback_signature: None,
                })?),
                None,
                None,
                1,
                offer_asset.code_hash,
                offer_asset.address,
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cycle {
    pub pair_addrs: Vec<ArbPair>,
    pub start_addr: HumanAddr,
}
