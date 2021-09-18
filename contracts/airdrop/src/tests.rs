#[cfg(test)]
pub mod tests {
    use cosmwasm_std::{
        testing::{
            mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier
        },
        coins, from_binary, StdError, Uint128,
        Extern,
    };
    use shade_protocol::{

    };
    use mockall_double::double;

    use crate::{
        contract::{
            init, handle, query,
        },
        handle::{
            calculate_commission,
            calculate_mint,
            try_burn,
        },
    };

    // TODO: when equation is done, create tests
}