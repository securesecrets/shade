use utils::token::Token;

use super::suite::SuiteBuilder;
use crate::error::ContractError;

#[test]
fn adjust_common_token() {
    let mut suite = SuiteBuilder::new().build();

    let old_common_token = suite.query_config().unwrap().common_token;
    let new_native_token = Token::Native("new_token".into());
    assert_ne!(old_common_token, new_native_token);

    suite
        .adjust_common_token(suite.credit_agency().as_str(), new_native_token.clone())
        .unwrap();
    assert_eq!(new_native_token, suite.query_config().unwrap().common_token);

    let new_cw20_token = Token::Cw20("new_token".into());
    assert_ne!(old_common_token, new_cw20_token);

    suite
        .adjust_common_token(suite.credit_agency().as_str(), new_cw20_token.clone())
        .unwrap();
    assert_eq!(new_cw20_token, suite.query_config().unwrap().common_token);
}

#[test]
fn adjust_common_token_without_ca() {
    let mut suite = SuiteBuilder::new().build();
    let new_native_token = Token::Native("new_token".into());

    let err = suite
        .adjust_common_token("not_credit_agency", new_native_token)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());

    let new_cw20_token = Token::Cw20("new_token".into());

    let err = suite
        .adjust_common_token("not_credit_agency", new_cw20_token)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
}
