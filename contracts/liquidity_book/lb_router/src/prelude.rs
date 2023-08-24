// Use this crate's custom Error type
pub use crate::error::LBRouterError as Error;

// Force all Result types to use our Error type
pub type Result<T> = core::result::Result<T, Error>;

// Generic Wrapper tuple struct for newtype pattern.
// pub struct W<T>(pub T);

// Personal preference.
// pub use std::format as f;
