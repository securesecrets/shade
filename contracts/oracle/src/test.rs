#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{debug_print, coins, from_binary, Uint128, StdResult, HumanAddr, Extern};
    use shade_protocol::{
        oracle::{InitMsg, QueryMsg},
        band::ReferenceData,
        asset::Contract,
    };
    use crate::contract::{init, query};

    fn create_contract(address: &str, code_hash: &str) -> Contract {
        let env = mock_env(address.to_string(), &[]);
        return Contract{
            address: env.message.sender,
            code_hash: code_hash.to_string()
        }
    }

    fn dummy_init(admin: &str, band: Contract) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
            band: band,
        };
        let env = mock_env(admin.to_string(), &coins(1000, "earth"));
        let _res = init(&mut deps, env, msg).unwrap();

        return deps
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            admin: None,
            band: create_contract("", ""),
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn price_query() {
        let deps = dummy_init(&"admin".to_string(),
                                  create_contract("", ""));
        let msg = QueryMsg::GetPrice{
            symbol: "SHD".to_string(),
        };
        let res = query(&deps, msg).unwrap();

        let value: ReferenceData = from_binary(&res).unwrap();
        assert_eq!(value.rate, Uint128(1147 * 10u128.pow(16)));

    }

    #[test]
    fn prices_query() {
        let deps = dummy_init(&"admin".to_string(),
                                  create_contract("secretaddress", ""));
        debug_print!("TESTGET PRICES");
        let msg = QueryMsg::GetPrices{
            symbols: [
                "SHD".to_string(), 
                "SILK".to_string(),
            ].to_vec()
        };
        let res = query(&deps, msg).unwrap();

        let values: Vec<ReferenceData> = from_binary(&res).unwrap();
        assert_eq!(values[0].rate, Uint128(1147 * 10u128.pow(16)));
        assert_eq!(values[1].rate, Uint128(1 * 10u128.pow(18)));
    }
}
