use super::TokenPair;
use std::fmt::{Display, Formatter, Result};

impl Display for TokenPair {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Token 1: {} \n Token 2: {}", self.0, self.1)
    }
}
