#![allow(clippy::clone_double_ref)]
#![allow(clippy::too_many_arguments)]

#[cfg(not(target_arch = "wasm32"))]
pub mod multi;

pub mod interfaces;
