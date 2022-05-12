#[cfg(feature = "snip20")]
pub mod snip20 {
    use snip20_reference_impl;
    use crate::harness_macro;

    pub struct Snip20;
    harness_macro::implement_harness!(Snip20, snip20_reference_impl);
}

#[cfg(feature = "mint")]
pub mod mint {
    use mint;
    use crate::harness_macro;

    pub struct Mint;
    harness_macro::implement_harness!(Mint, mint);
}

#[cfg(feature = "oracle")]
pub mod oracle {
    use oracle;
    use crate::harness_macro;

    pub struct Oracle;
    harness_macro::implement_harness!(Oracle, oracle);
}

#[cfg(feature = "mock_band")]
pub mod mock_band {
    use mock_band;
    use crate::harness_macro;

    pub struct MockBand;
    harness_macro::implement_harness!(MockBand, mock_band);
}

#[cfg(feature = "treasury")]
pub mod treasury {
    use treasury;
    use crate::harness_macro;

    pub struct Treasury;
    harness_macro::implement_harness!(Treasury, treasury);
}

#[cfg(feature = "treasury_manager")]
pub mod treasury_manager {
    use treasury_manager;
    use crate::harness_macro;

    pub struct TreasuryManager;
    harness_macro::implement_harness!(TreasuryManager, treasury_manager);
}

#[cfg(feature = "scrt_staking")]
pub mod scrt_staking {
    use scrt_staking;
    use crate::harness_macro;

    pub struct ScrtStaking;
    harness_macro::implement_harness!(ScrtStaking, scrt_staking);
}
