#[cfg(feature = "admin")]
pub mod admin {
    pub use shade_admin_multi_test::multi::AdminAuth;
}

#[cfg(feature = "snip20")]
pub mod snip20 {
    use snip20;
    multi_derive::implement_multi!(Snip20, snip20);
}

#[cfg(feature = "liability_mint")]
pub mod liability_mint {
    use liability_mint;
    multi_derive::implement_multi!(LiabilityMint, liability_mint);
}

// #[cfg(feature = "mint")]
// pub mod mint {
//     use mint;
//     multi_derive::implement_multi!(Mint, mint);
// }

// #[cfg(feature = "oracle")]
// pub mod oracle {
//     use oracle;
//     multi_derive::implement_multi!(Oracle, oracle);
// }

// #[cfg(feature = "mock_band")]
// pub mod mock_band {
//     use crate::multi_derive;
//     use mock_band;

//     pub struct MockBand;
//     multi_derive::implement_multi!(MockBand, mock_band);
// }

#[cfg(feature = "governance")]
pub mod governance {
    use governance;

    multi_derive::implement_multi!(Governance, governance);
}

#[cfg(feature = "snip20_staking")]
pub mod snip20_staking {
    use spip_stkd_0;

    multi_derive::implement_multi!(Snip20Staking, spip_stkd_0);
}

// #[cfg(feature = "bonds")]
// pub mod bonds {
//     use crate::multi_derive;
//     use bonds;

//     pub struct Bonds;
//     multi_derive::implement_multi!(Bonds, bonds);
// }

#[cfg(feature = "query_auth")]
pub mod query_auth {
    use query_auth;

    multi_derive::implement_multi!(QueryAuth, query_auth);
}

#[cfg(feature = "treasury_manager")]
pub mod treasury_manager {
    use treasury_manager;
    multi_derive::implement_multi!(TreasuryManager, treasury_manager);
}

#[cfg(feature = "treasury")]
pub mod treasury {
    use treasury;
    multi_derive::implement_multi!(Treasury, treasury);
}

#[cfg(feature = "scrt_staking")]
pub mod scrt_staking {
    use scrt_staking;
    multi_derive::implement_multi!(ScrtStaking, scrt_staking);
}
