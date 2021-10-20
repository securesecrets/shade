#[cfg(test)]
pub mod tests {
    use binary_heap_plus::{BinaryHeap, MinComparator};
    use shade_protocol::staking::Unbonding;
    use cosmwasm_std::Uint128;
    use crate::handle::stake_weight;

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
}