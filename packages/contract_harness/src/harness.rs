#[cfg(feature = "mint")]
pub mod mint {
    use crate::harness_macro;
    use mint;

    pub struct Mint;
    harness_macro::implement_harness!(Mint, mint);
}

#[cfg(feature = "oracle")]
pub mod oracle {
    use crate::harness_macro;
    use oracle;

    pub struct Oracle;
    harness_macro::implement_harness!(Oracle, oracle);
}

#[cfg(feature = "mock_band")]
pub mod mock_band {
    use crate::harness_macro;
    use mock_band;

    pub struct MockBand;
    harness_macro::implement_harness!(MockBand, mock_band);
}

#[cfg(feature = "governance")]
pub mod governance {
    use crate::harness_macro;
    use governance;

    pub struct Governance;
    harness_macro::implement_harness!(Governance, governance);
}

#[cfg(feature = "snip20_staking")]
pub mod snip20_staking {
    use crate::harness_macro;
    use spip_stkd_0;

    pub struct Snip20Staking;
    harness_macro::implement_harness!(Snip20Staking, spip_stkd_0);
}

#[cfg(feature = "snip20")]
pub mod snip20 {
    use crate::harness_macro;
    use snip20;

    pub struct Snip20;
    harness_macro::implement_harness!(Snip20, snip20);
}

#[cfg(feature = "bonds")]
pub mod bonds {
    use crate::harness_macro;
    use bonds;

    pub struct Bonds;
    harness_macro::implement_harness!(Bonds, bonds);
}