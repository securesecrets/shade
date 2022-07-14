#[cfg(feature = "dex")]
pub mod dex;

#[cfg(feature = "dao")]
pub mod dao;

pub mod oracles;

#[cfg(feature = "mint")]
pub mod mint;

#[cfg(feature = "scrt_staking")]
pub mod staking;

pub mod sky;

#[cfg(feature = "snip20")]
pub mod snip20;

// Protocol init libraries
#[cfg(feature = "airdrop")]
pub mod airdrop;

#[cfg(feature = "initializer")]
pub mod initializer;

// Protocol libraries
#[cfg(feature = "governance")]
pub mod governance;

// Bonds
#[cfg(feature = "bonds")]
pub mod bonds;

#[cfg(feature = "query_auth")]
pub mod query_auth;