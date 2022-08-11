use crate::shared::is_valid_permission;
use rstest::*;

#[rstest]
#[case("test", false)]
#[case("VAULT_", false)]
#[case("VAULT_TARGET", true)]
#[case("VAULT_TARG3T_2", true)]
#[case("", false)]
#[case("*@#$*!*#!#!#****", false)]
#[case("VAULT_TARGET_addr", false)]
fn test_is_valid_permission(#[case] permission: String, #[case] is_valid: bool) {
    let resp = is_valid_permission(permission.as_str());
    if is_valid {
        assert!(resp.is_ok());
    } else {
        assert!(resp.is_err());
    }
}
