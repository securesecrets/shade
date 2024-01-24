use anyhow::Ok;
use serial_test::serial;
use shade_protocol::c_std::Addr;

use crate::multitests::test_helper::{SHADE, SILK};

use super::test_helper::{init_addrs, setup};
use shade_multi_test::interfaces::{
    router::{self, query_router_registered_tokens},
    utils::SupportedContracts,
};

#[test]
#[serial]
pub fn router_registered_tokens() -> Result<(), anyhow::Error> {
    let addrs = init_addrs();
    let (mut app, _lb_factory, mut deployed_contracts, _, _) = setup(None, None)?;

    //intro app
    router::init(&mut app, addrs.admin().as_str(), &mut deployed_contracts)?;

    let router = match deployed_contracts.clone().get(&SupportedContracts::Router) {
        Some(router) => router,
        None => panic!("Router init failed"),
    }
    .clone()
    .into();

    //generate the snip-20's and rewards token shd,silk

    // init admin_contract

    //init the staking contract

    // query registered tokens

    let reg_tokens = query_router_registered_tokens(&app, &router)?;
    assert_eq!(
        reg_tokens,
        Vec::<Addr>::new(),
        "Empty registered tokens after init"
    );

    // register the tokens

    let shd = match deployed_contracts.get(&SupportedContracts::Snip20(SHADE.to_string())) {
        Some(shd) => shd,
        None => panic!("Shade not registered"),
    };

    let silk = match deployed_contracts.get(&SupportedContracts::Snip20(SILK.to_string())) {
        Some(silk) => silk,
        None => panic!("Silk not registered"),
    };

    router::register_snip20_token(
        &mut app,
        addrs.admin().as_str(),
        &router,
        &shd.clone().into(),
    )?;

    router::register_snip20_token(
        &mut app,
        addrs.admin().as_str(),
        &router,
        &silk.clone().into(),
    )?;
    //test the registered tokens

    let reg_tokens = query_router_registered_tokens(&app, &router)?;
    assert_eq!(reg_tokens.len(), 2, "2 tokens not registered");
    assert_eq!(reg_tokens[0], shd.address, "Shade tokens not registered ");
    assert_eq!(
        reg_tokens[1], silk.address,
        "Silk tokens not registered tokens"
    );

    Ok(())
}
