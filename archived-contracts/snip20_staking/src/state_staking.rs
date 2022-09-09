use shade_protocol::c_std::{Uint128, Uint256};
use shade_protocol::c_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::{
    contract_interfaces::staking::snip20_staking::stake::{
        Cooldown,
        DailyUnbonding,
        Unbonding,
        VecQueue,
    },
    utils::storage::default::{BucketStorage, SingletonStorage},
};

// used to determine what each token is worth to calculate rewards
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TotalShares(pub Uint256);

impl SingletonStorage for TotalShares {
    const NAMESPACE: &'static [u8] = b"total_shares";
}

// used to separate tokens minted from total tokens (includes rewards)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TotalTokens(pub Uint128);

impl SingletonStorage for TotalTokens {
    const NAMESPACE: &'static [u8] = b"total_tokens";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct UserShares(pub Uint256);

impl BucketStorage for UserShares {
    const NAMESPACE: &'static [u8] = b"user_shares";
}

// stores received token info if no treasury is set
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct UnsentStakedTokens(pub Uint128);

impl SingletonStorage for UnsentStakedTokens {
    const NAMESPACE: &'static [u8] = b"unsent_staked_tokens";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TotalUnbonding(pub Uint128);

impl SingletonStorage for TotalUnbonding {
    const NAMESPACE: &'static [u8] = b"total_unbonding";
}

// Distributors wrappers

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Distributors(pub Vec<Addr>);

impl SingletonStorage for Distributors {
    const NAMESPACE: &'static [u8] = b"distributors";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct DistributorsEnabled(pub bool);

impl SingletonStorage for DistributorsEnabled {
    const NAMESPACE: &'static [u8] = b"distributors_transfer";
}

// Unbonding Queues

#[cw_serde]
pub struct UnbondingQueue(pub VecQueue<Unbonding>);

impl BucketStorage for UnbondingQueue {
    const NAMESPACE: &'static [u8] = b"unbonding_queue";
}

#[cw_serde]
pub struct DailyUnbondingQueue(pub VecQueue<DailyUnbonding>);

impl SingletonStorage for DailyUnbondingQueue {
    const NAMESPACE: &'static [u8] = b"daily_unbonding_queue";
}

// Used for vote cooldown after send
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct UserCooldown {
    pub total: Uint128,
    pub queue: VecQueue<Cooldown>,
}

impl BucketStorage for UserCooldown {
    const NAMESPACE: &'static [u8] = b"user_cooldown";
}

impl UserCooldown {
    pub fn add_cooldown(&mut self, cooldown: Cooldown) {
        self.total += cooldown.amount;
        self.queue.push(&cooldown);
    }

    pub fn remove_cooldown(&mut self, amount: Uint128) {
        let mut remaining = amount;
        while remaining != Uint128::zero() {
            let index = self.queue.0.len() - 1;
            if self.queue.0[index].amount <= remaining {
                let item = self.queue.0.remove(index);
                remaining = remaining.checked_sub(item.amount).unwrap();
            } else {
                self.queue.0[index].amount =
                    self.queue.0[index].amount.checked_sub(remaining).unwrap();
                break;
            }
        }
    }

    pub fn update(&mut self, time: u64) {
        while !self.queue.0.is_empty() {
            if self.queue.0[0].release <= time {
                let i = self.queue.pop().unwrap();
                self.total = self.total.checked_sub(i.amount).unwrap();
            } else {
                break;
            }
        }
    }
}
