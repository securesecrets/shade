use shade_protocol::{multi_test::App, utils::asset::Contract};

pub fn init(chain: &mut App, super_admin: Option<String>) -> Contract {
    Contract::from(shade_admin::admin::InstantiateMsg { super_admin })
}
