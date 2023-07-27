#[cfg(feature = "admin")]
pub mod admin {
    pub use admin;
    use shade_protocol::{admin::InstantiateMsg, multi_test::App, utils::InstantiateCallback};
    multi_derive::implement_multi!(Admin, admin);

    // Multitest helper
    pub fn init_admin_auth(app: &mut App, superadmin: &Addr) -> ContractInfo {
        InstantiateMsg {
            super_admin: Some(superadmin.clone().to_string()),
        }
        .test_init(Admin::default(), app, superadmin.clone(), "admin_auth", &[])
        .unwrap()
    }
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

#[cfg(feature = "stkd_scrt")]
pub mod stkd_scrt {
    use stkd_scrt;
    multi_derive::implement_multi!(StkdScrt, stkd_scrt);
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

    multi_derive::implement_multi_with_reply!(Governance, governance);
}

// #[cfg(feature = "snip20_staking")]
// pub mod snip20_staking {
//     use spip_stkd_0;
//
//     multi_derive::implement_multi!(Snip20Staking, spip_stkd_0);
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

#[cfg(feature = "mock_adapter")]
pub mod mock_adapter {
    use mock_adapter;
    multi_derive::implement_multi!(MockAdapter, mock_adapter);
}

#[cfg(feature = "scrt_staking")]
pub mod scrt_staking {
    use scrt_staking;
    multi_derive::implement_multi!(ScrtStaking, scrt_staking);
}

#[cfg(feature = "basic_staking")]
pub mod basic_staking {
    use basic_staking;
    multi_derive::implement_multi!(BasicStaking, basic_staking);
}

#[cfg(feature = "peg_stability")]
pub mod peg_stability {
    use peg_stability;

    multi_derive::implement_multi!(PegStability, peg_stability);
}

#[cfg(feature = "mock_stkd")]
pub mod mock_stkd {
    pub use mock_stkd;
    multi_derive::implement_multi!(MockStkd, mock_stkd);
}

#[cfg(feature = "mock_sienna")]
pub mod mock_sienna {
    pub use mock_sienna;
    multi_derive::implement_multi!(MockSienna, mock_sienna);
}

#[cfg(feature = "snip20_migration")]
pub mod snip20_migration {
    use snip20_migration;

    multi_derive::implement_multi!(Snip20Migration, snip20_migration);
}
