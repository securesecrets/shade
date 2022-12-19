use std::{
    ops::*,
    convert::{TryFrom, TryInto},
};
use shade_protocol::{
	c_std::{
        Addr, 
        Decimal,
        Deps,
        Isqrt,
        StdError, 
        StdResult,
        Uint128,
        Uint256,
    },
	contract_interfaces::{
        dao::adapter,
        sky::{
            cycles::{
                ArbPair,
                Derivative,
            },
            sky_derivatives::{
                Config,
		        Direction,
                DexPairs,
		        QueryAnswer,
                Rollover,
                SwapAmounts,
            },
        },
	},
    utils::storage::plus::ItemStorage,
};
use cosmwasm_floating_point::float::Float;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(deps.storage)?,
    })
}

pub fn dex_pairs(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::DexPairs {
        dex_pairs: DexPairs::load(deps.storage)?.0,
    })
}

pub fn current_rollover(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::CurrentRollover {
        rollover: Rollover::load(deps.storage)?.0,
    })
}
 
pub fn is_profitable(
    deps: Deps,
    pair_index: usize,
    max_swap: Option<Uint128>,
) -> StdResult<QueryAnswer> {
    let dex_pairs = DexPairs::load(deps.storage)?.0;
    if pair_index >= dex_pairs.len() {
        return Err(StdError::generic_err(format!("Invalid dex_pair index: {}", pair_index)));
    }

    let config = Config::load(deps.storage)?;
    let mut dex_pair = dex_pairs[pair_index].clone();
    let dex_pools = dex_pair.pool_amounts(deps)?;
    
    let derivative_price = Float::from(config.derivative.query_exchange_price(deps)?);
    let max_swap = max_swap.and_then(|max| Some(Float::from(max)));

    // Subtracts will not overflow if trading fees are properly checked
    let unbond_rate: Float = Float::from(Decimal::one() - config.trading_fees.unbond_fee);
    let stake_rate: Float = Float::from(Decimal::one() - config.trading_fees.stake_fee);
    let dex_rate: Float = Float::from(Decimal::one() - config.trading_fees.dex_fee);

    let dex_pools_float: (Float, Float) = (
        Float::from(dex_pools.0)
            .shift_decimal(-(u128::from(dex_pair.token0_decimals) as i32))?,
        Float::from(dex_pools.1)
            .shift_decimal(-(u128::from(dex_pair.token1_decimals) as i32))?,
    );

    // Actual calculate optimal amount
    let opt_res = optimization_math(
        dex_pools_float, 
        derivative_price, 
        unbond_rate, 
        stake_rate, 
        dex_rate, 
        max_swap
    );

    // Convert back to Uint types
    match opt_res? {
        Some(optimization) => {
            Ok(QueryAnswer::IsProfitable {
                is_profitable: true,
                swap_amounts: Some(SwapAmounts {
                    optimal_swap: optimization.optimal_swap
                        .shift_decimal(u128::from(dex_pair.token0_decimals) as i32)?
                        .try_into()?,
                    swap1_result: optimization.swap1_result
                        .shift_decimal(u128::from(dex_pair.token0_decimals) as i32)?
                        .try_into()?,
                    swap2_result: optimization.swap2_result
                        .shift_decimal(u128::from(dex_pair.token0_decimals) as i32)?
                        .try_into()?,
                }),
                direction: Some(optimization.direction),
            })
        },
        None => {
            Ok(QueryAnswer::IsProfitable {
                is_profitable: false,
                swap_amounts: None,
                direction: None,
            })
        },
    }
}

#[derive(Debug, PartialEq)]
pub struct OptimizationResult {
    direction: Direction,
    optimal_swap: Float,
    swap1_result: Float,
    swap2_result: Float,
}

