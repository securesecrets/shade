#[cfg(feature = "mint")]
pub mod mint {
    use crate::implement_harness;
    use mint;

    pub struct Mint;
    implement_harness!(Mint, mint);
}

#[cfg(feature = "oracle")]
pub mod oracle {
    use crate::implement_harness;
    use oracle;

    pub struct Oracle;
    implement_harness!(Oracle, oracle);
}

#[cfg(feature = "mock_band")]
pub mod mock_band {
    use crate::implement_harness;
    use mock_band;

    pub struct MockBand;
    implement_harness!(MockBand, mock_band);
}

#[cfg(feature = "governance")]
pub mod governance {
    use crate::implement_harness;
    use governance;

    pub struct Governance;
    implement_harness!(Governance, governance);
}

#[cfg(feature = "snip20_staking")]
pub mod snip20_staking {
    use crate::implement_harness;
    use spip_stkd_0;

    pub struct Snip20Staking;
    implement_harness!(Snip20Staking, spip_stkd_0);
}

#[cfg(feature = "scrt_staking")]
pub mod scrt_staking {
    use crate::implement_harness;
    use scrt_staking;

    pub struct ScrtStaking;
    implement_harness!(ScrtStaking, scrt_staking);
}

#[cfg(feature = "snip20")]
pub mod snip20 {
    use crate::implement_harness;
    use snip20;

    pub struct Snip20;
    implement_harness!(Snip20, snip20);
}

#[cfg(feature = "bonds")]
pub mod bonds {
    use crate::implement_harness;
    use bonds;

    pub struct Bonds;
    implement_harness!(Bonds, bonds);
}

#[cfg(feature = "query_auth")]
pub mod query_auth {
    use crate::implement_harness;
    use query_auth;

    pub struct QueryAuth;
    implement_harness!(QueryAuth, query_auth);
}

#[cfg(feature = "admin")]
pub mod admin {
    use crate::implement_harness;
    use admin;

    pub struct Admin;
    implement_harness!(Admin, admin);
}

#[cfg(feature = "snip20_reference_impl")]
pub mod snip20_reference_impl {
    use crate::implement_harness;
    use snip20_reference_impl;

    pub struct Snip20ReferenceImpl;
    implement_harness!(Snip20ReferenceImpl, snip20_reference_impl);
}

#[cfg(feature = "treasury_manager")]
pub mod treasury_manager {
    use crate::implement_harness;
    use treasury_manager;

    pub struct TreasuryManager;
    implement_harness!(TreasuryManager, treasury_manager);
}

#[cfg(feature = "treasury")]
pub mod treasury {
    use crate::implement_harness;
    use treasury;

    pub struct Treasury;
    implement_harness!(Treasury, treasury);
}
