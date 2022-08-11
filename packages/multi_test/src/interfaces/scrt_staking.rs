use crate::{
    interfaces::{snip20, treasury_manager},
    multi::scrt_staking::ScrtStaking,
};
use shade_admin_multi_test::multi::admin::init_admin_auth;
use shade_protocol::{
    contract_interfaces::dao::scrt_staking,
    multi_test::App,
    utils::asset::Contract,
};

pub fn init(
    chain: &mut App,
    sender: &srt,
    validator_bounds: Option<scrt_staking::ValidatorBounds>,
    admin_auth: Option<Contract>,
    treasury: Option<Contract>,
    manager: Option<Contract>,
    sscrt: Option<Contract>,
) -> HashMap<utils::Contracts, Contract> {
    let admin_auth = match admin_auth {
        Some(admin) => admin,
        None => Contract::from(init_admin_auth(chain, Addr::unchecked(sender), None)),
    };
    let manager = match manager {
        Some(manager) => manager,
        None => treasury_manager::init(chain, sender, admin_auth, treasury),
    };
    let sscrt = match sscrt {
        Some(sscrt) => sscrt,
        None => snip20::init(
            chain,
            sender,
            "secretSecret".to_string(),
            "SSCRT".to_string(),
            6,
            None,
        ),
    };
    Contract::from(
        scrt_staking::InstantiateMsg {
            admin_auth: admin_auth.into(),
            owner: manager.address.into(),
            sscrt: sscrt.into(),
            validator_bounds,
            viewing_key: "veiwing_key".into(),
        }
        .test_init(
            ScrtStaking::default(),
            chain,
            sender.to_string(),
            "scrt_staking",
            &[],
        )
        .unwrap(),
    )
}
