use crate::{
    contract_interfaces::{
        dao::adapter,
        dex::{dex::Dex, secretswap, shadeswap, sienna},
        mint::mint,
    },
    utils::{asset::Contract, storage::plus::ItemStorage},
};
use cosmwasm_math_compat::{Decimal, Uint128};
use cosmwasm_std::{to_binary, Api, CosmosMsg, Extern, HumanAddr, Querier, StdError, Storage};
use schemars::JsonSchema;
use secret_storage_plus::Item;
use secret_toolkit::{
    snip20::send_msg,
    utils::{HandleCallback, InitCallback, Query},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub shade_admin: Contract,
    pub mint_contract_shd: Contract,
    pub mint_contract_silk: Contract,
    pub market_swap_contract: Contract, // Get rid eventually
    pub shd_token_contract: Contract,
    pub silk_token_contract: Contract,
    pub sscrt_token_contract: Contract,
    pub treasury: Contract,
    pub payback_rate: Decimal,
}

impl ItemStorage for Config {
    const ITEM: Item<'static, Config> = Item::new("item_config");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ViewingKeys(pub String);

impl ItemStorage for ViewingKeys {
    const ITEM: Item<'static, ViewingKeys> = Item::new("item_view_keys");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SelfAddr(pub HumanAddr);

impl ItemStorage for SelfAddr {
    const ITEM: Item<'static, SelfAddr> = Item::new("item_self_addr");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cycles(pub Vec<Cycle>);

impl ItemStorage for Cycles {
    const ITEM: Item<'static, Cycles> = Item::new("item_cycles");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Minted(pub Uint128, pub Uint128); //0: shd, 1: silk

impl ItemStorage for Minted {
    const ITEM: Item<'static, Minted> = Item::new("item_minted");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub shade_admin: Contract,
    pub mint_contract_shd: Contract,
    pub mint_contract_silk: Contract,
    pub market_swap_contract: Contract,
    pub shd_token_contract: Contract,
    pub silk_token_contract: Contract,
    pub sscrt_token_contract: Contract,
    pub treasury: Contract,
    pub viewing_key: String,
    pub payback_rate: Decimal,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        shade_admin: Option<Contract>,
        mint_contract_silk: Option<Contract>,
        mint_contract_shd: Option<Contract>,
        market_swap_contract: Option<Contract>,
        shd_token_contract: Option<Contract>,
        silk_token_contract: Option<Contract>,
        sscrt_token_contract: Option<Contract>,
        treasury: Option<Contract>,
        payback_rate: Option<Decimal>,
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
    ResetMinted {
        shd: Option<Uint128>,
        silk: Option<Uint128>,
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
    IsAnyCycleProfitable { amount: Uint128 },
    GetMinted {},
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
    GetMinted {
        shd: Uint128,
        silk: Uint128,
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
    MintedReset {
        status: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArbPair {
    pub pair_contract: Option<Contract>,
    pub token0_contract: Contract,
    pub token1_contract: Contract,
    pub dex: Dex,
}

impl ArbPair {
    pub fn simulate_swap<S: Storage, A: Api, Q: Querier>(
        self,
        deps: &Extern<S, A, Q>,
        offer: Offer,
    ) -> Result<Uint128, StdError> {
        let mut swap_result = Uint128::zero();
        match self.dex {
            Dex::SecretSwap => {
                let res = secretswap::PairQuery::Simulation {
                    offer_asset: secretswap::Asset {
                        amount: offer.amount,
                        info: secretswap::AssetInfo {
                            token: secretswap::Token {
                                contract_addr: offer.asset.address,
                                token_code_hash: offer.asset.code_hash,
                                viewing_key: "".to_string(), //TODO will sky have to make viewing keys for every asset?
                            },
                        },
                    },
                }
                .query(
                    &deps.querier,
                    self.pair_contract.clone().unwrap().code_hash,
                    self.pair_contract.clone().unwrap().address,
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
                            token_code_hash: offer.asset.code_hash.clone(),
                            contract_addr: offer.asset.address.clone(),
                        },
                        amount: offer.amount,
                    },
                }
                .query(
                    &deps.querier,
                    self.pair_contract.clone().unwrap().code_hash,
                    self.pair_contract.clone().unwrap().address,
                )?;
                match res {
                    sienna::SimulationResponse { return_amount, .. } => swap_result = return_amount,
                }
            }
            Dex::ShadeSwap => {
                let res = shadeswap::PairQuery::GetEstimatedPrice {
                    offer: shadeswap::TokenAmount {
                        token: shadeswap::TokenType::CustomToken {
                            token_code_hash: offer.asset.code_hash.clone(),
                            contract_addr: offer.asset.address.clone(),
                        },
                        amount: offer.amount,
                    },
                }
                .query(
                    &deps.querier,
                    self.pair_contract.clone().unwrap().code_hash,
                    self.pair_contract.clone().unwrap().address,
                )?;
                match res {
                    shadeswap::QueryMsgResponse::EstimatedPrice { estimated_price } => {
                        swap_result = estimated_price
                    }
                    _ => {}
                }
            }
            Dex::Mint => {
                let mint_contract = get_mint_contract(deps, offer.asset.clone())?;
                let res = mint::QueryMsg::Mint {
                    offer_asset: offer.asset.address,
                    amount: offer.amount,
                }
                .query(
                    &deps.querier,
                    mint_contract.code_hash,
                    mint_contract.address,
                )?;
                match res {
                    mint::QueryAnswer::Mint { amount, .. } => swap_result = amount,
                    _ => {}
                }
            }
        }
        Ok(swap_result)
    }

    pub fn to_cosmos_msg<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
        offer: Offer,
        expected_return: Uint128,
    ) -> Result<CosmosMsg, StdError> {
        match self.dex {
            Dex::SiennaSwap => send_msg(
                self.pair_contract.clone().unwrap().address.clone(),
                cosmwasm_std::Uint128(offer.amount.u128()),
                Some(to_binary(&sienna::CallbackMsg {
                    swap: sienna::CallbackSwap { expected_return },
                })?),
                None,
                None,
                1,
                offer.asset.code_hash,
                offer.asset.address,
            ),
            Dex::SecretSwap => send_msg(
                self.pair_contract.clone().unwrap().address.clone(),
                cosmwasm_std::Uint128(offer.amount.u128()),
                Some(to_binary(&secretswap::CallbackMsg {
                    swap: secretswap::CallbackSwap { expected_return },
                })?),
                None,
                None,
                1,
                offer.asset.code_hash,
                offer.asset.address,
            ),
            Dex::ShadeSwap => send_msg(
                self.pair_contract.clone().unwrap().address.clone(),
                cosmwasm_std::Uint128(offer.amount.u128()),
                Some(to_binary(&shadeswap::SwapTokens {
                    expected_return: Some(expected_return),
                    to: None,
                    router_link: None,
                    callback_signature: None,
                })?),
                None,
                None,
                1,
                offer.asset.code_hash,
                offer.asset.address,
            ),
            Dex::Mint => {
                let mint_contract = get_mint_contract(deps, offer.asset.clone())?;
                send_msg(
                    mint_contract.address.clone(),
                    cosmwasm_std::Uint128(offer.amount.u128()),
                    Some(to_binary(&mint::MintMsgHook {
                        minimum_expected_amount: expected_return,
                    })?),
                    None,
                    None,
                    1,
                    offer.asset.code_hash,
                    offer.asset.address,
                )
            }
        }
    }
}

pub fn get_mint_contract<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    offer_contract: Contract,
) -> Result<Contract, StdError> {
    let config = Config::load(&deps.storage)?;
    if offer_contract.clone() == config.shd_token_contract {
        Ok(config.mint_contract_silk)
    } else if offer_contract.clone() == config.silk_token_contract {
        Ok(config.mint_contract_shd)
    } else {
        Err(StdError::generic_err(
            "Must be sending either silk or shd to mint contracts",
        ))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cycle {
    pub pair_addrs: Vec<ArbPair>,
    pub start_addr: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Offer {
    pub asset: Contract,
    pub amount: Uint128,
}
