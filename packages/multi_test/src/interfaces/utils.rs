use shade_protocol::utils::asset::Contract;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum SupportedContracts {
    AdminAuth,
    Snip20(String),
    Treasury,
    TreasuryManager(usize),
    MockAdapter(usize),
    ScrtStaking,
}

pub type DeployedContracts = HashMap<SupportedContracts, Contract>;
