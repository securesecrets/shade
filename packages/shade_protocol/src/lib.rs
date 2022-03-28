#[cfg(feature = "band")]
pub mod band;

#[cfg(feature = "secretswap")]
pub mod secretswap;

#[cfg(feature = "snip20")]
pub mod snip20;

pub mod utils;

// Protocol init libraries
#[cfg(feature = "airdrop")]
pub mod airdrop;

#[cfg(feature = "initializer")]
pub mod initializer;

// Protocol libraries
#[cfg(feature = "governance")]
pub mod governance;

#[cfg(feature = "mint")]
pub mod mint;

#[cfg(feature = "mint_router")]
pub mod mint_router;

#[cfg(feature = "oracle")]
pub mod oracle;

#[cfg(feature = "scrt_staking")]
pub mod scrt_staking;

#[cfg(feature = "staking")]
pub mod staking;

#[cfg(feature = "treasury")]
pub mod treasury;
