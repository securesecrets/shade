use crate::{
    core::{ContractInstantiationInfo, TokenType},
    BLOCK_SIZE,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint128, Uint256};
use shade_protocol::{
    query_auth::QueryPermit,
    snip20::Snip20ReceiveMsg,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, Query},
    Contract,
};
#[cfg(feature = "staking")]
pub use state::*;

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub struct StakingContractInstantiateInfo {
    pub staking_contract_info: ContractInstantiationInfo,
    pub custom_label: Option<String>,
    pub first_reward_token: Option<RewardTokenCreate>,
    pub query_auth: Option<RawContract>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub amm_pair: String,
    pub lp_token: RawContract,
    pub admin_auth: RawContract,
    pub query_auth: Option<RawContract>,
    pub first_reward_token: Option<RewardTokenCreate>,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimRewards {
        padding: Option<String>,
    },
    Unstake {
        amount: Uint128,
        remove_liquidity: Option<bool>,
        padding: Option<String>,
    },
    Receive(Snip20ReceiveMsg),
    UpdateRewardTokens(Vec<RewardTokenUpdate>),
    CreateRewardTokens(Vec<RewardTokenCreate>),
    UpdateConfig {
        admin_auth: Option<RawContract>,
        query_auth: Option<RawContract>,
        padding: Option<String>,
    },
    RecoverFunds {
        token: TokenType,
        amount: Uint128,
        to: String,
        msg: Option<Binary>,
        padding: Option<String>,
    },
}

#[cw_serde]
pub enum InvokeMsg {
    /// From is used to determine the staker since this can be called by the AMMPair when auto staking.
    Stake {
        from: Option<String>,
        padding: Option<String>,
    },
}

#[cw_serde]
pub struct RewardTokenUpdate {
    pub reward_token: RawContract,
    pub index: u64,
    pub valid_to: u64,
}

#[cw_serde]
pub struct RewardTokenCreate {
    pub reward_token: RawContract,
    pub daily_reward_amount: Uint128,
    pub valid_to: u64,
}

#[allow(clippy::large_enum_variant)]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(PermitQueryResponse)]
    WithPermit {
        permit: QueryPermit,
        query: AuthQuery,
    },
}

#[cw_serde]
pub struct QueryPermitData {}

#[cw_serde]
pub enum AuthQuery {
    GetStakerInfo {},
}

#[derive(PartialEq, Debug, Clone)]
pub struct ClaimRewardResponse {
    pub token: Contract,
    pub amount: Uint128,
}

// RESPONSE TYPES

#[cw_serde]
pub struct ConfigResponse {
    pub lp_token: Contract,
    pub amm_pair: Addr,
    pub admin_auth: Contract,
    pub query_auth: Option<Contract>,
    pub total_amount_staked: Uint128,
    pub reward_tokens: Vec<RewardTokenInfo>,
}

#[cw_serde]
pub enum PermitQueryResponse {
    StakerInfo {
        /// Amount normally staked.
        staked: Uint128,
        /// Staked
        total_staked: Uint128,
        claimable_rewards: Vec<ClaimableRewardsResponse>,
    },
}

#[cw_serde]
pub struct ClaimableRewardsResponse {
    pub token: Contract,
    pub amount: Uint128,
}

#[cw_serde]
pub struct RewardTokenInfo {
    pub token: Contract,
    pub decimals: u8,
    pub reward_per_second: Uint256,
    pub reward_per_staked_token: Uint256,
    pub valid_to: u64,
    pub last_updated: u64,
}

impl RewardTokenUpdate {
    pub fn new(reward_token: impl Into<RawContract>, index: u64, valid_to: u64) -> Self {
        Self {
            reward_token: reward_token.into(),
            index,
            valid_to,
        }
    }
}

impl RewardTokenCreate {
    pub fn new(
        reward_token: impl Into<RawContract>,
        daily_reward_amount: Uint128,
        valid_to: u64,
    ) -> Self {
        Self {
            reward_token: reward_token.into(),
            daily_reward_amount,
            valid_to,
        }
    }
}

