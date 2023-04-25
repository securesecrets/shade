use std::convert::TryInto;
use shade_protocol::{
	c_std::{
        Addr, 
        Deps,
        StdError, 
        StdResult,
        QuerierWrapper,
        Uint128,
    },
	contract_interfaces::{
        dao::adapter,
        sky::{
            cycles::ArbPair,
            sky_derivatives::{
                Config,
                Direction,
                DexPairs,
                QueryAnswer,
                SelfAddr,
                SwapAmounts,
                TreasuryUnbondings,
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
// Where dex_pools.0 is the base token pool and dex_pools.1 is the derivative pool
//
// If either of these values are positive (they should never both be positive) there is a
// profitable trade in that direction
pub fn optimization_math(
    dex_pools: (Float, Float),
    derivative_price: Float,
    unbond_rate: Float,
    stake_rate: Float,
    dex_rate: Float,
    max_swap: Float,
) -> StdResult<Option<OptimizationResult>> {
    // Float used here for easy math
    // Checked math not used because of the absurd range of Float
    let common_radical = dex_pools.0 * dex_pools.1 * dex_rate;
	let unbond_optimal_amount = (common_radical * derivative_price * unbond_rate)
                                    .sqrt()
                                    .checked_sub(dex_pools.0);
	match unbond_optimal_amount {
		Ok(amount) => {
            let swap_amount = Float::min(amount, max_swap);
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
            let swap_amount = Float::min(optimal_amount, max_swap);
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

// Helper function for determining profitability
pub fn is_arb_profitable(
    querier: &QuerierWrapper,
    config: &Config,
    dex_pair: &ArbPair,
    max_swap: Uint128,
) -> StdResult<QueryAnswer> {
    let dex_pools = dex_pair.query_pool_amounts(querier)?;
    
    let derivative_price = Float::from(config.derivative.query_exchange_price(querier)?)
        .shift_decimal(u128::from(dex_pair.token0_decimals) as i32 - 
                       u128::from(dex_pair.token1_decimals) as i32)?;
    let max_swap = Float::from(max_swap);
    let min_profit = config.min_profit_amount;

    // Subtracts will not overflow if trading fees are properly checked
    let unbond_rate = Float::one() - Float::from(config.trading_fees.unbond_fee);
    let stake_rate = Float::one() - Float::from(config.trading_fees.stake_fee);
    let dex_rate = Float::one() - Float::from(config.trading_fees.dex_fee);

    let dex_pools_float = (Float::from(dex_pools.0), Float::from(dex_pools.1));

    // Actual calculate optimal amount
    let opt_res = optimization_math(
        dex_pools_float, 
        derivative_price, 
        unbond_rate, 
        stake_rate, 
        dex_rate, 
        max_swap,
    );

    // Convert back to Uint types
    match opt_res? {
        Some(optimization) => {
            let swap_amounts = SwapAmounts {
                optimal_swap: optimization.optimal_swap.try_into()?,
                swap1_result: optimization.swap1_result.try_into()?,
                swap2_result: optimization.swap2_result.try_into()?,
            };
            
            // Check if profit is significant
            let mut is_profitable = true;
            if swap_amounts.swap2_result - swap_amounts.optimal_swap < min_profit {
                is_profitable = false;
            }

            Ok(QueryAnswer::IsProfitable {
                is_profitable,
                swap_amounts: Some(swap_amounts),
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
 
pub fn is_profitable(
    deps: Deps,
    pair_index: usize,
) -> StdResult<QueryAnswer> {
    let dex_pairs = DexPairs::load(deps.storage)?.0;
    if pair_index >= dex_pairs.len() {
        return Err(StdError::generic_err(format!("Invalid dex_pair index: {}", pair_index)));
    }

    let config = Config::load(deps.storage)?;
    let mut dex_pair = dex_pairs[pair_index].clone();

    is_arb_profitable(&deps.querier, &config, &mut dex_pair, config.max_arb_amount)
}

pub fn is_any_pair_profitable(
    deps: Deps,
) -> StdResult<QueryAnswer> {
    let pairs = DexPairs::load(deps.storage)?.0;
    if pairs.len() == 0 {
        return Err(StdError::generic_err("No dex pairs to arb!"));
    }

    let mut is_profitable_vec = vec![];
    let mut swap_amounts_vec = vec![];
    let mut direction_vec = vec![];
    let config = Config::load(deps.storage)?;
    for mut dex_pair in pairs {
        match is_arb_profitable(&deps.querier, &config, &mut dex_pair, config.max_arb_amount)? {
            QueryAnswer::IsProfitable { is_profitable, swap_amounts, direction } => {
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
    let config = Config::load(deps.storage)?;
    let derivative = config.derivative;

    // Only relevant token sky staking holds
    if asset != derivative.base_asset.address {
        return Ok(adapter::QueryAnswer::Balance {
            amount: Uint128::zero(),
        })
    }

    let self_addr = SelfAddr::load(deps.storage)?.0;
    let balance = derivative.query_base_balance(
        &deps.querier, 
        self_addr.clone(), 
        config.viewing_key.clone(),
    )?;
    let unbondings = derivative.query_unbondings(
        &deps.querier, 
        self_addr.clone(), 
        config.viewing_key.clone(),
    )?;
    let price = derivative.query_exchange_price(&deps.querier)?;

    // TODO: when available checked math for multiplying Uint128 and Decimal
    let mut derivative_value = unbondings * price;
    if derivative.deriv_decimals != derivative.base_decimals { // Adjust to match base decimals
        if derivative.deriv_decimals > derivative.base_decimals {
            derivative_value *= Uint128::new(10)
                .pow(derivative.deriv_decimals - derivative.base_decimals);
        } else {
            derivative_value *= Uint128::new(10)
                .pow(derivative.base_decimals - derivative.deriv_decimals);
        }
    }

    Ok(adapter::QueryAnswer::Balance {
        amount: balance.checked_add(derivative_value)?,
    })
}

pub fn adapter_claimable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(deps.storage)?;

    // Only relevant token sky staking holds
    if asset != config.derivative.base_asset.address {
        return Ok(adapter::QueryAnswer::Claimable {
            amount: Uint128::zero(),
        })
    }

    // Balance up to unbondings is claimable
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let balance = config.derivative.query_base_balance(
        &deps.querier, 
        self_addr, 
        config.viewing_key
    )?;

    let unbondings = TreasuryUnbondings::load(deps.storage)?.0;
    let claimable = Uint128::min(balance, unbondings);

    Ok(adapter::QueryAnswer::Claimable {
        amount: claimable,
    })
}

pub fn adapter_unbonding(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let derivative = Config::load(deps.storage)?.derivative;

    // Only relevant token sky staking holds
    if asset != derivative.base_asset.address {
        return Ok(adapter::QueryAnswer::Balance {
            amount: Uint128::zero(),
        })
    }

    Ok(adapter::QueryAnswer::Unbonding {
        amount: TreasuryUnbondings::load(deps.storage)?.0,
    })
}

pub fn adapter_unbondable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    // Whole balance is unbondable
    if let adapter::QueryAnswer::Balance { amount } = adapter_balance(deps, asset)? {
        Ok(adapter::QueryAnswer::Unbondable {
            amount,
        })
    } else {
        Err(StdError::generic_err("This should not happen!"))
    }
}

pub fn adapter_reserves(_deps: Deps, _asset: Addr) -> StdResult<adapter::QueryAnswer> {
    // Sky Staking has no reserves
    Ok(adapter::QueryAnswer::Reserves {
        amount: Uint128::zero(),
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
                Float::from(u128::MAX),
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
                Float::from(u128::MAX),
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
                Float::from(u128::MAX),
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
