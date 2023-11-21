use super::{TokenPair, TokenType};
use std::fmt::{Display, Formatter, Result};

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            TokenType::NativeToken { denom, .. } => write!(f, "{}", denom),
            TokenType::CustomToken { contract_addr, .. } => write!(f, "{}", contract_addr),
        }
    }
}

impl Display for TokenPair {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Token 1: {} \n Token 2: {}", self.0, self.1)
    }
}