#[cfg(feature = "staking")]
pub mod state {
    use better_secret_math::{
        common::{bankers_round, exp10, muldiv},
        ud60x18::mul,
        U256,
    };
    use core::time;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{
        Addr, CosmosMsg, Decimal256, OverflowError, QuerierWrapper, StdError, StdResult, Storage,
        Uint128, Uint256,
    };
    use secret_storage_plus::{Bincode2, Item, ItemStorage, Map};
    use shade_protocol::{
        contract_interfaces::snip20::ExecuteMsg as Snip20ExecuteMsg, utils::ExecuteCallback,
        Contract,
    };
    use std::{cmp::min, collections::HashMap};

    use super::{
        ClaimRewardResponse, ClaimableRewardsResponse, ConfigResponse, PermitQueryResponse,
        RewardTokenInfo,
    };

    /// Manages the global state of the staking contract.
    #[cw_serde]
    pub struct Custodian {
        pub lp_token: Contract,
        pub amm_pair: Addr,
        pub admin_auth: Contract,
        pub query_auth: Option<Contract>,
        pub total_amount_staked: Uint128,
    }

    impl ItemStorage for Custodian {
        const ITEM: Item<'static, Self> = Item::new("custodian");
    }

    #[cw_serde]
    pub struct RewardTokenSet(Vec<Addr>);

    impl RewardTokenSet {
        pub fn insert(&mut self, addr: &Addr) {
            if !self.0.contains(addr) {
                self.0.push(addr.clone());
            }
        }
        pub fn get(&self) -> &[Addr] {
            self.0.as_slice()
        }
    }

