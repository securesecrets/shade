#[cfg(feature = "snip20")]
pub mod snip20 {
    use crate::harness_macro;
    use snip20_reference_impl;

    pub struct Snip20;
    harness_macro::implement_harness!(Snip20, snip20_reference_impl);
}

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
