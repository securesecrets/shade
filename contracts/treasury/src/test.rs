#[cfg(test)]
pub mod tests {
    /*
    use shade_protocol::c_std::{
        testing::{
            mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier
        },
        Addr, coins, Extern,
    };
    use shade_protocol::{
        treasury::InitMsg,
    };

    use crate::{
        contract::init,
    };

    fn create_contract(address: &str, code_hash: &str) -> Contract {
        let env = mock_env(address.to_string(), &[]);
        return Contract{
            address: info.sender,
            code_hash: code_hash.to_string()
        }
    }

    fn dummy_init(admin: String, viewing_key: String) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: Option::from(Addr(admin.clone())),
            viewing_key,
        };
        let env = mock_env(admin, &coins(1000, "earth"));
        let _res = init(&mut deps, env, msg).unwrap();

        return deps
    }
    */
}
