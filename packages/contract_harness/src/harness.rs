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

#[cfg(feature = "scrt_staking")]
pub mod scrt_staking {
    use crate::harness_macro;
    use scrt_staking;

    pub struct ScrtStaking;
    harness_macro::implement_harness!(ScrtStaking, scrt_staking);
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

#[cfg(feature = "query_auth")]
pub mod query_auth {
    use crate::harness_macro;
    use query_auth;

    pub struct QueryAuth;
    harness_macro::implement_harness!(QueryAuth, query_auth);
}

#[cfg(feature = "admin")]
pub mod admin {
    use crate::harness_macro;
    use admin;

    pub struct Admin;
    harness_macro::implement_harness!(Admin, admin);
}

#[cfg(feature = "snip20_reference_impl")]
pub mod snip20_reference_impl {
    use crate::harness_macro;
    use snip20_reference_impl;

    pub struct Snip20ReferenceImpl;
    harness_macro::implement_harness!(Snip20ReferenceImpl, snip20_reference_impl);
}

#[cfg(feature = "treasury_manager")]
pub mod treasury_manager {
    use crate::harness_macro;
    use treasury_manager;

    pub struct TreasuryManager;
    harness_macro::implement_harness!(TreasuryManager, treasury_manager);
}

#[cfg(feature = "treasury")]
pub mod treasury {
    use crate::harness_macro;
    use treasury;

    pub struct Treasury;
    harness_macro::implement_harness!(Treasury, treasury);
}