// Calculate optimal amounts for arbitrage, equations obtained by finding the zero of the
// derivative of the constant product equation for the two exchange operations:
// 
//     unbond_optimal_amount = sqrt(dex_pools.0 * dex_pools.1 * derivative_price * dex_rate *
//                                  unbond_rate) - dex_pools.0
//     stake_optimal_amount  = (derivative_price / stake_rate) * (sqrt(dex_pools.0 * dex_pools.1 *
//                                  dex_rate * stake_rate / stake_price) - dex_pools.1)
// 
// Where unbond means: buy on dex, then start derivative unbond
//    and stake means: mint derivative, then sell on dex
// If either of these values are positive (they should never both be positive) there is a
// profitable trade in that direction
pub fn optimization_math(
    dex_pools: (Float, Float),
    derivative_price: Float,
    unbond_rate: Float,
    stake_rate: Float,
    dex_rate: Float,
    max_swap: Option<Float>,
) -> StdResult<Option<OptimizationResult>> {
    // Float used here for easy math
    // Checked math not used because of the absurd range of Float
    let common_radical = dex_pools.0 * dex_pools.1 * dex_rate;
	let unbond_optimal_amount = (common_radical * derivative_price * unbond_rate)
                                    .sqrt()
                                    .checked_sub(dex_pools.0);
	match unbond_optimal_amount {
		Ok(amount) => {
            let swap_amount = match max_swap {
                Some(max) => Float::max(amount, max),
                None => amount,
            };
            // derivative resulting from dex swap
            let expected_return_1 = cp_result(
                                        swap_amount,
                                        dex_pools.0, 
                                        dex_pools.1,
                                        dex_rate,
                                    )?;
            // base currency resulting from unbond
            let expected_return_2 = expected_return_1 * derivative_price * unbond_rate;
			return Ok(Some(OptimizationResult {
                direction: Direction::Unbond,
                optimal_swap: swap_amount,
                swap1_result: expected_return_1,
                swap2_result: expected_return_2,
			}))
		},
		Err(_err) => { }, // unbond optimal amount negative, not profitable here
	};

	let stake_optimal_inner = (common_radical * stake_rate / derivative_price)
                                    .sqrt()
                                    .checked_sub(dex_pools.1);
	match stake_optimal_inner {
		Ok(amount) => {
			let optimal_amount = derivative_price / stake_rate * amount;
            let swap_amount = match max_swap {
               Some(max) => Float::max(optimal_amount, max),
               None => optimal_amount,
            };
            
            // derivative resulting from derivative mint/stake
            let expected_return_1 = swap_amount / derivative_price * stake_rate;
            // base currency resulting from dex swap
            let expected_return_2 = cp_result(
                                        expected_return_1, 
                                        dex_pools.1, 
                                        dex_pools.0, 
                                        dex_rate
                                    )?;
			Ok(Some(OptimizationResult {
                direction: Direction::Stake,
                optimal_swap: swap_amount,
                swap1_result: expected_return_1,
                swap2_result: expected_return_2,
            }))
		},
		Err(_err) => Ok(None) // mint optimal amount negative,
	}
}

pub fn is_any_pair_profitable(
    deps: Deps,
    max_swap: Option<Uint128>,
) -> StdResult<QueryAnswer> {
    let pairs = DexPairs::load(deps.storage)?.0;
    if pairs.len() == 0 {
        return Err(StdError::generic_err("No dex pairs to arb!"));
    }

    let mut is_profitable_vec = vec![];
    let mut swap_amounts_vec = vec![];
    let mut direction_vec = vec![];
    for index in 0..pairs.len() {
        match is_profitable(deps, index, max_swap)? {
            QueryAnswer::IsProfitable { is_profitable, swap_amounts, direction} => {
                is_profitable_vec.push(is_profitable);
                swap_amounts_vec.push(swap_amounts);
                direction_vec.push(direction);
            },
            _ => {
                return Err(StdError::generic_err("Unexpected query answer")); // This shouln't happen
            }
        };
    }
    
    Ok(QueryAnswer::IsAnyPairProfitable {
        is_profitable: is_profitable_vec,
        swap_amounts: swap_amounts_vec,
        direction: direction_vec,
    })
}

