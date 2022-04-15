#[cfg(test)]
pub mod tests {
    use crate::handle::{calculate_shares, calculate_tokens, stake_weight};
    use binary_heap_plus::{BinaryHeap, MinComparator};
    use cosmwasm_math_compat::Uint128;
    use shade_protocol::staking::stake::{Stake, Unbonding, UserStake};

    #[test]
    fn test_weight_calculation() {
        let stake = Uint128::new(1000000u128);

        assert_eq!(Uint128::new(500000u128), stake_weight(stake, 50));
        assert_eq!(Uint128::new(250000u128), stake_weight(stake, 25));
    }

    #[test]
    fn binary_heap_order() {
        let mut unbonding_heap: BinaryHeap<Unbonding, MinComparator> = BinaryHeap::new_min();

        // Add the three values in a non order fashion
        let val1 = Unbonding {
            amount: Default::default(),
            unbond_time: 0,
        };
        let val2 = Unbonding {
            amount: Default::default(),
            unbond_time: 1,
        };
        let val3 = Unbonding {
            amount: Default::default(),
            unbond_time: 2,
        };

        unbonding_heap.push(val2);
        unbonding_heap.push(val1);
        unbonding_heap.push(val3);

        assert_eq!(0, unbonding_heap.pop().unwrap().unbond_time);
        assert_eq!(1, unbonding_heap.pop().unwrap().unbond_time);
        assert_eq!(2, unbonding_heap.pop().unwrap().unbond_time);
    }

    fn init_user() -> UserStake {
        UserStake {
            shares: Uint128::zero(),
            tokens_staked: Uint128::zero(),
        }
    }

    fn stake(state: &mut Stake, user: &mut UserStake, amount: Uint128) -> Uint128 {
        let shares = calculate_shares(amount, state);
        state.total_tokens += amount;
        state.total_shares += shares;
        user.tokens_staked += amount;
        user.shares += shares;

        shares
    }

    fn unbond(state: &mut Stake, user: &mut UserStake, amount: Uint128) -> Uint128 {
        let shares = calculate_shares(amount, state);
        state.total_tokens = state.total_tokens - amount;
        state.total_shares = state.total_shares - shares;
        user.tokens_staked = user.tokens_staked - amount;
        user.shares = user.shares - shares;

        shares
    }

    #[test]
    fn standard_staking() {
        let mut state = Stake {
            total_shares: Uint128::zero(),
            total_tokens: Uint128::zero(),
        };

        // User 1 stakes 100
        let mut u1 = init_user();
        let u1_stake = Uint128::new(100u128);
        stake(&mut state, &mut u1, u1_stake);

        assert_eq!(u1_stake, calculate_tokens(u1.shares, &state));

        // User 2 stakes 50
        let mut u2 = init_user();
        let u2_stake = Uint128::new(50u128);
        stake(&mut state, &mut u2, u2_stake);

        assert_eq!(u1_stake, calculate_tokens(u1.shares, &state));
        assert_eq!(u2_stake, calculate_tokens(u2.shares, &state));

        // User 3 stakes 35
        let mut u3 = init_user();
        let u3_stake = Uint128::new(35u128);
        stake(&mut state, &mut u3, u3_stake);

        assert_eq!(u1_stake, calculate_tokens(u1.shares, &state));
        assert_eq!(u2_stake, calculate_tokens(u2.shares, &state));
        assert_eq!(u3_stake, calculate_tokens(u3.shares, &state));
    }

    #[test]
    fn unbonding() {
        let mut state = Stake {
            total_shares: Uint128::zero(),
            total_tokens: Uint128::zero(),
        };

        // User 1 stakes 100
        let mut u1 = init_user();
        let u1_stake = Uint128::new(100u128);
        stake(&mut state, &mut u1, u1_stake);

        // User 2 stakes 50
        let mut u2 = init_user();
        let u2_stake = Uint128::new(50u128);
        stake(&mut state, &mut u2, u2_stake);

        // User 3 stakes 35
        let mut u3 = init_user();
        let u3_stake = Uint128::new(35u128);
        stake(&mut state, &mut u3, u3_stake);

        // User 2 unbonds 25
        let u2_unbond = Uint128::new(25u128);
        unbond(&mut state, &mut u2, u2_unbond);

        assert_eq!(u1_stake, calculate_tokens(u1.shares, &state));
        assert_eq!(u2_stake - u2_unbond, calculate_tokens(u2.shares, &state));
        assert_eq!(u3_stake, calculate_tokens(u3.shares, &state));
    }

    #[test]
    fn rewards_distribution() {
        let mut state = Stake {
            total_shares: Uint128::zero(),
            total_tokens: Uint128::zero(),
        };

        // User 1 stakes 100
        let mut u1 = init_user();
        let u1_stake = Uint128::new(100u128);
        stake(&mut state, &mut u1, u1_stake);

        // User 2 stakes 50
        let mut u2 = init_user();
        let u2_stake = Uint128::new(50u128);
        stake(&mut state, &mut u2, u2_stake);

        // User 3 stakes 50
        let mut u3 = init_user();
        let u3_stake = Uint128::new(50u128);
        stake(&mut state, &mut u3, u3_stake);

        // Add a 200 reward, (should double user amounts)
        state.total_tokens += Uint128::new(200u128);

        assert_eq!(
            u1_stake.multiply_ratio(Uint128::new(2u128), Uint128::new(1u128)),
            calculate_tokens(u1.shares, &state)
        );
        assert_eq!(
            u2_stake.multiply_ratio(Uint128::new(2u128), Uint128::new(1u128)),
            calculate_tokens(u2.shares, &state)
        );
        assert_eq!(
            u3_stake.multiply_ratio(Uint128::new(2u128), Uint128::new(1u128)),
            calculate_tokens(u3.shares, &state)
        );
    }
}
