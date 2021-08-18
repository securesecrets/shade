/*
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{coins, from_binary};
    use shade_protocol::asset::Contract;
    use mockall::{automock, predicate::*};

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

    #[cfg_attr(test, automock)]
    trait Query{
        fn query(&self,
            _querier: &QueryMsg,
            _block_size: usize,
            _callback_code_hash: String,
            _contract_addr: HumanAddr,
        ) -> StdResult<ReferenceData> {
            Ok(ReferenceData {
                //11.47 * 10^18
                rate: Uint128(1147 * 10u128.pow(16)),
                last_updated_base: 0,
                last_updated_quote: 0
            })
        }
    }

    #[test]
    fn price_query() {
        let mut deps = dummy_init(&"admin".to_string(),
                                  create_contract("", ""));
        let msg = QueryMsg::GetPrice{
            symbol: "SHD".to_string(),
        };
        let res = query(&mut deps, msg).unwrap();
        let value: ReferenceData = from_binary(&res).unwrap();
        assert_eq!(value.rate, Uint128(1147 * 10u128.pow(16)))
    }
}
*/
