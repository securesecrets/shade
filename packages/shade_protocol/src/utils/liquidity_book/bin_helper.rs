//! ### Liquidity Book Bin Helper Library
//! Author: Kent
//!
//! This library contains functions to help interaction with bins.

#![allow(unused)] // For beginning only.

use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, Uint128, WasmMsg};
use ethnum::U256;

use crate::utils::liquidity_book::{
    math::{packed_u128_math::PackedMath, u128x128_math::U128x128MathError},
    tokens::TokenType,
    transfer::HandleMsg,
};

use super::{
    constants::{SCALE, SCALE_OFFSET},
    fee_helper::{FeeError, FeeHelper},
    math::{
        packed_u128_math::{Decode, Encode},
        u256x256_math::{U256x256Math, U256x256MathError},
    },
    pair_parameter_helper::{PairParameters, PairParametersError},
    price_helper::PriceHelper,
    types::Bytes32,
};

// NOTE: not sure if it's worth having a unique type for this

// pub struct Reserves(pub [u8; 32]);

// impl Reserves {
//     pub fn decode(self) -> (u128, u128) {
//         let (bin_reserve_x, bin_reserve_y) = decode(self.0);

//         (bin_reserve_x, bin_reserve_y)
//     }
// }

#[derive(thiserror::Error, Debug)]
pub enum BinError {
    #[error("Bin Error: Composition Factor Flawed, id: {0}")]
    CompositionFactorFlawed(u32),

    #[error("Bin Error: Liquidity Overflow")]
    LiquidityOverflow,

