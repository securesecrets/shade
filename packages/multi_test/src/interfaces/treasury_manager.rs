use crate::{
    interfaces::{
        treasury,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::treasury_manager::TreasuryManager,
};
use shade_admin_multi_test::multi::helpers::init_admin_auth;
use shade_protocol::{
    c_std::Addr,
    contract_interfaces::dao::treasury_manager,
    multi_test::App,
    utils::{asset::Contract, InstantiateCallback, MultiTestable},
};

pub fn init(chain: &mut App, sender: &str, contracts: &mut DeployedContracts) {
    /*let admin_auth = match admin_auth {
        Some(admin) => admin,
        None => Contract::from(init_admin_auth(chain, Addr::unchecked(sender), None)),
    };
    let treasury = match treasury {
        Some(treasury) => treasury,
        None => treasury::init(chain, sender, Some(admin_auth)),
    };*/
    let manager = Contract::form(
        treasury_manager::InstantiateMsg {
            admin_auth: admin_auth.into(),
            viewing_key: "viewing_key".to_string(),
            treasury: treasury.address.into(),
        }
        .test_init(
            TreasuryManager::default(),
            chain,
            sender.into(),
            "manager",
            &[],
        )
        .unwrap(),
    );
    contracts.insert(SupportedContracts::TreasuryManager, treasury_manager);
}
