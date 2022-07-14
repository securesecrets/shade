#[macro_export]
macro_rules! implement_multi {
    ($x:ident, $s:ident) => {
        use shade_protocol::c_std::{from_binary, ContractInfo, Empty, Coin, Addr, Binary, Env, Response, StdResult};
        use shade_protocol::multi_test::{Executor, Contract, ContractWrapper, App};
        use shade_protocol::serde::Serialize;
        use shade_multi_test::MultiTestable;
        impl MultiTestable for $x {
            fn get_info(&self) -> &ContractInfo {
                &self.info
            }
        
            fn contract() -> Box<dyn Contract<Empty>> {
                let contract = ContractWrapper::new_with_empty($s::contract::execute, $s::contract::instantiate, $s::contract::query);
                Box::new(contract)
            }
        
            fn new(info: ContractInfo) -> Self {
                $x { info }
            }
            fn init<T: Serialize>(
                router: &mut App,
                sender: Addr,
                label: &str,
                send_funds: &[Coin],
                msg: &T,
            ) -> ContractInfo {
                let stored_code = router.store_code($x::contract());
                router
                    .instantiate_contract(stored_code, sender, &msg, send_funds, label, None)
                    .unwrap()
            }
        }
    };
}

pub use implement_multi;
