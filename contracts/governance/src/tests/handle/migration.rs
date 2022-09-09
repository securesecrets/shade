// Migrate to new contract
// Send data
// Receive data

use crate::tests::handle::runstate::init_gov;
use shade_protocol::{
    c_std::{to_binary, Addr, ContractInfo, StdResult, Uint128},
    governance,
    governance::profile::Profile,
    multi_test::{App, AppResponse, BasicApp, Executor},
    utils::ExecuteCallback,
};

#[test]
fn migrate() {
    let (mut chain, gov, snip20, gov_id) = init_gov().unwrap();

    for i in 0..20 {
        // Generate multiple assemblies to migrate
        governance::ExecuteMsg::AddAssembly {
            name: format!("Assembly {}", i),
            metadata: "some data".to_string(),
            members: vec![
                Addr::unchecked("alpha"),
                Addr::unchecked("beta"),
                Addr::unchecked("charlie"),
            ],
            profile: Uint128::new(1),
            padding: None,
        }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            gov.address.clone(),
            &[],
        )
        .unwrap();

        // Generate multiple profiles
        governance::ExecuteMsg::AddProfile {
            profile: Profile {
                name: format!("Profile {}", i),
                enabled: false,
                assembly: None,
                funding: None,
                token: None,
                cancel_deadline: 0,
            },
            padding: None,
        }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            gov.address.clone(),
            &[],
        )
        .unwrap();

        // Generate multiple assembly msgs
        governance::ExecuteMsg::AddAssemblyMsg {
            name: format!("AssemblyMsg {}", i),
            msg: "{}".to_string(),
            assemblies: vec![],
            padding: None,
        }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            gov.address.clone(),
            &[],
        )
        .unwrap();

        // Generate multiple contracts
        governance::ExecuteMsg::AddContract {
            name: format!("Contract {}", i),
            metadata: "".to_string(),
            contract: Default::default(),
            assemblies: None,
            padding: None,
        }
        .test_exec(
            // Sender is self
            &gov,
            &mut chain,
            gov.address.clone(),
            &[],
        )
        .unwrap();
    }

    governance::ExecuteMsg::Migrate {
        id: gov_id,
        label: "new_governance".to_string(),
        code_hash: gov.code_hash.clone(),
    }
    .test_exec(
        // Sender is self
        &gov,
        &mut chain,
        gov.address.clone(),
        &[],
    )
    .unwrap();
}
