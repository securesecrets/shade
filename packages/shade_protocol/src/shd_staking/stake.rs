use std::cmp::Ordering;
use std::collections::BinaryHeap;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{HumanAddr, Uint128};
use crate::utils::storage::{BucketStorage, SingletonStorage};
use crate::utils::asset::Contract;

// Configuration file for staking
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StakeConfig {
    pub unbond_time: u64,
    pub staked_token: Contract,
    pub decimal_difference: u8,
    pub treasury: Option<HumanAddr>
}

impl SingletonStorage for StakeConfig {
    const NAMESPACE: &'static [u8] = b"stake_config";
}

// Unbonding information for the total accross users
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DailyUnbonding {
    pub unbonding: Uint128,
    pub funded: Uint128,
    pub release: u64
}

impl DailyUnbonding {
    pub fn new(unbonding: Uint128, release: u64) -> Self {
        Self {
            unbonding,
            funded: Uint128::zero(),
            release
        }
    }

    pub fn is_funded(&self) -> bool {
        self.unbonding == self.funded
    }

    ///
    /// Attempts to fund, will return whatever amount wasnt used
    ///
    pub fn fund(&mut self, amount: Uint128) -> Uint128 {
        if self.is_funded() {
            return amount
        }

        let to_fund = (self.unbonding - self.funded).unwrap();
        if to_fund < amount {
            self.funded = self.unbonding.into();
            return (amount - to_fund).unwrap()
        }

        self.funded += amount;
        return Uint128::zero()
    }
}

impl Ord for DailyUnbonding {
    fn cmp(&self, other: &DailyUnbonding) -> Ordering {
        self.release.cmp(&other.release)
    }
}

impl PartialOrd for DailyUnbonding {
    fn partial_cmp(&self, other: &DailyUnbonding) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl VecQueueMerge for DailyUnbonding {
    fn merge(&mut self, item: &Self) {
        self.unbonding += item.unbonding;
    }
}

// Queue item
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueueItem {
    pub amount: Uint128,
    pub release: u64,
}

impl Ord for QueueItem {
    fn cmp(&self, other: &QueueItem) -> Ordering {
        self.release.cmp(&other.release)
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &QueueItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl VecQueueMerge for QueueItem {
    fn merge(&mut self, item: &Self) {
        self.amount += item.amount;
    }
}

// Queue item is used for both user unbonding and user vote cooldown
pub use QueueItem as Unbonding;
pub use QueueItem as Cooldown;

// A flexible queue system
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VecQueue<T: Ord + Serialize + Clone + VecQueueMerge>(pub Vec<T>);

impl<T: Ord + Serialize + Clone + VecQueueMerge> VecQueue<T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self(vec)
    }

    pub fn push(&mut self, item: &T) {
        // Look if item is in list
        match self.0.binary_search(item) {
            Ok(index) => {
                // Item is found so we update it
                self.0[index].merge(item);
            }
            Err(index) => {
                self.0.insert(index, item.clone());
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

pub trait VecQueueMerge {
    fn merge(&mut self, item: &Self);
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Uint128;
    use crate::shd_staking::stake::{DailyUnbonding, QueueItem, VecQueue};

    #[test]
    fn is_funded() {
        assert!(DailyUnbonding{ unbonding: Uint128(100), funded: Uint128(100), release: 0 }.is_funded());
        assert!(!DailyUnbonding{ unbonding: Uint128(150), funded: Uint128(100), release: 0 }.is_funded());
    }

    #[test]
    fn fund() {
        // Initialize new unbond
        let mut unbond = DailyUnbonding::new(Uint128(500), 0);
        assert!(!unbond.is_funded());

        // Add small fund
        let residue = unbond.fund(Uint128(250));
        assert_eq!(unbond.funded, Uint128(250));
        assert_eq!(residue, Uint128::zero());

        // Add overflowing fund
        let residue = unbond.fund(Uint128(500));
        assert!(unbond.is_funded());
        assert_eq!(residue, Uint128(250));

        // Add to funded fund
        let residue = unbond.fund(Uint128(300));
        assert_eq!(residue, Uint128(300));
    }

    #[test]
    fn vecqueue() {
        let mut vec: VecQueue<QueueItem> = VecQueue::new(vec![]);
        assert_eq!(vec.0.len(), 0);

        vec.push(&QueueItem {
            amount: Uint128(1),
            release: 1
        });
        vec.push(&QueueItem {
            amount: Uint128(1),
            release: 2
        });
        vec.push(&QueueItem {
            amount: Uint128(1),
            release: 2
        });
        vec.push(&QueueItem {
            amount: Uint128(1),
            release: 3
        });

        assert_eq!(vec.0[0], QueueItem {
            amount: Uint128(1),
            release: 1
        });
        assert_eq!(vec.0[1], QueueItem {
            amount: Uint128(2),
            release: 2
        });
        assert_eq!(vec.0[2], QueueItem {
            amount: Uint128(1),
            release: 3
        });
    }
}