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

#[cfg(feature = "sky")]
pub mod sky {
    use crate::harness_macro;
    use sky;

    pub struct Sky;
    harness_macro::implement_harness!(Sky, sky);
}

#[cfg(feature = "shadeswap_exchange")]
pub mod shadeswap_exchange {
    use crate::harness_macro;
    use amm_pair;

    pub struct ShadeswapExchange;
    harness_macro::implement_harness!(ShadeswapExchange, exchange);
}

#[cfg(feature = "shadeswap_factory")]
pub mod shadeswap_factory {
    use crate::harness_macro;
    use factory;

    pub struct ShadeswapFactory;
    harness_macro::implement_harness_fadroma!(ShadeswapFactory, factory);
}

#[cfg(feature = "shadeswap_lp_token")]
pub mod shadeswap_lp_token {
    use crate::harness_macro;
    use lp_token;

    pub struct ShadeswapLpToken;
    harness_macro::implement_harness!(ShadeswapLpToken, lp_token);
}
#[cfg(feature = "mock_shdswp")]
pub mod mock_shdswp {
    use crate::harness_macro;
    use mock_shade_pair;

    pub struct MockShdSwp;
    harness_macro::implement_harness!(MockShdSwp, mock_shade_pair);
}
