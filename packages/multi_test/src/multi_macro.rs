#[macro_export]
macro_rules! implement_multi {
    ($x:ident, $s:ident) => {
        use shade_protocol::c_std::{ContractInfo, Empty, Env, Addr};
        use shade_protocol::multi_test::{Contract, ContractWrapper};
        use shade_protocol::utils::callback::MultiTestable;
        impl MultiTestable for $x {
            fn contract(&self) -> Box<dyn Contract<Empty>> {
                let contract = ContractWrapper::new_with_empty($s::contract::execute, $s::contract::instantiate, $s::contract::query);
                Box::new(contract)
            }

            fn default() -> Self {
                let info = ContractInfo {
                    address: Addr::unchecked(""),
                    code_hash: String::default(),
                };
                $x { info }
            }
         }
    };
}

pub use implement_multi;
