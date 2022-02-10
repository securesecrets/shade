use std::cmp::Ordering;
use std::collections::BinaryHeap;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{HumanAddr, Uint128};
use crate::storage::{BucketStorage, SingletonStorage};
use crate::utils::asset::Contract;

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

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DailyUnbonding {
    pub unbonding: u128,
    pub funded: u128,
    pub release: u64
}

impl DailyUnbonding {
    pub fn new(unbonding: u128, release: u64) -> Self {
        Self {
            unbonding,
            funded: 0,
            release
        }
    }

    pub fn is_funded(&self) -> bool {
        self.unbonding == self.funded
    }

    ///
    /// Attempts to fund, will return whatever amount wasnt used
    ///
    pub fn fund(&mut self, amount: u128) -> u128 {
        if self.is_funded() {
            return amount
        }

        let to_fund = &self.unbonding - &self.funded;
        if to_fund < amount {
            self.funded = self.unbonding.into();
            return amount - to_fund
        }

        self.funded += amount;
        return 0
    }
}

impl Ord for DailyUnbonding {
    fn cmp(&self, other: &Unbonding) -> Ordering {
        self.release.cmp(&other.release)
    }
}

impl PartialOrd for DailyUnbonding {
    fn partial_cmp(&self, other: &Unbonding) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Unbonding {
    pub amount: Uint128,
    pub release: u64,
}

impl Ord for Unbonding {
    fn cmp(&self, other: &Unbonding) -> Ordering {
        self.release.cmp(&other.release)
    }
}

impl PartialOrd for Unbonding {
    fn partial_cmp(&self, other: &Unbonding) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use crate::shd_staking::stake::DailyUnbonding;

    #[test]
    fn is_funded() {
        assert!(DailyUnbonding{ unbonding: 100, funded: 100, release: 0 }.is_funded());
        assert!(!DailyUnbonding{ unbonding: 150, funded: 100, release: 0 }.is_funded());
    }

    #[test]
    fn fund() {
        // Initialize new unbond
        let mut unbond = DailyUnbonding::new(500, 0);
        assert!(!unbond.is_funded());

        // Add small fund
        let residue = unbond.fund(250);
        assert_eq!(unbond.funded, 250);
        assert_eq!(residue, 0);

        // Add overflowing fund
        let residue = unbond.fund(500);
        assert!(unbond.is_funded());
        assert_eq!(residue, 250);

        // Add to funded fund
        let residue = unbond.fund(300);
        assert_eq!(residue, 300);
    }
}