//! ### Liquidity Book Tree Math Library
//! Author: Kent
//!
//! This module contains functions to interact with a tree of TreeUint24.

use cosmwasm_schema::cw_serde;
use ethnum::U256;
use std::collections::HashMap;

use super::bit_math::BitMath;
use crate::utils::liquidity_book::types::Bytes32;

/// Can store 256^3 = 16,777,216 values.
#[cw_serde]
pub struct TreeUint24 {
    pub level0: Bytes32,
    pub level1: HashMap<Bytes32, Bytes32>,
    pub level2: HashMap<Bytes32, Bytes32>,
}

impl Default for TreeUint24 {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeUint24 {
    /// Creates a new empty TreeUint24.
    pub fn new() -> Self {
        TreeUint24 {
            level0: Bytes32::default(),
            level1: HashMap::<Bytes32, Bytes32>::new(),
            level2: HashMap::<Bytes32, Bytes32>::new(),
        }
    }

    /// Checks if the tree contains the given `id`.
    ///
    /// Returns `true` if the tree contains the `id`.
    pub fn contains(&self, id: u32) -> bool {
        let key2 = (U256::from(id) >> 8u8).to_le_bytes();

        self.level2.get(&key2).is_some() & (1u32 << (id & u8::MAX as u32) != 0u32)
    }

    /// Adds the given `id` to the tree.
    ///
    /// Returns `true` if the `id` was not already in the tree.
    /// If the `id` was already in the tree, no changes are made and `false` is returned.
    // TODO: introduce u24
    pub fn add(&mut self, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        let leaves =
            U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]));
        let new_leaves = leaves | (U256::ONE << (id & u8::MAX as u32));

        if leaves != new_leaves {
            self.level2
                .insert(key2.to_le_bytes(), new_leaves.to_le_bytes());

            if leaves == U256::ZERO {
                let key1 = key2 >> 8u8;
                let leaves = U256::from_le_bytes(
                    *self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                let value1 = leaves | (U256::ONE << (key2 & U256::from(u8::MAX)));

                self.level1.insert(key1.to_le_bytes(), value1.to_le_bytes());

                if leaves == U256::ZERO {
                    let value0 = U256::from_le_bytes(self.level0)
                        | (U256::ONE << (key1 & U256::from(u8::MAX)));
                    self.level0 = value0.to_le_bytes();
                }
                return true;
            }
        }

        false
    }

    /// Removes the given `id` from the tree.
    ///
    /// Returns `true` if the `id` was in the tree.
    /// If the `id` was not in the tree, no changes are made and `false` is returned.
    pub fn remove(&mut self, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        let leaves =
            U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]));
        let new_leaves = leaves & !(U256::ONE << (id & u8::MAX as u32));

        if leaves != new_leaves {
            self.level2
                .insert(key2.to_le_bytes(), new_leaves.to_le_bytes());

            if new_leaves == U256::ZERO {
                let key1 = key2 >> 8u8;
                let leaves = U256::from_le_bytes(
                    *self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                let value1 = leaves & !(U256::ONE << (key2 & U256::from(u8::MAX)));
                self.level1.insert(key1.to_le_bytes(), value1.to_le_bytes());

                if leaves == U256::ZERO {
                    let value0 = U256::from_le_bytes(self.level0)
                        & !(U256::ONE << (key1 & U256::from(u8::MAX)));
                    self.level0 = value0.to_le_bytes();
                }
                return true;
            }
        }

        false
    }

    /// Finds the first `id` in the tree that is less than or equal to the given `id`.
    ///
    /// Returns the found `id`, or `u32::MAX` if there is no such `id` in the tree.
    pub fn find_first_right(&self, id: u32) -> u32 {
        let mut leaves: U256;

        let key2 = U256::from(id >> 8);
        let mut bit = (id & u32::from(u8::MAX)) as u8;

        if bit != 0 {
            // TODO: for all of the unwraps in this module, what should we do if it's None?
            leaves =
                U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]));
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                return (key2 << 8u8).as_u32() | closest_bit.as_u32();
            }
        }

        let key1 = key2 >> 8u8;
        bit = (key2 & U256::from(u8::MAX)).as_u8();

        if bit != 0 {
            leaves =
                U256::from_le_bytes(*self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]));
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                let key2 = key1 << 8u8 | closest_bit;
                leaves = U256::from_le_bytes(
                    *self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::most_significant_bit(leaves) as u32;
            }
        }

        bit = (key1 & U256::from(u8::MAX)).as_u8();

        if bit != 0 {
            leaves = U256::from_le_bytes(self.level0);
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                let key1 = closest_bit;
                leaves = U256::from_le_bytes(
                    *self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                let key2 = key1 << 8u8 | U256::from(BitMath::most_significant_bit(leaves));
                leaves = U256::from_le_bytes(
                    *self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::most_significant_bit(leaves) as u32;
            }
        }

        u32::MAX
    }

    /// Finds the first `id` in the tree that is greater than or equal to the given `id`.
    ///
    /// Returns the found `id`, or `0` if there is no such `id` in the tree.
    pub fn find_first_left(&self, id: u32) -> u32 {
        let mut leaves: U256;

        let key2 = U256::from(id >> 8);
        let mut bit = (id & u32::from(u8::MAX)) as u8;

        if bit != u8::MAX {
            leaves =
                U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]));
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                return (key2 << 8u8).as_u32() | closest_bit.as_u32();
            }
        }

        let key1 = key2 >> 8u8;
        bit = (key2 & U256::from(u8::MAX)).as_u8();

        if bit != u8::MAX {
            leaves =
                U256::from_le_bytes(*self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]));
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                let key2 = key1 << 8u8 | closest_bit;
                leaves = U256::from_le_bytes(
                    *self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::least_significant_bit(leaves) as u32;
            }
        }

        bit = (key1 & U256::from(u8::MAX)).as_u8();

        if bit != u8::MAX {
            leaves = U256::from_le_bytes(self.level0);
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                let key1 = closest_bit;
                leaves = U256::from_le_bytes(
                    *self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                let key2 = key1 << 8u8 | U256::from(BitMath::least_significant_bit(leaves));
                leaves = U256::from_le_bytes(
                    *self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::least_significant_bit(leaves) as u32;
            }
        }

        0u32
    }

    /// Helper function: finds the first bit in the given `leaves` that is strictly lower than the given `bit`.
    ///
    /// Returns the found bit, or `U256::MAX` if there is no such bit.
    fn _closest_bit_right(leaves: U256, bit: u8) -> U256 {
        BitMath::closest_bit_right(leaves, bit - 1)
    }

    /// Helper function: finds the first bit in the given `leaves` that is strictly higher than the given `bit`.
    ///
    /// Returns the found bit, or `U256::MAX` if there is no such bit.
    fn _closest_bit_left(leaves: U256, bit: u8) -> U256 {
        BitMath::closest_bit_left(leaves, bit + 1)
    }
}
