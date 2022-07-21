#[cfg(feature = "admin")]
pub mod admin {
    pub use shade_admin_multi_test::AdminAuth;
}

#[cfg(feature = "snip20")]
pub mod snip20 {
    use snip20;
    multi_derive::implement_multi!(Snip20, snip20);
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
//     use crate::multi_macro;
//     use mock_band;

//     pub struct MockBand;
//     multi_macro::implement_multi!(MockBand, mock_band);
// }

#[cfg(feature = "governance")]
pub mod governance {
    use governance;

    pub struct Governance;
    multi_macro::implement_multi!(Governance, governance);
}

#[cfg(feature = "snip20_staking")]
pub mod snip20_staking {
    use spip_stkd_0;

    pub struct Snip20Staking;
    multi_macro::implement_multi!(Snip20Staking, spip_stkd_0);
}

// #[cfg(feature = "scrt_staking")]
// pub mod scrt_staking {
//     use crate::multi_macro;
//     use scrt_staking;

//     pub struct ScrtStaking;
//     multi_macro::implement_multi!(ScrtStaking, scrt_staking);
// }

// #[cfg(feature = "bonds")]
// pub mod bonds {
//     use crate::multi_macro;
//     use bonds;

//     pub struct Bonds;
//     multi_macro::implement_multi!(Bonds, bonds);
// }

#[cfg(feature = "query_auth")]
pub mod query_auth {
    use query_auth;

    pub struct QueryAuth;
    multi_macro::implement_multi!(QueryAuth, query_auth);
}

// #[cfg(feature = "treasury_manager")]
// pub mod treasury_manager {
//     use crate::multi_macro;
//     use treasury_manager;

//     pub struct TreasuryManager;
//     multi_macro::implement_multi!(TreasuryManager, treasury_manager);
// }

// #[cfg(feature = "treasury")]
// pub mod treasury {
//     use crate::multi_macro;
//     use treasury;

//     pub struct Treasury;
//     multi_macro::implement_multi!(Treasury, treasury);
// }