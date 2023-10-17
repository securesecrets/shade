use super::suite::{contract_market, contract_token, SuiteBuilder};

use crate::msg::MigrateMsg;

#[test]
fn migration_with_token_id() {
    let mut suite = SuiteBuilder::new().build();

    let new_market_id = suite.app().store_code(contract_market());
    let new_token_id = suite.app().store_code(contract_token());

    suite
        .migrate(
            new_market_id,
            &MigrateMsg {
                wynd_lend_token_id: Some(new_token_id),
            },
        )
        .unwrap();
    assert_eq!(new_token_id, suite.query_config().unwrap().token_id);
}

#[test]
fn migration_without_token_id() {
    let mut suite = SuiteBuilder::new().build();

    let old_token_id = suite.query_config().unwrap().token_id;
    let new_market_id = suite.app().store_code(contract_market());

    suite
        .migrate(
            new_market_id,
            &MigrateMsg {
                wynd_lend_token_id: None,
            },
        )
        .unwrap();
    assert_eq!(old_token_id, suite.query_config().unwrap().token_id);
}
