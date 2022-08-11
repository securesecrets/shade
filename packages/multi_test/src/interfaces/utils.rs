use shade_protocol::utils::asset::Contract;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash)]
pub enum SupportedContracts {
    AdminAuth,
    Snip20,
    Treasury,
    TreasuryManager,
    ScrtStaking,
}

pub type DeployedContracts = HashMap<SupportedContracts, Contract>;
