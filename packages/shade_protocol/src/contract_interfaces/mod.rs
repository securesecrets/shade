#[cfg(feature = "dex")]
pub mod dex;

#[cfg(feature = "dao")]
pub mod dao;

pub mod oracles;

#[cfg(feature = "mint")]
pub mod mint;

pub mod staking;

#[cfg(feature = "sky")]
pub mod sky;

#[cfg(feature = "snip20")]
pub mod snip20;

// Protocol init libraries
#[cfg(feature = "airdrop")]
pub mod airdrop;

// Protocol libraries
#[cfg(feature = "governance")]
pub mod governance;

// Bonds
#[cfg(feature = "bonds")]
pub mod bonds;

#[cfg(feature = "query_auth")]
pub mod query_auth;

#[cfg(feature = "admin")]
pub mod admin;

#[cfg(feature = "peg_stability")]
pub mod peg_stability;
