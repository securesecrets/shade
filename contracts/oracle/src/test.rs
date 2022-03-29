#[cfg(test)]
mod tests {
    use crate::contract;
    use cosmwasm_std::{coins, from_binary, testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage}, Extern, StdError, Uint128, HumanAddr};
    use shade_protocol::{utils::asset::Contract, oracle::{self, OracleConfig, QueryAnswer}};
    use crate::query;
}
