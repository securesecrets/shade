#[cfg(test)]
pub mod tests {
    use binary_heap_plus::{BinaryHeap, MinComparator};
    use shade_protocol::staking::{StakeState, Unbonding, UserStakeState};
    use cosmwasm_std::{Decimal, Uint128};
    use crate::handle::{calculate_shares, calculate_tokens, stake_weight};

    #[test]
    fn test_weight_calculation() {
        let stake = Uint128(1000000);

        assert_eq!(Uint128(500000), stake_weight(stake, 50));
        assert_eq!(Uint128(250000), stake_weight(stake, 25));
    }

    #[test]
    fn binary_heap_order() {
        let mut unbonding_heap: BinaryHeap<Unbonding, MinComparator> = BinaryHeap::new_min();

        // Add the three values in a non order fashion
        let val1 = Unbonding {
            account: Default::default(),
            amount: Default::default(),
            unbond_time: 0
        };
        let val2 = Unbonding {
            account: Default::default(),
            amount: Default::default(),
            unbond_time: 1
        };
        let val3 = Unbonding {
            account: Default::default(),
            amount: Default::default(),
            unbond_time: 2
        };

        unbonding_heap.push(val2);
        unbonding_heap.push(val1);
        unbonding_heap.push(val3);

        assert_eq!(0, unbonding_heap.pop().unwrap().unbond_time);
        assert_eq!(1, unbonding_heap.pop().unwrap().unbond_time);
        assert_eq!(2, unbonding_heap.pop().unwrap().unbond_time);
    }

    fn init_user() -> UserStakeState {
        UserStakeState {
            shares: Uint128::zero(),
            tokens_staked: Uint128::zero()
        }
    }

    fn stake(state: &mut StakeState, user: &mut UserStakeState, amount: Uint128) -> Uint128 {
        let shares = calculate_shares(amount, state);
        state.total_tokens += amount;
        state.total_shares += shares;
        user.tokens_staked += amount;
        user.shares += shares;

        shares
    }

    fn unbond(state: &mut StakeState, user: &mut UserStakeState, amount: Uint128) -> Uint128 {
        let shares = calculate_shares(amount, state);
        state.total_tokens = (state.total_tokens - amount).unwrap();
        state.total_shares = (state.total_shares - shares).unwrap();
        user.tokens_staked = (user.tokens_staked - amount).unwrap();
        user.shares = (user.shares - shares).unwrap();

        shares
    }

    #[test]
    fn standard_staking() {
        let mut state = StakeState {
            total_shares: Uint128::zero(),
            total_tokens: Uint128::zero()
        };

        // User 1 stakes 100
        let mut u1 = init_user();
        let u1_stake = Uint128(100);
        stake(&mut state, &mut u1, u1_stake);

        assert_eq!(u1_stake, calculate_tokens(u1.shares, &state));

        // User 2 stakes 50
        let mut u2 = init_user();
        let u2_stake = Uint128(50);
        stake(&mut state, &mut u2, u2_stake);

        assert_eq!(u1_stake, calculate_tokens(u1.shares, &state));
        assert_eq!(u2_stake, calculate_tokens(u2.shares, &state));

        // User 3 stakes 35
        let mut u3 = init_user();
        let u3_stake = Uint128(35);
        stake(&mut state, &mut u3, u3_stake);

        assert_eq!(u1_stake, calculate_tokens(u1.shares, &state));
        assert_eq!(u2_stake, calculate_tokens(u2.shares, &state));
        assert_eq!(u3_stake, calculate_tokens(u3.shares, &state));
    }

    #[test]
    fn unbonding() {
        let mut state = StakeState {
            total_shares: Uint128::zero(),
            total_tokens: Uint128::zero()
        };

        // User 1 stakes 100
        let mut u1 = init_user();
        let u1_stake = Uint128(100);
        stake(&mut state, &mut u1, u1_stake);

        // User 2 stakes 50
        let mut u2 = init_user();
        let u2_stake = Uint128(50);
        stake(&mut state, &mut u2, u2_stake);

        // User 3 stakes 35
        let mut u3 = init_user();
        let u3_stake = Uint128(35);
        stake(&mut state, &mut u3, u3_stake);

        // User 2 unbonds 25
        let u2_unbond = Uint128(25);
        unbond(&mut state, &mut u2, u2_unbond);

        assert_eq!(u1_stake, calculate_tokens(u1.shares, &state));
        assert_eq!((u2_stake - u2_unbond).unwrap(), calculate_tokens(u2.shares, &state));
        assert_eq!(u3_stake, calculate_tokens(u3.shares, &state));

    }

    #[test]
    fn rewards_distribution() {
        let mut state = StakeState {
            total_shares: Uint128::zero(),
            total_tokens: Uint128::zero()
        };

        // User 1 stakes 100
        let mut u1 = init_user();
        let u1_stake = Uint128(100);
        stake(&mut state, &mut u1, u1_stake);

        // User 2 stakes 50
        let mut u2 = init_user();
        let u2_stake = Uint128(50);
        stake(&mut state, &mut u2, u2_stake);

        // User 3 stakes 50
        let mut u3 = init_user();
        let u3_stake = Uint128(50);
        stake(&mut state, &mut u3, u3_stake);

        // Add a 200 reward, (should double user amounts)
        state.total_tokens += Uint128(200);

        assert_eq!(u1_stake.multiply_ratio(Uint128(2), Uint128(1)), calculate_tokens(u1.shares, &state));
        assert_eq!(u2_stake.multiply_ratio(Uint128(2), Uint128(1)), calculate_tokens(u2.shares, &state));
        assert_eq!(u3_stake.multiply_ratio(Uint128(2), Uint128(1)), calculate_tokens(u3.shares, &state));
    }

}