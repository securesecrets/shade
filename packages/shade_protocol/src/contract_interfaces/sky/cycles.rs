use crate::{
    contract_interfaces::{
        dex::{dex::Dex, secretswap, shadeswap, sienna},
        mint::mint,
    },
    utils::asset::Contract,
};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{to_binary, Api, CosmosMsg, Extern, Querier, StdError, Storage};
use schemars::JsonSchema;
use secret_toolkit::{snip20::send_msg, utils::Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ArbPair {
    pub pair_contract: Option<Contract>,
    pub mint_info: Option<MintInfo>,
    pub token0: Contract,
    pub token1: Contract,
    pub dex: Dex,
}

impl ArbPair {
    // Returns the calculated swap result when passed an offer with respect to the dex enum option
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
                let mint_contract = self.get_mint_contract(offer.asset.clone())?;
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

    // Returns the snip20 send_msg that will execute a swap for each of the possible Dex enum
    // options
    pub fn to_cosmos_msg(
        &self,
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
                let mint_contract = self.get_mint_contract(offer.asset.clone())?;
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

    // Returns either the silk mint or the shade mint contract depending on what the input asset is
    pub fn get_mint_contract(&self, offer_contract: Contract) -> Result<Contract, StdError> {
        if offer_contract.clone() == self.mint_info.clone().unwrap().shd_token {
            Ok(self.mint_info.clone().unwrap().mint_contract_silk)
        } else if offer_contract == self.mint_info.clone().unwrap().silk_token {
            Ok(self.mint_info.clone().unwrap().mint_contract_shd)
        } else {
            Err(StdError::generic_err(
                "Must be sending either silk or shd to mint contracts",
            ))
        }
    }

    // Gatekeeper that validates the ArbPair for entry into contract storage
    pub fn validate_pair(&self) -> Result<bool, StdError> {
        match self.dex {
            Dex::Mint => {
                self.clone()
                    .mint_info
                    .expect("Mint arb pairs must include mint info");
            }
            _ => {
                self.clone()
                    .pair_contract
                    .expect("Dex pairs must include pair contract");
            }
        }
        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cycle {
    pub pair_addrs: Vec<ArbPair>,
    pub start_addr: Contract,
}

impl Cycle {
    // Gatekeeper that validates if the contract should accept the cycle into storage
    pub fn validate_cycle(&self) -> Result<bool, StdError> {
        // check if start address is in both the first arb pair and the last arb pair
        let start_addr_in_first_pair = self.start_addr == self.pair_addrs[0].token0
            || self.start_addr == self.pair_addrs[0].token1;
        let start_addr_in_last_pair = self.start_addr
            == self.pair_addrs[self.pair_addrs.len() - 1].token0
            || self.start_addr == self.pair_addrs[self.pair_addrs.len() - 1].token1;
        if !(start_addr_in_first_pair && start_addr_in_last_pair) {
            return Err(StdError::generic_err(
                "First and last pair in cycle must contain start addr",
            ));
        }
        // check to see if each arb pair has the necessary information and if there is an actual
        // path

        // initialize this for later use
        let mut hash_vec = vec![];
        let mut cur_asset = self.start_addr.clone();
        for arb_pair in self.pair_addrs.clone() {
            match arb_pair.dex {
                Dex::Mint => {
                    arb_pair
                        .mint_info
                        .expect("Mint arb pairs must include mint info");
                }
                _ => {
                    arb_pair
                        .pair_contract
                        .clone()
                        .expect("Dex pairs must include pair contract");
                    hash_vec.push(arb_pair.pair_contract.unwrap().code_hash.clone());
                }
            }
            if arb_pair.token0 == cur_asset {
                cur_asset = arb_pair.token1;
            } else if arb_pair.token1 == cur_asset {
                cur_asset = arb_pair.token0;
            } else {
                return Err(StdError::generic_err("cycle not complete"));
            }
        }
        let initial_len = hash_vec.clone().len();
        // Sorting and dedup ing will remove any dublicates and tell us if there's 2 of the same
        // pair contract included in the cycle
        hash_vec.sort();
        hash_vec.dedup();
        if hash_vec.len() < initial_len {
            return Err(StdError::generic_err(
                "cycles should include one copy of each pair",
            ));
        }
        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Offer {
    pub asset: Contract,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MintInfo {
    pub mint_contract_shd: Contract,
    pub mint_contract_silk: Contract,
    pub shd_token: Contract,
    pub silk_token: Contract,
}
