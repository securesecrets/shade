// use cosmwasm_std::{
//     StdError,
//     StdResult,
//     Storage, Response, MessageInfo, Addr
// };
// use cosmwasm_storage::{singleton, Singleton, singleton_read, ReadonlySingleton};

// pub static ADMIN: &[u8] =b"contract_pair_admin";

// pub fn admin_w(storage: &mut dyn Storage) -> Singleton<Addr> {
//     singleton(storage, ADMIN)
// }

// pub fn admin_r(storage: & dyn Storage) -> ReadonlySingleton<Addr> {
//     singleton_read(storage, ADMIN)
// }

// // pub fn apply_admin_guard(
// //     caller: &Addr,
// //     storage: &mut dyn Storage,
// // ) -> StdResult<bool> {
// //     let admin_address = admin_r(storage).load()?;
// //     if caller != &admin_address {
// //          return Err(StdError::generic_err("Caller is not admin"))
// //     }
// //     return Ok(true)
// // }

// pub fn set_admin_guard(
//     storage: &mut dyn Storage,
//     info: MessageInfo,
//     admin: Addr
// ) -> StdResult<Response>{
//     apply_admin_guard(&info.sender, storage)?;
//     admin_w(storage).save(&admin)?;
//     Ok(Response::default())
// }
