use shade_multi_test::interfaces::{lb_factory, lb_pair};

use super::{lb_pair_fees::ACTIVE_ID, test_helper::*};

#[test]
pub fn staking_contract_init() -> Result<(), anyhow::Error> {
    // should be init with the lb-pair
    //then query it about the contract info
    let addrs = init_addrs();
    let (mut app, lb_factory, deployed_contracts) = setup(None, None)?;

    let silk = extract_contract_info(&deployed_contracts, SILK)?;
    let shade = extract_contract_info(&deployed_contracts, SHADE)?;
    let token_x = token_type_snip20_generator(&shade)?;
    let token_y = token_type_snip20_generator(&silk)?;

    lb_factory::create_lb_pair(
        &mut app,
        addrs.admin().as_str(),
        &lb_factory.clone().into(),
        DEFAULT_BIN_STEP,
        ACTIVE_ID,
        token_x.clone(),
        token_y.clone(),
        "viewing_key".to_string(),
        "entropy".to_string(),
    )?;

    let all_pairs =
        lb_factory::query_all_lb_pairs(&mut app, &lb_factory.clone().into(), token_x, token_y)?;
    let lb_pair = all_pairs[0].clone();
    println!("LB_PAIR {:?}", lb_pair);

    let lb_token = lb_pair::query_lb_token(&mut app, &lb_pair.lb_pair.contract)?;
    println!("lb_token {:?}", lb_token);

    let staking_contract = lb_pair::query_staking_contract(&mut app, &lb_pair.lb_pair.contract)?;
    println!("staking_contract {:?}", staking_contract);

    Ok(())
}
