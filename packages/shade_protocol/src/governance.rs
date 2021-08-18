use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use crate::asset::Contract;
use crate::generic_response::ResponseStatus;
use crate::msg_traits::{Init, Handle, Query};
use secret_toolkit::snip20::TokenInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GovernanceConfig {
    pub treasury: HumanAddr,
    pub oracle: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Snip20Asset {
    pub contract: Contract,
    pub token_info: TokenInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssetPair {
    pub snip20: Snip20Asset,
    pub mint: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Network {
    pub oracle: Contract,
    pub treasury: Contract,
    pub assets: Vec<AssetPair>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Vote {
    For,
    Against,
    Abstain,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalAction {
    // deploy new snip20/mint pair pegged to an asset
    AddAsset {
        name: String,
        symbol: String,
        code_id: String,
        decimals: Uint128,
        peg: String,
    },

    // Kinda dangerous, probably best not to allow this for now
    //RemoveAsset,

    // General dev work, bugfix/feature
    Development,
    // Change configuration of contract(s) e.g. commission
    Configure {
        contract: Contract,
        // should be something like MintConfig or a generic object
        configuration: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Upcoming,
    Expired,
    InProgress,
    Approved,
    Denied,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proposal{
    pub proposal_id: Uint128,

    pub title: String,
    pub body: String,
    pub requested_funding: Uint128,
    pub funding_denom: String, // SSCRT/SHD/SILK

    pub votes_for: Uint128,
    pub votes_against: Uint128,
    pub votes_abstain: Uint128,

    pub status: ProposalStatus,
    pub action: ProposalAction,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub treasury: HumanAddr,
    pub oracle: HumanAddr,
}

impl Init<'_> for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    SubmitProposal {
        proposal: Proposal,
    },
    Vote {
        proposal_id: Uint128,
        vote: Vote,
        // amount of SHD to vote with
        amount: Uint128,
        reason: Option<String>,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    AddAsset {
        name: String,
        symbol: String,
        code_id: Uint128,
        decimals: Uint128,
        peg: String,
    },
    RegisterBurn {
        // Asset to be registered for burning
        burn_asset: Contract,
        // Asset that will accept the new burner
        mint_asset: Contract,
    },
    RegisterBurnBatch {
        // Assets to be registered for burning
        burn_assets: Vec<Contract>,
        // Assets that will accept the new burners
        mint_assets: Vec<Contract>,
        //code_id: Uint128,
    },
    // Refresh all asset data
    RefreshAsset {
        asset: HumanAddr,
    },
    // Refresh all asset data
    RefreshAssets {
        assets: Option<Vec<HumanAddr>>,
    },
}

impl Handle<'_> for HandleMsg{}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus, address: HumanAddr },
    Receive { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetNetwork {},
    GetProposals {},
    GetProposal { proposal_id: Uint128 },
}

impl Query for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: GovernanceConfig },
    Network { network: Network },
}
