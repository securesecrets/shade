//! ### Liquidity Book Bin Helper Library
//! Author: Kent
//!
//! This library contains functions to help interaction with bins.

#![allow(unused)] // For beginning only.

use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, Uint128, WasmMsg};
use ethnum::U256;

use crate::utils::liquidity_book::math::packed_u128_math::PackedMath;
use crate::utils::liquidity_book::math::u128x128_math::U128x128MathError;
use crate::utils::liquidity_book::tokens::TokenType;
use crate::utils::liquidity_book::transfer::HandleMsg;

use super::constants::{SCALE, SCALE_OFFSET};
use super::fee_helper::{FeeError, FeeHelper};
use super::math::packed_u128_math::{Decode, Encode};
use super::math::u256x256_math::{U256x256Math, U256x256MathError};
use super::pair_parameter_helper::{PairParameters, PairParametersError};
use super::price_helper::PriceHelper;
use super::types::Bytes32;

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
    ) -> Result<Bytes32, BinError> {
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

        Ok(Bytes32::encode(
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

        let user_liquidity = Self::get_liquidity(amounts_in, price);
        if total_supply == U256::ZERO || user_liquidity == U256::ZERO {
            return Ok((user_liquidity, amounts_in));
        }

        let bin_liquidity = Self::get_liquidity(bin_reserves, price);
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
    pub fn get_liquidity(amounts: [u8; 32], price: U256) -> U256 {
        let (x, y) = amounts.decode();
        println!("GET_LIQ X {:?} AND Y,,: {:?}", x, y);

        let x = U256::from(x);
        let y = U256::from(y);

        println!("GET_LIQ X {:?} AND Y: {:?}", x, y);

        let mut liquidity = U256::ZERO;

        if x > U256::ZERO {
            liquidity = price.checked_mul(x).unwrap();
        }

        // println!("Liquidity {:?}", liquidity);

        if y > U256::ZERO {
            let shifted_y = y << 128;
            liquidity = liquidity.checked_add(shifted_y).unwrap();
        }

        // println!("Liquidity {:?}", liquidity);

        liquidity
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

        let (received_amount_x, received_amount_y) = Self::get_amount_out_of_bin(
            bin_reserves.add(amounts_in),
            shares,
            total_supply + shares,
        )?
        .decode();

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
            return bin_reserves.decode_alt(is_x) == 0;
        }
        bin_reserves.decode_alt(!is_x) == 0
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
    ) -> Result<(Bytes32, Bytes32, Bytes32), BinError> {
        let price = PriceHelper::get_price_from_id(active_id, bin_step)?;

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

        let amount_in128 = amounts_in_left.decode_alt(swap_for_y);
        let (fee128, amount_out128) = if amount_in128 >= max_amount_in {
            (max_fee, bin_reserve_out)
        } else {
            let fee128 = FeeHelper::get_fee_amount_from(amount_in128, total_fee)?;

            let amount_in = amount_in128 - fee128;

            let amount_out128 = if swap_for_y {
                U256x256Math::mul_shift_round_down(U256::from(amount_in), price, SCALE_OFFSET)?
                    .min(U256::from(u128::MAX))
                    .as_u128()
            } else {
                U256x256Math::shift_div_round_down(U256::from(amount_in), SCALE_OFFSET, price)?
                    .min(U256::from(u128::MAX))
                    .as_u128()
            };

            if amount_out128 > bin_reserve_out {
                (fee128, bin_reserve_out)
            } else {
                (fee128, amount_out128)
            }
        };

        let (amounts_in_with_fees, amounts_out_of_bin, total_fees) = if swap_for_y {
            (
                Bytes32::encode_first(amount_in128),
                Bytes32::encode_second(amount_out128),
                Bytes32::encode_first(fee128),
            )
        } else {
            (
                Bytes32::encode_second(amount_in128),
                Bytes32::encode_first(amount_out128),
                Bytes32::encode_second(fee128),
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
    pub fn received(
        reserves: Bytes32,
        amount_received_x: Uint128,
        amount_received_y: Uint128,
    ) -> Bytes32 {
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
                    let msg = HandleMsg::Send {
                        recipient: recipient.to_string(),
                        amount,
                        padding: None,
                        msg: None,
                        recipient_code_hash: None,
                        memo: None,
                    };
                    // //TODO add token hash
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
                    // //TODO add token hash
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
    use cosmwasm_std::StdResult;
    use ethnum::U256;

    use crate::utils::liquidity_book::math::packed_u128_math::{Decode, Encode};

    use super::BinHelper;

    #[test]
    fn test_share() -> StdResult<()> {
        let bin_reserves = Encode::encode(10000, 10000);
        let amount_in = Encode::encode(1000, 1000);
        let price = U256::from(100u128);
        let total_supply = U256::from(100u128);

        let ((shares, effective_amounts_in)) = BinHelper::get_shares_and_effective_amounts_in(
            bin_reserves,
            amount_in,
            price,
            total_supply,
        )
        .unwrap();



        Ok(())
    }
}
