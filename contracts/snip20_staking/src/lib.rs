mod batch;
pub mod contract;
mod distributors;
mod expose_balance;
pub mod msg;
mod rand;
pub mod receiver;
mod stake;
mod stake_queries;
pub mod state;
mod state_staking;
mod transaction_history;
mod utils;
mod viewing_key;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::contract;
    use shade_protocol::c_std::{
        do_handle,
        do_init,
        do_query,
        ExternalApi,
        ExternalQuerier,
        ExternalStorage,
    };

    #[no_mangle]
    extern "C" fn instantiate(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_init(
            &contract::instantiate::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn execute(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_handle(
            &contract::execute::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn query(msg_ptr: u32) -> u32 {
        do_query(
            &contract::query::<ExternalStorage, ExternalApi, ExternalQuerier>,
            msg_ptr,
        )
    }

    // Other C externs like cosmwasm_vm_version_1, allocate, deallocate are available
    // automatically because we `use cosmwasm_std`.
}
