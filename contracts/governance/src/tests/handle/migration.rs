// Migrate to new contract
// Send data
// Receive data

use crate::tests::handle::runstate::init_gov;
use shade_protocol::{
    c_std::{Addr, ContractInfo},
    governance,
    governance::{profile::Profile, ExecuteMsg, MigrationDataAsk, QueryAnswer, QueryMsg},
    multi_test::App,
    utils::{ExecuteCallback, Query},
};

#[test]
fn migrate() {
    let (mut chain, gov, _snip20, gov_id) = init_gov().unwrap();

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
            profile: 1,
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

    let answer: governance::QueryAnswer = governance::QueryMsg::Config {}
        .test_query(&gov, &chain)
        .unwrap();

    let new_gov = match answer {
        QueryAnswer::Config { config } => {
            if let Some(contract) = config.migrated_to {
                ContractInfo {
                    address: contract.address,
                    code_hash: contract.code_hash,
                }
            } else {
                panic!("No migration target")
            }
        }
        _ => panic!("Not the expected response"),
    };

    // Check that totals are well initialized
    assert_query_totals(QueryMsg::TotalAssemblies {}, &chain, &gov, &new_gov);
    assert_query_totals(QueryMsg::TotalAssemblyMsgs {}, &chain, &gov, &new_gov);
    assert_query_totals(QueryMsg::TotalContracts {}, &chain, &gov, &new_gov);
    assert_query_totals(QueryMsg::TotalProfiles {}, &chain, &gov, &new_gov);

    // Check that gov is not well initialized
    assert_migrated_items(&chain, &gov, &new_gov, false);

    // Make sure that it can handle exact migration
    ExecuteMsg::MigrateData {
        data: MigrationDataAsk::Assembly,
        total: 22,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("anyone"), &[])
    .unwrap();

    // Handle fractional migrations
    ExecuteMsg::MigrateData {
        data: MigrationDataAsk::Profile,
        total: 11,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("anyone"), &[])
    .unwrap();
    ExecuteMsg::MigrateData {
        data: MigrationDataAsk::Profile,
        total: 11,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("anyone"), &[])
    .unwrap();

    // Handle amount overflow
    ExecuteMsg::MigrateData {
        data: MigrationDataAsk::AssemblyMsg,
        total: 40,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("anyone"), &[])
    .unwrap();

    ExecuteMsg::MigrateData {
        data: MigrationDataAsk::Contract,
        total: 25,
    }
    .test_exec(&gov, &mut chain, Addr::unchecked("anyone"), &[])
    .unwrap();

    // Finally assert that everything is fine
    assert_migrated_items(&chain, &gov, &new_gov, true);
}

fn assert_query_totals(query: QueryMsg, chain: &App, gov1: &ContractInfo, gov2: &ContractInfo) {
    let query1 = query.test_query(&gov1, &chain).unwrap();
    let query2 = query.test_query(&gov2, &chain).unwrap();
    if let QueryAnswer::Total { total: total1 } = query1 {
        if let QueryAnswer::Total { total: total2 } = query2 {
            assert_eq!(total1, total2);
        } else {
            panic!("Expected something")
        }
    } else {
        panic!("Expected something")
    }
}

fn assert_migrated_items(
    chain: &App,
    gov1: &ContractInfo,
    gov2: &ContractInfo,
    should_equal: bool,
) {
    ///////// ASSEMBLIES
    let query = QueryMsg::Assemblies { start: 0, end: 25 };
    let query1 = query.test_query(&gov1, &chain).unwrap();
    let query2_try = query.test_query(&gov2, &chain);

    // Should error out cause item is not found
    if !should_equal {
        assert!(query2_try.is_err());
    } else if let QueryAnswer::Assemblies {
        assemblies: assemblies1,
    } = query1
    {
        let query2 = query2_try.unwrap();
        if let QueryAnswer::Assemblies {
            assemblies: assemblies2,
        } = query2
        {
            assert_eq!(assemblies1.len(), assemblies2.len());
            if should_equal {
                for (i, assembly) in assemblies1.iter().enumerate() {
                    assert_eq!(assembly.clone(), assemblies2[i]);
                }
            }
        } else {
            panic!("Expected something")
        }
    } else {
        panic!("Expected something")
    }

    ///////// ASSEMBLY MSGS
    let query = QueryMsg::AssemblyMsgs { start: 0, end: 25 };
    let query1 = query.test_query(&gov1, &chain).unwrap();
    let query2_try = query.test_query(&gov2, &chain);

    // Should error out cause item is not found
    if !should_equal {
        assert!(query2_try.is_err());
    } else if let QueryAnswer::AssemblyMsgs { msgs: msgs1 } = query1 {
        let query2 = query2_try.unwrap();
        if let QueryAnswer::AssemblyMsgs { msgs: msgs2 } = query2 {
            assert_eq!(msgs1.len(), msgs2.len());
            if should_equal {
                for (i, msg) in msgs1.iter().enumerate() {
                    assert_eq!(msg.clone(), msgs2[i]);
                }
            }
        } else {
            panic!("Expected something")
        }
    } else {
        panic!("Expected something")
    }

    ///////// PROFILES
    let query = QueryMsg::Profiles { start: 0, end: 25 };
    let query1 = query.test_query(&gov1, &chain).unwrap();
    let query2_try = query.test_query(&gov2, &chain);

    // Should error out cause item is not found
    if !should_equal {
        assert!(query2_try.is_err());
    } else if let QueryAnswer::Profiles {
        profiles: profiles1,
    } = query1
    {
        let query2 = query2_try.unwrap();
        if let QueryAnswer::Profiles {
            profiles: profiles2,
        } = query2
        {
            assert_eq!(profiles1.len(), profiles2.len());
            if should_equal {
                for (i, profile) in profiles1.iter().enumerate() {
                    assert_eq!(profile.clone(), profiles2[i]);
                }
            }
        } else {
            panic!("Expected something")
        }
    } else {
        panic!("Expected something")
    }

    ///////// CONTRACTS
    let query = QueryMsg::Contracts { start: 0, end: 25 };
    let query1 = query.test_query(&gov1, &chain).unwrap();
    let query2_try = query.test_query(&gov2, &chain);

    // Should error out cause item is not found
    if !should_equal {
        assert!(query2_try.is_err());
    } else if let QueryAnswer::Contracts {
        contracts: contracts1,
    } = query1
    {
        let query2 = query2_try.unwrap();
        if let QueryAnswer::Contracts {
            contracts: contracts2,
        } = query2
        {
            assert_eq!(contracts1.len(), contracts2.len());
            if should_equal {
                for (i, contract) in contracts1.iter().enumerate() {
                    assert_eq!(contract.clone(), contracts2[i]);
                }
            }
        } else {
            panic!("Expected something")
        }
    } else {
        panic!("Expected something")
    }
}
