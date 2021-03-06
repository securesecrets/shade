#[cfg(feature = "adapter")]
pub mod adapter;

#[cfg(feature = "treasury_manager")]
pub mod treasury_manager;

#[cfg(feature = "rewards_emission")]
pub mod rewards_emission;

#[cfg(feature = "treasury")]
pub mod treasury;

#[cfg(feature = "scrt_staking")]
pub mod scrt_staking;

#[cfg(feature = "lp_shade_swap")]
pub mod lp_shade_swap;