    impl<'a> Custodian {
        pub const STAKERS: Map<'static, &'a Addr, u128, Bincode2> = Map::new("stakers");
        pub const REWARD_TOKEN_INFO: Map<'static, &'a Addr, Vec<RewardTokenInfo>> =
            Map::new("reward_token_info");
        pub const REWARD_TOKENS: Item<'static, RewardTokenSet> = Item::new("reward_tokens");
    }

    impl Custodian {
        pub fn require_lp_token(&self, addr: &Addr) -> StdResult<()> {
            if self.lp_token.address.eq(addr) {
                return Ok(());
            }
            Err(StdError::generic_err(format!(
                "Must stake the LP token {}. Attempted to stake {addr}.",
                self.lp_token.address
            )))
        }
        pub fn save_staker(storage: &mut dyn Storage, staker: &Staker) -> StdResult<()> {
            staker.save_rewards(storage)?;
            Self::STAKERS.save(storage, &staker.addr, &staker.staked.u128())
        }
    }

    impl Custodian {
        pub fn store_empty_reward_set(&self, storage: &mut dyn Storage) -> StdResult<()> {
            match Self::REWARD_TOKENS.may_load(storage)? {
                Some(_) => Err(StdError::generic_err("Reward token storage already exists")),
                None => Self::REWARD_TOKENS.save(storage, &RewardTokenSet(vec![])),
            }
        }

        pub fn update_reward_token(
            &self,
            now: u64,
            storage: &mut dyn Storage,
            token: &Contract,
            index: u64,
            valid_to: u64,
        ) -> StdResult<Vec<RewardTokenInfo>> {
            if valid_to < now {
                return Err(StdError::generic_err("valid_to cannot be in the past"));
            }
            let mut reward_configs = Self::REWARD_TOKEN_INFO.load(storage, &token.address)?;
            match reward_configs.get_mut(index as usize) {
                Some(info) => {
                    info.valid_to = valid_to;
                }
                None => return Err(StdError::generic_err("Invalid index")),
            };
            Self::REWARD_TOKEN_INFO.save(storage, &token.address, &reward_configs)?;
            Ok(reward_configs)
        }

        pub fn create_reward_token(
            &self,
            storage: &mut dyn Storage,
            now: u64,
            token: &Contract,
            daily_emission_amount: Uint128,
            valid_to: u64,
            decimals: u8,
        ) -> StdResult<Vec<RewardTokenInfo>> {
            let mut reward_configs =
                match Self::REWARD_TOKEN_INFO.may_load(storage, &token.address)? {
                    Some(rewards) => rewards,
                    None => vec![],
                };
            let info = RewardTokenInfo::init_from_daily_rewards(
                now,
                token,
                decimals,
                daily_emission_amount,
                valid_to,
            )?;
            match Self::REWARD_TOKENS.may_load(storage)? {
                Some(mut tokens) => {
                    tokens.insert(&info.token.address);
                    Self::REWARD_TOKENS.save(storage, &tokens)?;
                }
                None => Self::REWARD_TOKENS
                    .save(storage, &RewardTokenSet(vec![info.token.address.clone()]))?,
            };
            reward_configs.push(info);
            Self::REWARD_TOKEN_INFO.save(storage, &token.address, &reward_configs)?;
            Ok(reward_configs)
        }

        pub fn to_config_response(self, storage: &dyn Storage) -> StdResult<ConfigResponse> {
            let tokens = Self::REWARD_TOKENS.load(storage)?;
            let mut infos = Vec::with_capacity(tokens.0.len());
            for token in tokens.0 {
                if let Some(reward_configs) = Self::REWARD_TOKEN_INFO.may_load(storage, &token)? {
                    for info in reward_configs {
                        let info = info.to_response()?;
                        infos.push(info);
                    }
                }
            }
            Ok(ConfigResponse {
                lp_token: self.lp_token,
                amm_pair: self.amm_pair,
                admin_auth: self.admin_auth,
                query_auth: self.query_auth,
                reward_tokens: infos,
                total_amount_staked: self.total_amount_staked,
            })
        }

        pub fn update_reward_per_token(
            &self,
            now: u64,
            info: &mut RewardTokenInfo,
        ) -> StdResult<Uint256> {
            info.update_reward_per_token(now, self.total_amount_staked)
        }

        pub fn may_load_staker(storage: &dyn Storage, user: &Addr) -> StdResult<Option<Staker>> {
            if let Some(staked) = Self::STAKERS.may_load(storage, user)? {
                Ok(Some(Staker {
                    addr: user.clone(),
                    staked: Uint128::new(staked),
                    claimable_rewards: HashMap::default(),
                }))
            } else {
                Ok(None)
            }
        }
    }

    impl RewardTokenInfo {
        pub const SECONDS_IN_DAY: U256 = U256::new(24u128 * 3600u128);
        pub const MAX_DECIMALS: U256 = exp10(18);

        pub fn normalize_amount(amount: impl Into<U256>) -> StdResult<U256> {
            let amount: U256 = amount.into();
            amount
                .checked_mul(Self::MAX_DECIMALS)
                .ok_or_else(|| StdError::generic_err("Overflow"))
        }

        pub fn denormalize_amount(amount: impl Into<U256>) -> StdResult<U256> {
            let amount: U256 = amount.into();
            amount
                .checked_div(Self::MAX_DECIMALS)
                .ok_or_else(|| StdError::generic_err("Overflow"))
        }

        pub fn init_from_daily_rewards(
            now: u64,
            token: &Contract,
            decimals: u8,
            daily_emission_amount: Uint128,
            valid_to: u64,
        ) -> StdResult<Self> {
            let daily_emission_amount = Self::normalize_amount(daily_emission_amount)?;
            Ok(Self {
                token: token.clone(),
                decimals,
                reward_per_second: (daily_emission_amount / Self::SECONDS_IN_DAY).into(),
                valid_to,
                reward_per_staked_token: Uint256::zero(),
                last_updated: now,
            })
        }

        pub fn update_reward_per_second(
            &mut self,
            daily_emission_amount: Uint128,
        ) -> StdResult<()> {
            let daily_emission_amount = Self::normalize_amount(daily_emission_amount)?;
            self.reward_per_second = (daily_emission_amount / Self::SECONDS_IN_DAY).into();
            Ok(())
        }

        /// Denormalizes the reward per second and reward per token stored.
        pub fn to_response(&self) -> StdResult<RewardTokenInfo> {
            Ok(RewardTokenInfo {
                token: self.token.clone(),
                decimals: self.decimals,
                reward_per_second: Self::denormalize_amount(self.reward_per_second)?.into(),
                valid_to: self.valid_to,
                reward_per_staked_token: Self::denormalize_amount(self.reward_per_staked_token)?
                    .into(),
                last_updated: self.last_updated,
            })
        }

        /// recalculates reward per staked token
        pub fn update_reward_per_token(
            &mut self,
            now: u64,
            total_staked: Uint128,
        ) -> StdResult<Uint256> {
            let min_time_rewards_applicable = min(now, self.valid_to);
            if !total_staked.is_zero() && min_time_rewards_applicable > self.last_updated {
                let time_since_updated =
                    U256::new((min_time_rewards_applicable - self.last_updated).into());
                let total_staked = U256::new(total_staked.u128());
                let rewards_since_updated = muldiv(
                    time_since_updated,
                    self.reward_per_second.into(),
                    total_staked,
                )?;
                self.reward_per_staked_token = self
                    .reward_per_staked_token
                    .checked_add(rewards_since_updated.into())?;
            }
            self.last_updated = min_time_rewards_applicable;
            Ok(self.reward_per_staked_token)
        }
    }

    #[cw_serde]
    pub struct Staker {
        pub addr: Addr,
        pub staked: Uint128,
        pub claimable_rewards: HashMap<Addr, Vec<ClaimableRewardsInfo>>,
    }

    impl Staker {
        pub fn new(addr: &Addr) -> Self {
            Self {
                addr: addr.clone(),
                staked: Uint128::zero(),
                claimable_rewards: HashMap::default(),
            }
        }
    }

    #[cw_serde]
    pub struct ClaimableRewardsInfo {
        pub info: RewardTokenInfo,
        pub amount: Uint128,
        pub last_reward_per_staked_token_paid: Uint256,
    }

    impl ClaimableRewardsInfo {
        pub fn new(info: &RewardTokenInfo) -> Self {
            Self {
                info: info.clone(),
                amount: Uint128::zero(),
                last_reward_per_staked_token_paid: Uint256::zero(),
            }
        }
    }

    impl<'a> Staker {
        pub const REWARDS: Map<'static, (&'a Addr, &'a Addr), Vec<ClaimableRewardsInfo>> =
            Map::new("staker_rewards");
    }

    impl Staker {
        pub fn get_rewards_key<'a>(&'a self, reward_token: &'a Addr) -> (&'a Addr, &'a Addr) {
            (reward_token, &self.addr)
        }

        pub fn total_staked(&self) -> Uint128 {
            self.staked
        }

        pub fn stake(
            &mut self,
            storage: &mut dyn Storage,
            amount: impl Into<Uint128> + Copy,
        ) -> StdResult<Uint128> {
            let amount = amount.into();
            self.staked = self.staked.checked_add(amount)?;
            Ok(amount)
        }

        pub fn unstake(
            &mut self,
            storage: &mut dyn Storage,
            amount: impl Into<Uint128> + Copy,
        ) -> StdResult<Uint128> {
            let amount = amount.into();
            self.staked = self.staked.checked_sub(amount)?;
            Ok(amount)
        }

        //called once per reward token type, loads data from storage into staker object, updating claimable amounts
        pub fn update_claimable_rewards(
            &mut self,
            storage: &dyn Storage,
            reward_infos: &Vec<RewardTokenInfo>,
            reward_token_address: Addr,
        ) -> StdResult<()> {
            let rewards_key = self.get_rewards_key(&reward_token_address);
            let mut claimable_rewards = match Self::REWARDS.may_load(storage, rewards_key)? {
                Some(data) => data,
                None => vec![],
            };

            for i in claimable_rewards.len()..reward_infos.len() {
                claimable_rewards.push(ClaimableRewardsInfo::new(&reward_infos[i]))
            }
            if claimable_rewards.len() != reward_infos.len() {
                return Err(StdError::generic_err(
                    "Off by one error in reward list padding",
                ));
            }
            for (reward_info, mut claimable_reward) in
                reward_infos.into_iter().zip(claimable_rewards.into_iter())
            {
                if reward_info.token.address != reward_token_address {
                    return Err(StdError::generic_err(
                        "Update claimable rewards bad reward token address",
                    ));
                }
                if reward_info.reward_per_staked_token
                    > claimable_reward.last_reward_per_staked_token_paid
                {
                    let reward_per_staked_token_earned = reward_info.reward_per_staked_token
                        - claimable_reward.last_reward_per_staked_token_paid;
                    let normalized_amount_earned =
                        reward_per_staked_token_earned.checked_mul(self.total_staked().into())?;
                    let reward_amount_earned =
                        RewardTokenInfo::denormalize_amount(normalized_amount_earned)?;
                    claimable_reward.amount = claimable_reward
                        .amount
                        .checked_add(reward_amount_earned.into())?;
                    claimable_reward.last_reward_per_staked_token_paid =
                        reward_info.reward_per_staked_token;
                }

                //load update claimable reward into object
                match self.claimable_rewards.get_mut(&reward_info.token.address) {
                    Some(list) => list.push(claimable_reward.clone()),
                    None => {
                        let list = vec![claimable_reward.clone()];
                        self.claimable_rewards
                            .insert(reward_info.token.address.clone(), list);
                    }
                }
            }
            Ok(())
        }

        /// write data from staker object to contract storage
        pub fn save_rewards(&self, storage: &mut dyn Storage) -> StdResult<()> {
            for reward in &self.claimable_rewards {
                let rewards_key = self.get_rewards_key(&reward.0);
                Self::REWARDS.save(storage, rewards_key, &reward.1)?;
            }
            Ok(())
        }

        /// send all claimable rewards to staker and save 0 into claimable rewards storage
        pub fn claim_and_save_rewards(
            &self,
            storage: &mut dyn Storage,
        ) -> StdResult<(Vec<CosmosMsg>, Vec<ClaimRewardResponse>)> {
            let mut response_data = vec![];
            let mut msgs = vec![];
            for rewards in self.claimable_rewards.iter() {
                let mut claimable_rewards = vec![];
                for reward in rewards.1 {
                    if !reward.amount.is_zero() {
                        response_data.push(ClaimRewardResponse {
                            token: reward.info.token.clone(),
                            amount: reward.amount,
                        });
                        msgs.push(
                            Snip20ExecuteMsg::Send {
                                recipient: self.addr.to_string(),
                                recipient_code_hash: None,
                                amount: reward.amount,
                                msg: None,
                                memo: None,
                                padding: None,
                            }
                            .to_cosmos_msg(&reward.info.token, vec![])?,
                        );
                    }
                    claimable_rewards.push(ClaimableRewardsInfo {
                        info: reward.info.clone(),
                        amount: Uint128::zero(),
                        last_reward_per_staked_token_paid: reward.last_reward_per_staked_token_paid,
                    });
                }

                let rewards_key = self.get_rewards_key(rewards.0);
                Self::REWARDS.save(storage, rewards_key, &claimable_rewards)?;
            }
            Ok((msgs, response_data))
        }

        pub fn to_staker_info_response(self) -> StdResult<PermitQueryResponse> {
            let mut all_rewards = vec![];
            for (_, rewards) in &self.claimable_rewards {
                all_rewards.append(
                    &mut rewards
                        .into_iter()
                        .map(|r| ClaimableRewardsResponse {
                            token: r.info.token.clone(),
                            amount: r.amount,
                        })
                        .collect(),
                );
            }
            Ok(PermitQueryResponse::StakerInfo {
                staked: self.staked,
                total_staked: self.total_staked(),
                claimable_rewards: all_rewards,
            })
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[cfg(feature = "staking")]
        #[test]
        fn test_max_emission_rate_no_panic() {
            let high_emissions = Uint128::new(exp10(77).as_u128());
            let info = RewardTokenInfo::init_from_daily_rewards(
                1u64,
                &Contract::default(),
                18u8,
                high_emissions,
                1u64,
            )
            .unwrap();
            let info = info.to_response().unwrap();
            let rps: Uint128 = info.reward_per_second.try_into().unwrap();
            assert_eq!(
                rps,
                high_emissions / Uint128::new(RewardTokenInfo::SECONDS_IN_DAY.as_u128())
            );
        }
    }
}
