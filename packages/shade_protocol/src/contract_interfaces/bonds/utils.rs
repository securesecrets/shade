use sha2::{Digest, Sha256};
use std::convert::TryInto;
use subtle::ConstantTimeEq;

pub const VIEWING_KEY_SIZE: usize = 32;
pub const VIEWING_KEY_PREFIX: &str = "api_key_";

pub fn ct_slice_compare(s1: &[u8], s2: &[u8]) -> bool {
    bool::from(s1.ct_eq(s2))
}

pub fn create_hashed_password(s1: &str) -> [u8; VIEWING_KEY_SIZE] {
    Sha256::digest(s1.as_bytes())
        .as_slice()
        .try_into()
        .expect("Wrong password length")
}
