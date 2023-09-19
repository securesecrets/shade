use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot process zero tokens")]
    InvalidZeroAmount {},

    #[error("Cannot transfer tokens - controller refuses to transfer more than {max_transferable} tokens")]
    CannotTransfer { max_transferable: Uint128 },

    #[error("Performing operation while there is not enough tokens, {available} tokens available, {needed} needed")]
    InsufficientTokens { available: Uint128, needed: Uint128 },

    #[error("No token holders, noone to distribute tokens to")]
    NoHoldersToDistributeTo {},

    #[error("Cw20 tokens are not supported yet")]
    Cw20TokensNotSupported,
}

impl ContractError {
    pub fn insufficient_tokens(available: impl Into<Uint128>, needed: impl Into<Uint128>) -> Self {
        Self::InsufficientTokens {
            available: available.into(),
            needed: needed.into(),
        }
    }
}
