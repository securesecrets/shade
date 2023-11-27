// Use this crate's custom Error type
pub use crate::error::LBPairError as Error;

// Force all Result types to use our Error type
pub type Result<T> = core::result::Result<T, Error>;
