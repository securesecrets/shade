#[cfg(not(target_arch = "wasm32"))]
pub mod multi;
#[cfg(not(target_arch = "wasm32"))]
pub mod multi_macro;