pub fn adapter_balance(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {

    // TODO

    Ok(adapter::QueryAnswer::Balance {
        amount: shade_protocol::c_std::Uint128::zero(),
    })
}

pub fn adapter_claimable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {

    // TODO

    Ok(adapter::QueryAnswer::Claimable {
        amount: shade_protocol::c_std::Uint128::zero(),
    })
}

pub fn adapter_unbonding(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {

    // TODO

    Ok(adapter::QueryAnswer::Unbonding {
        amount: shade_protocol::c_std::Uint128::zero(),
    })
}

pub fn adapter_unbondable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {

    // TODO

    Ok(adapter::QueryAnswer::Unbondable {
        amount: shade_protocol::c_std::Uint128::zero(),
    })
}

pub fn adapter_reserves(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {

    // TODO

    Ok(adapter::QueryAnswer::Reserves {
        amount: shade_protocol::c_std::Uint128::zero(),
    })
}


/// Constant Product Rule similator
fn cp_result(
    amount: Float, 
    pool_1: Float, 
    pool_2: Float, 
    swap_fee: Float
) -> StdResult<Float> {
    let expected_res = pool_2 - (pool_1 * pool_2) / (pool_1 + amount);
    Ok(expected_res * swap_fee)
}


#[cfg(test)]
mod tests {
    use crate::query;
    use cosmwasm_floating_point::float::Float; 
    use shade_protocol::contract_interfaces::sky::sky_derivatives::Direction;

    #[test]
    fn optimization_math() {
        // Result from Wolfram Alpha:
        //     optimal_swap: 3602.9945957676956911787695567953165463581381660696721658187575524
        //     swap1_result: 3345.9161627365989274204451376517893115307531970706239652492688967
        //     swap2_result: 3615.1269042323043088212304432046834536418618339303278341812424474 
        // Results below are within ~10^-13% of each other, due to Float round-off
        // Which is more than good enough for this type of operation
        assert_eq!(
            query::optimization_math(
                (Float::from(1_070_000u32), Float::from(1_000_000u32)),
                Float::from_float(1.081),
                Float::from_float(0.9995),
                Float::from_float(0.998),
                Float::from_float(0.997),
                None,
            ).unwrap().unwrap(),
            query::OptimizationResult {
                direction: Direction::Unbond,
                optimal_swap: Float::new(3_602_994_595_767_695_000, -15).unwrap(),
                swap1_result: Float::new(3_345_916_162_736_598_468, -15).unwrap(),
                swap2_result: Float::new(3_615_126_904_232_303_811, -15).unwrap(),
            },
        );

        // Same thing here, answers are correct to ~15 significant digits
        assert_eq!(
            query::optimization_math(
                (Float::from(1_087_500u128), Float::from(1_000_000u128)),
                Float::from_float(1.081),
                Float::from_float(0.9995),
                Float::from_float(0.998),
                Float::from_float(0.997),
                None,
            ).unwrap().unwrap(),
            query::OptimizationResult {
                direction: Direction::Stake,
                optimal_swap: Float::new(5_354_513_201_098_882_984, -16).unwrap(),
                swap1_result: Float::new(4_943_389_615_815_619_997, -16).unwrap(),
                swap2_result: Float::new(5_357_160_145_594_486_580, -16).unwrap(),
            },
        );

        // Market conditions on 12/19/22
        assert_eq!(
            query::optimization_math(
                (Float::from(389168442081u128).shift_decimal(-6).unwrap(),
                 Float::from(362059840107u128).shift_decimal(-6).unwrap()),
                Float::from_float(1.183813),
                Float::from_float(0.9995),
                Float::from_float(0.998),
                Float::from_float(0.997),
                None,
            ).unwrap().unwrap(),
            query::OptimizationResult {
                direction: Direction::Unbond,
                optimal_swap: Float::decimal(1_853_043_600_579_028_130, 4).unwrap(),
                swap1_result: Float::decimal(1_640_671_504_583_677_894, 4).unwrap(),
                swap2_result: Float::decimal(1_941_277_131_727_789_619, 4).unwrap(),
            },
        );
    }
}
