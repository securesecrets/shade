use crate::{
    interfaces::utils::{DeployedContracts, SupportedContracts},
    multi::treasury::Treasury,
};
use shade_admin_multi_test::multi::helpers::init_admin_auth;
use shade_protocol::{
    c_std::Addr,
    contract_interfaces::dao::treasury::InstantiateMsg,
    multi_test::App,
    utils::{asset::Contract, InstantiateCallback, MultiTestable},
};

pub fn init(chain: &mut App, sender: &str, contracts: &mut DeployedContracts) {
    /*let admin = {
        if contracts.contains_key(&SupportedContracts::AdminAuth) {
            contracts.get(&SupportedContracts::AdminAuth).unwrap()
        } else {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender)));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            &contract
        }
    };*/
    let admin = match contracts.get(&SupportedContracts::AdminAuth) {
        Some(admin) => admin.clone(),
        None => {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender)));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            contract
        }
    };
    let treasury = Contract::from(
        InstantiateMsg {
            admin_auth: admin.clone().into(),
            viewing_key: "viewing_key".to_string(),
        }
        .test_init(
            Treasury::default(),
            chain,
            Addr::unchecked(sender),
            "treasury",
            &[],
        )
        .unwrap(),
    );
    contracts.insert(SupportedContracts::Treasury, treasury);
}
