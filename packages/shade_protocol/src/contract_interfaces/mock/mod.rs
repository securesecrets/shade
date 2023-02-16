use crate::{
    c_std::{
        Addr,
        Binary,
        Uint128,
    },
    contract_interfaces::dex::sienna::Pair,
    cosmwasm_schema::cw_serde,
    utils::{
        asset::Contract, ExecuteCallback, InstantiateCallback,
        storage::plus::{Item, ItemStorage},
    },
};

pub mod mock_sienna {
    use super::*;

    #[cw_serde]
    pub struct InstantiateMsg {}

    impl InstantiateCallback for InstantiateMsg {
        const BLOCK_SIZE: usize = 256;
    }

    #[cw_serde]
    pub struct PairInfo {
        pub pair: Pair,
        pub amount_0: Uint128,
        pub amount_1: Uint128,
    }

    impl ItemStorage for PairInfo {
        const ITEM: Item<'static, Self> = Item::new("item-pair_info");
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        MockPool {
            token_a: Contract,
            amount_a: Uint128,
            token_b: Contract,
            amount_b: Uint128,
        },
        // SNIP20 receiver interface
        Receive {
            sender: Addr,
            from: Addr,
            msg: Option<Binary>,
            amount: Uint128,
        },   
    }

    impl ExecuteCallback for ExecuteMsg {
        const BLOCK_SIZE: usize = 256;
    }

    #[cw_serde]
    pub enum ReceiverCallbackMsg {
        Swap {
            expected_return: Option<Uint128>,
            to: Option<Addr>,
        },
    }

}
