#[cfg(feature = "admin")]
pub mod admin {
    pub use admin;
    multi_derive::implement_multi!(Admin, admin);
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

// #[cfg(feature = "scrt_staking")]
// pub mod scrt_staking {
//     use crate::multi_derive;
//     use scrt_staking;

//     pub struct ScrtStaking;
//     multi_derive::implement_multi!(ScrtStaking, scrt_staking);
// }

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

// #[cfg(feature = "treasury_manager")]
// pub mod treasury_manager {
//     use crate::multi_derive;
//     use treasury_manager;

//     pub struct TreasuryManager;
//     multi_derive::implement_multi!(TreasuryManager, treasury_manager);
// }

// #[cfg(feature = "treasury")]
// pub mod treasury {
//     use crate::multi_derive;
//     use treasury;

//     pub struct Treasury;
//     multi_derive::implement_multi!(Treasury, treasury);
// }
