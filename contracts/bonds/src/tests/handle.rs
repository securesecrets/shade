use crate::tests::init_contracts;
use fadroma::ensemble::MockEnv;
use cosmwasm_std::HumanAddr;
use shade_protocol::contract_interfaces::bonds;

#[test]
pub fn set_admin() {
    let (mut chain, 
        bonds, 
        issu, 
        coll, 
        band, 
        oracle,
        query_auth,
        shade_admins
    ) = init_contracts().unwrap();

    let msg = bonds::HandleMsg::AddAdmin {
        admin_to_add: HumanAddr::from("new_admin"),
        padding: None,
    };

    assert!(chain.execute(&msg, MockEnv::new("not_admin", bonds.clone())).is_err());
    assert!(chain.execute(&msg, MockEnv::new("admin", bonds.clone())).is_err());
    assert!(chain.execute(&msg, MockEnv::new("limit_admin", bonds.clone())).is_ok());

    let query: bonds::QueryAnswer = chain.query(
        bonds.address,
        &bonds::QueryMsg::Config {  }
    ).unwrap();

    match query {
        bonds::QueryAnswer::Config { config, .. } => {
            assert_eq!(config.admin, vec![HumanAddr::from("admin"), HumanAddr::from("new_admin")]);
        }
        _ => assert!(false)
    };
}

#[test]
pub fn purchase_opportunity() {
    let (mut chain, 
        bonds, 
        issu, 
        coll, 
        band, 
        oracle,
        query_auth,
        shade_admins
    ) = init_contracts().unwrap();


}