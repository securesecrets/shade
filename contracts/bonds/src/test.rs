mod test{
    use crate::query;
    use cosmwasm_std::{coins, from_binary, testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage}, Extern, StdError, Uint128, HumanAddr};
    use crate::contract;
    use shade_protocol::bonds::{self, Config, QueryAnswer, QueryMsg, InitMsg};

    #[test]
    fn test_config(){
        let mut deps = mock_dependencies(20, &coins(0, ""));

        // Initialize oracle contract
        let env = mock_env("creator", &coins(0, ""));
        let bonds_init_msg = bonds::InitMsg{
            admin: None,
        };
        let res = contract::init(&mut deps, env, bonds_init_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let check_state = Config{
            admin: HumanAddr::from("creator"),
        };
        let query_answer = query::config(&mut deps).unwrap();
        let query_result = match query_answer{
            QueryAnswer::Config{config} => config == check_state,
            _ => false,
        };
        assert_eq!(true, query_result);
    }
}