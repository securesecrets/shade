/// Used for creates a struct that implements the MultiTestable interface.
/// 
/// Needs the implementing package to have shade_protocol as a dependency with features.
/// 
/// First arg is the struct name that will implement the MultiTestable interface.
/// 
/// Second is the name of the package containing the contract module itself.
#[macro_export]
macro_rules! implement_multi {
    ($x:ident, $s:ident) => {
        use shade_protocol::c_std::{ContractInfo, Empty, Env, Addr};
        use shade_protocol::multi_test::{Contract, ContractWrapper};
        use shade_protocol::utils::callback::MultiTestable;

        pub struct $x { info: ContractInfo }
        
        impl MultiTestable for $x {
            fn contract(&self) -> Box<dyn Contract<Empty>> {
                let contract = ContractWrapper::new_with_empty(
                    $s::contract::execute, 
                    $s::contract::instantiate, 
                    $s::contract::query
                );
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

/// Used for creates a struct that implements the MultiTestable interface **(for contracts that implement the reply method)**
/// 
/// Needs the implementing package to have shade_protocol as a dependency with features.
/// 
/// First arg is the struct name that will implement the MultiTestable interface.
/// 
/// Second is the name of the package containing the contract module itself.
#[macro_export]
macro_rules! implement_multi_with_reply {
    ($x:ident, $s:ident) => {
        use shade_protocol::c_std::{ContractInfo, Empty, Env, Addr};
        use shade_protocol::multi_test::{Contract, ContractWrapper};
        use shade_protocol::utils::callback::MultiTestable;

        pub struct $x { info: ContractInfo }
        
        impl MultiTestable for $x {
            fn contract(&self) -> Box<dyn Contract<Empty>> {
                let contract = ContractWrapper::new_with_empty(
                    $s::contract::execute, 
                    $s::contract::instantiate, 
                    $s::contract::query
                ).with_reply($s::contract::reply);
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