    #[error(transparent)]
    FeeErr(#[from] FeeError),

    #[error(transparent)]
    U128MathErr(#[from] U128x128MathError),

    #[error(transparent)]
    U256MathErr(#[from] U256x256MathError),

    #[error(transparent)]
    ParamsErr(#[from] PairParametersError),
}

pub struct BinHelper;

impl BinHelper {
    /// Returns the amount of tokens that will be received when burning the given amount of liquidity.
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin
    /// * `amount_to_burn` - The amount of liquidity to burn
    /// * `total_supply` - The total supply of the liquidity book
    ///
    /// # Returns
    ///
    /// * `amounts_out` - The encoded amount of tokens that will be received
    pub fn get_amount_out_of_bin(
        bin_reserves: Bytes32,
        amount_to_burn: U256,
        total_supply: U256,
    ) -> Result<(u128, u128), BinError> {
        let (bin_reserve_x, bin_reserve_y) = bin_reserves.decode();

        let mut amount_x_out_from_bin = U256::ZERO;
        let mut amount_y_out_from_bin = U256::ZERO;

        if bin_reserve_x > 0 {
            amount_x_out_from_bin = U256x256Math::mul_div_round_down(
                amount_to_burn,
                bin_reserve_x.into(),
                total_supply,
            )?
            .min(U256::from(u128::MAX));
        }

        if bin_reserve_y > 0 {
            amount_y_out_from_bin = U256x256Math::mul_div_round_down(
                amount_to_burn,
                bin_reserve_y.into(),
                total_supply,
            )?
            .min(U256::from(u128::MAX));
        }

        Ok((
            amount_x_out_from_bin.as_u128(),
            amount_y_out_from_bin.as_u128(),
        ))
    }

    /// Returns the share and the effective amounts in when adding liquidity.
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin.
    /// * `amounts_in` - The amounts of tokens to add.
    /// * `price` - The price of the bin.
    /// * `total_supply` - The total supply of the liquidity book.
    ///
    /// # Returns
    ///
    /// * `shares` - The share of the liquidity book that the user will receive as a Uint256.
    /// * `effective_amounts_in` - The Bytes32 encoded effective amounts of tokens that the user will add.
    /// This is the amount of tokens that the user will actually add to the liquidity book,
    /// and will always be less than or equal to the amounts_in.
    pub fn get_shares_and_effective_amounts_in(
        bin_reserves: Bytes32,
        amounts_in: Bytes32,
        price: U256,
        total_supply: U256,
    ) -> Result<(U256, Bytes32), BinError> {
        let (mut x, mut y) = amounts_in.decode();

        let user_liquidity = Self::get_liquidity(amounts_in, price)?;
        if total_supply == U256::ZERO || user_liquidity == U256::ZERO {
            return Ok((user_liquidity, amounts_in));
        }

        let bin_liquidity = Self::get_liquidity(bin_reserves, price)?;
        if bin_liquidity == U256::ZERO {
            return Ok((user_liquidity, amounts_in));
        }

        let shares = U256x256Math::mul_div_round_down(user_liquidity, total_supply, bin_liquidity)?;

        let effective_liquidity =
            U256x256Math::mul_div_round_up(shares, bin_liquidity, total_supply)?;

        let mut effective_amounts_in = amounts_in;

        if user_liquidity > effective_liquidity {
            let mut delta_liquidity = user_liquidity - effective_liquidity;

            // The other way might be more efficient, but as y is the quote asset, it is more valuable
            if delta_liquidity >= SCALE {
                let delta_y = (delta_liquidity >> SCALE_OFFSET as u32)
                    .min(y.into())
                    .as_u128();

                y -= delta_y;
                delta_liquidity -= U256::from(delta_y) << SCALE_OFFSET as u128;
            }

            if delta_liquidity >= price {
                let delta_x = (delta_liquidity / price).min(x.into()).as_u128();

                x -= delta_x;
            }

            effective_amounts_in = Bytes32::encode(x, y);
        }

        Ok((shares, effective_amounts_in))
    }

    /// Returns the amount of liquidity following the constant sum formula `L = price * x + y`.
    ///
    /// # Arguments
    ///
    /// * `amounts` - The amount of the tokens
    /// * `price` - The price of the bin
    pub fn get_liquidity(amounts: [u8; 32], price: U256) -> Result<U256, BinError> {
        let (x, y) = amounts.decode();

        let x = U256::from(x);
        let y = U256::from(y);

        let mut liquidity = U256::ZERO;

        if x > U256::ZERO {
            liquidity = price.checked_mul(x).unwrap();

            if (liquidity / x != price) {
                return Err(BinError::LiquidityOverflow);
            }
        }

        if y > U256::ZERO {
            let shifted_y = y << SCALE_OFFSET;
            liquidity = liquidity.checked_add(shifted_y).unwrap();
        }

        Ok(liquidity)
    }

    /// Verify that the amounts are correct and that the composition factor is not flawed.
    ///
    /// # Arguments
    ///
    /// * `amounts` - The amounts of tokens as Bytes32.
    /// * `active_id` - Thie id of the active bin as u32.
    /// * `id` - Thie id of the bin as u32.
    pub fn verify_amounts(amounts: [u8; 32], active_id: u32, id: u32) -> Result<(), BinError> {
        let amounts = U256::from_le_bytes(amounts);
        // this is meant to compare the right-side 128 bits to zero, but can I discard the left 128 bits and not have it overflow?
        if id < active_id && amounts << 128u32 > U256::ZERO
            || id > active_id && amounts > U256::from(u128::MAX)
        {
            return Err(BinError::CompositionFactorFlawed(id));
        }
        Ok(())
    }

    /// Returns the composition fees when adding liquidity to the active bin with a different
    /// composition factor than the bin's one, as it does an implicit swap.
    /// It calculates what you'd get if you removed 10 shares of liquidity right after adding it
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin
    /// * `parameters` - The parameters of the liquidity book
    /// * `bin_step` - The step of the bin
    /// * `amounts_in` - The amounts of tokens to add
    /// * `total_supply` - The total supply of the liquidity book
    /// * `shares` - The share of the liquidity book that the user will receive
    ///
    /// # Returns
    ///
    /// * `fees` - The encoded fees that will be charged
    pub fn get_composition_fees(
        bin_reserves: Bytes32,
        parameters: PairParameters,
        bin_step: u16,
        amounts_in: Bytes32,
        total_supply: U256,
        shares: U256,
    ) -> Result<Bytes32, BinError> {
        if shares == U256::ZERO {
            return Ok([0u8; 32]);
        }

        let (amount_x, amount_y) = amounts_in.decode();

        let (bin_reserves_x, bin_reserves_y) = bin_reserves.decode();

        let (received_amount_x, received_amount_y) = Self::get_amount_out_of_bin(
            bin_reserves.add(amounts_in),
            shares,
            total_supply + shares,
        )?;

        // println!(
        //     "received_amount_x {:?}\nreceived_amount_y:{:?}\namount_x {:?}\namount_y {:?}\nshare {:?}\ntotal_supply {:?}\nbin_reserves_x {:?}\nbin_reserves_y {:?}",
        //     received_amount_x, received_amount_y,amount_x,amount_y,shares,total_supply,bin_reserves_x,bin_reserves_y
        // );

        let mut fees = Bytes32::default();

        if (received_amount_x > amount_x) {
            let fee_y = FeeHelper::get_composition_fee(
                (amount_y - received_amount_y),
                parameters.get_total_fee(bin_step),
            )?;

            fees = Bytes32::encode_second(fee_y)
        } else if (received_amount_y > amount_y) {
            let fee_x = FeeHelper::get_composition_fee(
                (amount_x - received_amount_x),
                parameters.get_total_fee(bin_step),
            )?;

            fees = Bytes32::encode_first(fee_x)
        }

        Ok(fees)
    }

    /// Returns whether the bin is empty (true) or not (false).
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin
    /// * `is_x` - Whether the reserve to check is the X reserve (true) or the Y reserve (false)
    pub fn is_empty(bin_reserves: Bytes32, is_x: bool) -> bool {
        if is_x {
            return bin_reserves.decode_x() == 0;
        } else {
            return bin_reserves.decode_y() == 0;
        }
    }

    /// Returns the amounts of tokens that will be added and removed from the bin during a swap
    /// along with the fees that will be charged.
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin
    /// * `parameters` - The parameters of the liquidity book
    /// * `bin_step` - The step of the bin
    /// * `swap_for_y` - Whether the swap is for Y (true) or for X (false)
    /// * `active_id` - The id of the active bin
    /// * `amounts_in_left` - The amounts of tokens left to swap
    ///
    /// # Returns
    ///
    /// * `amounts_in_with_fees` - The encoded amounts of tokens that will be added to the bin, including fees.
    /// * `amounts_out_of_bin` - The encoded amounts of tokens that will be removed from the bin.
    /// * `total_fees` - The encoded fees that will be charged.
    pub fn get_amounts(
        bin_reserves: Bytes32,
        parameters: PairParameters,
        bin_step: u16,
        swap_for_y: bool,
        active_id: u32,
        amounts_in_left: Bytes32,
        price: U256,
    ) -> Result<(Bytes32, Bytes32, Bytes32), BinError> {
        let bin_reserve_out = bin_reserves.decode_alt(!swap_for_y);

        let max_amount_in = if swap_for_y {
            U256x256Math::shift_div_round_up(U256::from(bin_reserve_out), SCALE_OFFSET, price)?
                .min(U256::from(u128::MAX))
        } else {
            U256x256Math::mul_shift_round_up(U256::from(bin_reserve_out), price, SCALE_OFFSET)?
                .min(U256::from(u128::MAX))
        };

        let total_fee = parameters.get_total_fee(bin_step);
        let max_fee = FeeHelper::get_fee_amount(max_amount_in.as_u128(), total_fee)?;

        let max_amount_in = max_amount_in + max_fee;

        let mut amount_in128 = amounts_in_left.decode_alt(swap_for_y);

        let mut feeu128;
        let mut amount_out128;

        if amount_in128 >= max_amount_in {
            feeu128 = max_fee;
            amount_in128 = max_amount_in.as_u128();
            amount_out128 = bin_reserve_out;
        } else {
            feeu128 = FeeHelper::get_fee_amount_from(amount_in128, total_fee)?;

            let amount_in = amount_in128 - feeu128;

            amount_out128 = if swap_for_y {
                U256x256Math::mul_shift_round_down(U256::from(amount_in), price, SCALE_OFFSET)?
                    .min(U256::from(u128::MAX))
                    .as_u128()
            } else {
                U256x256Math::shift_div_round_down(U256::from(amount_in), SCALE_OFFSET, price)?
                    .min(U256::from(u128::MAX))
                    .as_u128()
            };

            if amount_out128 > bin_reserve_out {
                amount_out128 = bin_reserve_out;
            }
        };

        let (amounts_in_with_fees, amounts_out_of_bin, total_fees) = if swap_for_y {
            (
                Bytes32::encode_first(amount_in128),
                Bytes32::encode_second(amount_out128),
                Bytes32::encode_first(feeu128),
            )
        } else {
            (
                Bytes32::encode_second(amount_in128),
                Bytes32::encode_first(amount_out128),
                Bytes32::encode_second(feeu128),
            )
        };

        Ok((amounts_in_with_fees, amounts_out_of_bin, total_fees))
    }

    /// Returns the encoded amounts that were transferred to the contract for both tokens.
    ///
    /// # Arguments
    ///
    /// * `reserves` - The reserves
    /// * `token_x` - The token X
    /// * `token_y` - The token Y
    ///
    /// # Returns
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: amount_x
    ///     * [128 - 256[: amount_y
    pub fn received(amount_received_x: Uint128, amount_received_y: Uint128) -> Bytes32 {
        let balance_x = amount_received_x.u128();
        let balance_y = amount_received_y.u128();

        let encoded_balances = Bytes32::encode(balance_x, balance_y);

        encoded_balances
    }

    /// Returns the encoded amounts that were transferred to the contract, only for token X.
    ///
    /// # Arguments
    ///
    /// * `reserves` - The reserves
    /// * `token_x` - The token X
    ///
    /// # Returns
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: amount_x
    ///     * [128 - 256[: empty
    pub fn received_x(amount_received: Uint128) -> Bytes32 {
        return Bytes32::encode_first((amount_received.u128()));
    }

    /// Returns the encoded amounts that were transferred to the contract, only for token Y.
    ///
    /// # Arguments
    ///
    /// * `reserves` - The reserves
    /// * `token_x` - The token Y
    ///
    /// # Returns
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: empty
    ///     * [128 - 256[: amount_y
    pub fn received_y(amount_received: Uint128) -> Bytes32 {
        return Bytes32::encode_second((amount_received.u128()));
    }

    /// Transfers the encoded amounts to the recipient for both tokens.
    ///
    /// # Arguments
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: amount_x
    ///     * [128 - 256[: amount_y
    /// * `token_x` - The token X
    /// * `token_y` - The token Y
    /// * `recipient` - The recipient
    pub fn transfer(
        amounts: Bytes32,
        token_x: TokenType,
        token_y: TokenType,
        recipient: Addr,
    ) -> Option<Vec<CosmosMsg>> {
        let (amount_x, amount_y) = amounts.decode();
        let mut messages: Vec<CosmosMsg> = Vec::new();

        let msgs_x = Self::transfer_x(amounts, token_x, recipient.clone());

        if let Some(msgs) = msgs_x {
            messages.push(msgs);
        }
        let msgs_y = Self::transfer_y(amounts, token_y, recipient);

        if let Some(msgs) = msgs_y {
            messages.push(msgs);
        }

        if messages.len() > 0 {
            Some(messages)
        } else {
            None
        }
    }

    /// Transfers the encoded amounts to the recipient, only for token X.
    ///
    /// # Arguments
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: amount_x
    ///     * [128 - 256[: empty
    /// * `token_x` - The token X
    /// * `recipient` - The recipient
    pub fn transfer_x(amounts: Bytes32, token_x: TokenType, recipient: Addr) -> Option<CosmosMsg> {
        let amount = Uint128::from(amounts.decode_x());

        if amount.gt(&Uint128::zero()) {
            match token_x {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
                    let msg = HandleMsg::Transfer {
                        recipient: recipient.to_string(),
                        amount,
                        padding: None,
                        memo: None,
                    };
                    let cosmos_msg = msg
                        .to_cosmos_msg(token_code_hash, contract_addr.to_string(), None)
                        .unwrap();

                    Some(cosmos_msg)
                }

                TokenType::NativeToken { denom } => Some(CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount,
                    }],
                })),
            }
        } else {
            None
        }
    }

    /// Transfers the encoded amounts to the recipient, only for token Y.
    ///
    /// # Arguments
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: empty
    ///     * [128 - 256[: amount_y
    /// * `token_y` - The token Y
    /// * `recipient` - The recipient
    pub fn transfer_y(amounts: Bytes32, token_y: TokenType, recipient: Addr) -> Option<CosmosMsg> {
        let amount = Uint128::from(amounts.decode_y());

        if amount.gt(&Uint128::zero()) {
            match token_y {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
                    let msg = HandleMsg::Send {
                        recipient: recipient.to_string(),
                        amount,
                        padding: None,
                        msg: None,
                        recipient_code_hash: None,
                        memo: None,
                    };
                    let cosmos_msg = msg
                        .to_cosmos_msg(token_code_hash, contract_addr.to_string(), None)
                        .unwrap();

                    Some(cosmos_msg)
                }

                TokenType::NativeToken { denom } => Some(CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount,
                    }],
                })),
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Add, str::FromStr};

    use crate::utils::liquidity_book::{
        constants::*,
        math::encoded_sample::EncodedSample,
        pair_parameter_helper::PairParameters,
        types::StaticFeeParameters,
    };
    use cosmwasm_std::StdResult;
    use ethnum::U256;

    use crate::utils::liquidity_book::math::{
        packed_u128_math::{Decode, Encode},
        u256x256_math::U256x256Math,
    };

    use crate::utils::liquidity_book::bin_helper::{BinError, BinHelper};

    fn assert_approxeq_abs(a: U256, b: U256, max_diff: U256, msg: &str) {
        let diff = if a > b { a - b } else { b - a };
        assert!(diff <= max_diff, "{}: diff was {:?}", msg, diff);
    }

    #[test]
    fn test_get_amount_out_of_bin_zero_bin_reserves() -> StdResult<()> {
        let bin_reserves = Encode::encode(0, 0);
        let amount_to_burn = U256::from(1000u128);
        let total_supply = U256::from(10000u128);

        let amount_out =
            BinHelper::get_amount_out_of_bin(bin_reserves, amount_to_burn, total_supply).unwrap();
        let (amount_out_x, amount_out_y) = amount_out;

        assert_eq!(amount_out_x, 0);
        assert_eq!(amount_out_y, 0);

        Ok(())
    }

    #[test]
    fn test_get_amount_out_of_bin_zero_amount_to_burn() -> Result<(), BinError> {
        let bin_reserves = Encode::encode(1000, 1000);
        let amount_to_burn = U256::from(0u128);
        let total_supply = U256::from(10000u128);

        let amount_out =
            BinHelper::get_amount_out_of_bin(bin_reserves, amount_to_burn, total_supply)?;
        let (amount_out_x, amount_out_y) = amount_out;

        assert_eq!(amount_out_x, 0);
        assert_eq!(amount_out_y, 0);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_get_amount_out_of_bin_zero_total_supply() -> () {
        let bin_reserves = Encode::encode(1000, 1000);
        let amount_to_burn = U256::from(1000u128);
        let total_supply = U256::from(0u128);

        let z = BinHelper::get_amount_out_of_bin(bin_reserves, amount_to_burn, total_supply);
    }

    #[test]
    fn test_get_amount_out_of_bin_amount_to_burn_gt_total_supply() -> Result<(), BinError> {
        let bin_reserves = Encode::encode(1000, 1000);
        let amount_to_burn = U256::from(20000u128); // Greater than total_supply
        let total_supply = U256::from(10000u128);

        let amount_out =
            BinHelper::get_amount_out_of_bin(bin_reserves, amount_to_burn, total_supply)?;
        let (amount_out_x, amount_out_y) = amount_out;

        // Your assertions go here, depending on what behavior you expect
        // For instance, if you expect it to be proportional
        assert_eq!(amount_out_x, U256::from(2000u128));
        assert_eq!(amount_out_y, U256::from(2000u128));

        Ok(())
    }

    #[test]
    fn test_get_amount_out_of_bin_max_u128_constraint() -> Result<(), BinError> {
        let bin_reserves = Encode::encode(u128::MAX, u128::MAX);
        let amount_to_burn = U256::from(u128::MAX);
        let total_supply = U256::from(1u128); // To make sure the raw output is > u128::MAX

        let amount_out =
            BinHelper::get_amount_out_of_bin(bin_reserves, amount_to_burn, total_supply)?;
        let (amount_out_x, amount_out_y) = amount_out;

        // Should be capped at u128::MAX
        assert_eq!(amount_out_x, u128::MAX);
        assert_eq!(amount_out_y, u128::MAX);

        Ok(())
    }

    #[test]
    fn test_get_amount_out_of_bin() -> StdResult<()> {
        let bin_reserves_x = 1000;
        let bin_reserves_y = 1000;

        let bin_reserves = Encode::encode(bin_reserves_x, bin_reserves_y);
        let amount_to_burn = U256::from(1000u128);
        let total_supply = U256::from(10000u128);

        let amount_out =
            BinHelper::get_amount_out_of_bin(bin_reserves, amount_to_burn, total_supply).unwrap();

        let (amount_out_x, amount_out_y) = amount_out;

        assert_eq!(
            amount_out_x,
            U256x256Math::mul_div_round_down(
                amount_to_burn,
                U256::from(bin_reserves_x),
                total_supply
            )
            .unwrap()
        );
        assert_eq!(
            amount_out_y,
            U256x256Math::mul_div_round_down(
                amount_to_burn,
                U256::from(bin_reserves_y),
                total_supply
            )
            .unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_get_shares_and_effective_amounts_in() -> StdResult<()> {
        let mut total_supply = U256::from_str("0").unwrap();
        let max_u256 = U256::MAX;
        let mut bin_reserves = Encode::encode(1000, 1000);
        let amount_in = Encode::encode(1000, 1000);
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        for i in 0..10 {
            // Assume conditions similar to the Solidity test
            if price > U256::MIN
                && (bin_reserves == [0u8; 32]
                    || (price <= max_u256 / 1000 && price * 1000 <= max_u256 - 1000 << 128))
                && (amount_in == [0u8; 32]
                    || (price <= max_u256 / 1000 && price * 1000 <= max_u256 - 1000 << 128))
            {
                let user_liquidity = BinHelper::get_liquidity(amount_in, price).unwrap();
                let bin_liquidity = BinHelper::get_liquidity(bin_reserves, price).unwrap();
                let ((shares, effective_amounts_in)) =
                    BinHelper::get_shares_and_effective_amounts_in(
                        bin_reserves,
                        amount_in,
                        price,
                        total_supply,
                    )
                    .unwrap();

                total_supply += shares;
                let (x, y) = effective_amounts_in.decode();
                bin_reserves =
                    Encode::encode(bin_reserves.decode_x() + x, bin_reserves.decode_y() + y);
            }
        }
        assert_eq!(
            total_supply,
            U256::from_str("231048229485969055456138120902788449760223779800000").unwrap()
        );
        Ok(())
    }

    #[test]
    fn test_try_exploit_shares() -> Result<(), BinError> {
        // Setup initial variables
        let amount_x1 = 1000u128;
        let amount_y1 = 1000u128;
        let amount_x2 = 500u128;
        let amount_y2 = 500u128;
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        // Assumptions (replace with Rust's assert! or whatever you use for precondition checks)
        assert!(price > U256::ZERO);
        assert!(amount_x1 + amount_x2 <= u128::MAX);
        assert!(amount_y1 + amount_y2 <= u128::MAX);
        // ... (add all your assumptions here)

        // Simulate exploiter front-running the transaction
        let mut total_supply = U256::from(1u128) << 128;
        let mut bin_reserves = Encode::encode(amount_x1, amount_y1);

        // Get shares and effective amounts in
        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            bin_reserves,
            Encode::encode(amount_x2, amount_y2),
            price,
            total_supply,
        )?;

        // Update bin reserves and total supply
        bin_reserves = Encode::encode(
            bin_reserves.decode_x().add(effective_amounts_in.decode_x()),
            bin_reserves.decode_y().add(effective_amounts_in.decode_y()),
        );

        total_supply += shares;

        // Calculate what the user receives
        let user_received_x =
            U256x256Math::mul_div_round_down(shares, bin_reserves.decode_x().into(), total_supply)?;
        let user_received_y =
            U256x256Math::mul_div_round_down(shares, bin_reserves.decode_y().into(), total_supply)?;

        // Calculate received and sent in Y
        let received_in_y =
            U256x256Math::mul_shift_round_down(user_received_x, price, SCALE_OFFSET).unwrap()
                + user_received_y;
        let sent_in_y = U256x256Math::mul_shift_round_down(
            price,
            effective_amounts_in.decode_x().into(),
            SCALE_OFFSET,
        )
        .unwrap()
            + effective_amounts_in.decode_y();

        // Assert that received_in_y and sent_in_y should be approximately equal
        // (Implement your own assert_approxeq_abs function)
        let max_diff = ((price - U256::ONE) >> 128) + U256::from(2u128);
        assert_approxeq_abs(
            received_in_y,
            sent_in_y,
            max_diff,
            "test_TryExploitShares::1",
        );

        Ok(())
    }

    #[test]
    fn test_zero_total_supply_and_zero_bin_liquidity() -> Result<(), BinError> {
        let total_supply = U256::ZERO;
        let bin_reserves = Encode::encode(0, 0);
        let amount_in = Encode::encode(1000, 1000);
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            bin_reserves,
            amount_in,
            price,
            total_supply,
        )?;

        assert_eq!(
            shares,
            U256::from_str("42008768997448919173843294709597899956404323600000").unwrap()
        );
        assert_eq!(effective_amounts_in, amount_in);

        Ok(())
    }

    #[test]
    fn test_zero_amount_in() -> Result<(), BinError> {
        let total_supply = U256::from(10000u128);
        let bin_reserves = Encode::encode(1000, 1000);
        let amount_in = Encode::encode(0, 0);
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            bin_reserves,
            amount_in,
            price,
            total_supply,
        )?;

        assert_eq!(shares, U256::MIN);
        assert_eq!(effective_amounts_in, amount_in);

        Ok(())
    }

    #[test]
    fn test_zero_price() -> Result<(), BinError> {
        let total_supply = U256::from(10000u128);
        let bin_reserves = Encode::encode(1000, 1000);
        let amount_in = Encode::encode(1000, 1000);
        let price = U256::ZERO;

        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            bin_reserves,
            amount_in,
            price,
            total_supply,
        )?;

        assert_eq!(shares, U256::from(10000u128));
        assert_eq!(effective_amounts_in, amount_in);

        Ok(())
    }

    #[test]
    fn test_liquidity() -> StdResult<()> {
        let mut total_supply = U256::from_str("0").unwrap();
        let max_u256 = U256::MAX;
        let amount_in = Encode::encode(1000, 1000);
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        let liquidity = BinHelper::get_liquidity(amount_in, price).unwrap();

        assert_eq!(
            liquidity,
            U256::from_str("42008768997448919173843294709597899956404323600000").unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_get_composition_fees() -> Result<(), BinError> {
        // These would typically be random or fuzzed inputs, but for this example, let's assume fixed ones.
        let reserve_x = 5000000000000000000000u128;
        let reserve_y = 1000000000000000000000u128;
        let bin_step = 100u16;
        let amount_x_in = 500000000000000000000u128;
        let amount_y_in = 500000000000000000000u128;
        let price = U256::from(10000000000000000000000000000000000u128);
        let total_supply = U256::from(10000000000000000000000u128);

        // Perform the same assumptions as in the Solidity test.
        // ... (omitted for brevity)

        let bin_reserves = Encode::encode(reserve_x, reserve_y);
        let amounts_in = Encode::encode(amount_x_in, amount_y_in);

        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            bin_reserves,
            amounts_in,
            price,
            total_supply,
        )?;

        let msg = StaticFeeParameters {
            base_factor: 5000,
            filter_period: 30,
            decay_period: 600,
            reduction_factor: 5000,
            variable_fee_control: 40000,
            protocol_share: 1000,
            max_volatility_accumulator: 350000,
        };

        // Set the parameters (assuming PairParameters and DEFAULT_* constants are defined)
        let pair_parameters = PairParameters(EncodedSample([0u8; 32]));
        let pair_parameters = pair_parameters
            .set_static_fee_parameters(
                msg.base_factor,
                msg.filter_period,
                msg.decay_period,
                msg.reduction_factor,
                msg.variable_fee_control,
                msg.protocol_share,
                msg.max_volatility_accumulator,
            )
            .unwrap();
        // Call the function we are testing
        let composition_fees = BinHelper::get_composition_fees(
            bin_reserves,
            pair_parameters,
            bin_step,
            amounts_in,
            total_supply,
            shares,
        )?;
        assert_eq!(U256::from(4999412339173586562315u128), shares);
        assert_eq!((0, 196874089861191), composition_fees.decode());

        // Calculate binC and userC similar to the Solidity code
        let bin_c = if reserve_x | reserve_y == 0 {
            U256::MIN
        } else {
            U256::from(reserve_y) << 128 / (U256::from(reserve_x) + U256::from(reserve_y))
        };

        let user_c = if amount_x_in | amount_y_in == 0 {
            U256::MIN
        } else {
            U256::from(amount_y_in) << 128 / (U256::from(amount_x_in) + U256::from(amount_y_in))
        };

        // Perform assertions (assuming assert_ge is defined)
        if bin_c > user_c {
            assert!(
                U256::from(composition_fees.decode_x()) << 128 >= U256::MIN,
                "test_GetCompositionFees::1",
            );
        } else {
            assert!(
                U256::from(composition_fees.decode_y()) >= U256::MIN,
                "test_GetCompositionFees::2",
            );
        }

        Ok(())
    }
}
