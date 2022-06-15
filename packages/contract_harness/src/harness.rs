#[cfg(feature = "snip20")]
pub mod snip20 {
    use crate::harness_macro;
    use snip20_reference_impl;

    pub struct Snip20;
    harness_macro::implement_harness!(Snip20, snip20_reference_impl);
}

#[cfg(feature = "sky")]
pub mod sky {
    use crate::harness_macro;
    use sky;

    pub struct Sky;
    harness_macro::implement_harness!(Sky, sky);
}

#[cfg(feature = "sienna_exchange")]
pub mod sienna_exchange {
    use crate::harness_macro;
    use exchange;

    pub struct SiennaExchange;
    harness_macro::implement_harness!(SiennaExchange, exchange);
}

#[cfg(feature = "sienna_factory")]
pub mod sienna_factory {
    use crate::harness_macro;
    use factory;

    pub struct SiennaFactory;
    harness_macro::implement_harness!(SiennaFactory, factory);
}

#[cfg(feature = "sienna_lp_token")]
pub mod sienna_lp_token {
    use crate::harness_macro;
    use lp_token;

    pub struct SiennaLpToken;
    harness_macro::implement_harness_lp!(SiennaLpToken, lp_token);
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