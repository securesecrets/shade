use crate::{
    interfaces::{
        snip20,
        treasury,
        treasury_manager,
        utils::{DeployedContracts, SupportedContracts},
    },
    multi::scrt_staking::ScrtStaking,
};
use shade_admin_multi_test::multi::helpers::init_admin_auth;
use shade_protocol::{
    c_std::Addr,
    contract_interfaces::dao::scrt_staking,
    multi_test::App,
    utils::{asset::Contract, InstantiateCallback, MultiTestable},
};

pub fn init(
    chain: &mut App,
    sender: &str,
    contracts: &mut DeployedContracts,
    validator_bounds: Option<scrt_staking::ValidatorBounds>,
    manager: usize,
) {
    let treasury_manager = match contracts.get(&SupportedContracts::TreasuryManager(0)) {
        Some(manager) => manager.clone(),
        None => {
            treasury_manager::init(chain, sender, contracts, 0);
            contracts
                .get(&SupportedContracts::TreasuryManager(0))
                .unwrap()
                .clone()
        }
    };
    let treasury = match contracts.get(&SupportedContracts::Treasury) {
        Some(treasury) => treasury.clone(),
        None => {
            treasury::init(chain, sender, contracts);
            contracts
                .get(&SupportedContracts::Treasury)
                .unwrap()
                .clone()
        }
    };
    let admin_auth = match contracts.get(&SupportedContracts::AdminAuth) {
        Some(admin) => admin.clone(),
        None => {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender)));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            contract
        }
    };
    let sscrt = match contracts.get(&SupportedContracts::Snip20("SSCRT".to_string())) {
        Some(snip20) => snip20.clone(),
        None => {
            snip20::init(
                chain,
                sender,
                contracts,
                "secretSCRT".to_string(),
                "SSCRT".to_string(),
                6,
                None,
            );
            contracts
                .get(&SupportedContracts::Snip20("SSCRT".to_string()))
                .unwrap()
                .clone()
        }
    };
    let scrt_staking = Contract::from(
        scrt_staking::InstantiateMsg {
            admin_auth: admin_auth.into(),
            owner: treasury_manager.address.into(),
            sscrt: sscrt.into(),
            validator_bounds,
            viewing_key: "viewing_key".into(),
        }
        .test_init(
            ScrtStaking::default(),
            chain,
            Addr::unchecked(sender),
            "scrt_staking",
            &[],
        )
        .unwrap(),
    );
    contracts.insert(SupportedContracts::ScrtStaking, scrt_staking);
}
