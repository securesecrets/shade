pub mod assembly;
pub mod contract;
pub mod profile;
pub mod proposal;
#[cfg(feature = "governance-impl")]
pub mod stored_id;
pub mod vote;

use crate::{
    contract_interfaces::governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::{Profile, UpdateProfile},
        proposal::{Proposal, ProposalMsg},
        vote::Vote,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use crate::c_std::Uint128;
use crate::c_std::{Binary, Coin, Addr};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::default::SingletonStorage;

// Admin command variable spot
pub const MSG_VARIABLE: &str = "{~}";

#[cw_serde]
pub struct Config {
    pub treasury: Addr,
    // When public voting is enabled, a voting token is expected
    pub vote_token: Option<Contract>,
    // When funding is enabled, a funding token is expected
    pub funding_token: Option<Contract>,
}

#[cfg(feature = "governance-impl")]
impl SingletonStorage for Config {
    const NAMESPACE: &'static [u8] = b"config-";
}

#[cw_serde]
pub struct InstantiateMsg {
    pub treasury: Addr,

    // Admin rules
    pub admin_members: Vec<Addr>,
    pub admin_profile: Profile,

    // Public rules
    pub public_profile: Profile,
    pub funding_token: Option<Contract>,
    pub vote_token: Option<Contract>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum RuntimeState {
    // Run like normal
    Normal,
    // Disable staking
    DisableVoteToken,
    // Allow only specific assemblys and admin
    SpecificAssemblys { commitees: Vec<Uint128> },
    // Set as admin only
    AdminOnly,
}

#[cfg(feature = "governance-impl")]
impl SingletonStorage for RuntimeState {
    const NAMESPACE: &'static [u8] = b"runtime_state-";
}

#[cw_serde]
pub enum ExecuteMsg {
    // Internal config
    SetConfig {
        treasury: Option<Addr>,
        funding_token: Option<Contract>,
        vote_token: Option<Contract>,
        padding: Option<String>,
    },
    SetRuntimeState {
        state: RuntimeState,
        padding: Option<String>,
    },

    // Proposals
    // Same as AssemblyProposal where assembly is 0 and assembly msg is 0
    Proposal {
        title: String,
        metadata: String,

        // Optionals, if none the proposal is assumed to be a text proposal
        // Allowed Contract
        contract: Option<Uint128>,
        // Msg for tx
        msg: Option<String>,
        coins: Option<Vec<Coin>>,
        padding: Option<String>,
    },

    // Proposal interaction
    /// Triggers the proposal when the MSG is approved
    Trigger {
        proposal: Uint128,
        padding: Option<String>,
    },
    /// Cancels the proposal if the msg keeps failing
    Cancel {
        proposal: Uint128,
        padding: Option<String>,
    },
    /// Forces a proposal update,
    /// proposals automatically update on interaction
    /// but this is a cheaper alternative
    Update {
        proposal: Uint128,
        padding: Option<String>,
    },
    /// Funds a proposal, msg is a prop ID
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    ClaimFunding {
        id: Uint128,
    },
    /// Votes on a assembly vote
    AssemblyVote {
        proposal: Uint128,
        vote: Vote,
        padding: Option<String>,
    },
    /// Votes on voting token
    ReceiveBalance {
        sender: Addr,
        msg: Option<Binary>,
        balance: Uint128,
        memo: Option<String>,
    },

    // Assemblies
    /// Creates a proposal under a assembly
    AssemblyProposal {
        assembly: Uint128,
        title: String,
        metadata: String,

        // Optionals, if none the proposal is assumed to be a text proposal
        msgs: Option<Vec<ProposalMsg>>,
        padding: Option<String>,
    },

    /// Creates a new assembly
    AddAssembly {
        name: String,
        metadata: String,
        members: Vec<Addr>,
        profile: Uint128,
        padding: Option<String>,
    },
    /// Edits an existing assembly
    SetAssembly {
        id: Uint128,
        name: Option<String>,
        metadata: Option<String>,
        members: Option<Vec<Addr>>,
        profile: Option<Uint128>,
        padding: Option<String>,
    },

    // AssemblyMsgs
    /// Creates a new assembly message and its allowed users
    AddAssemblyMsg {
        name: String,
        msg: String,
        assemblies: Vec<Uint128>,
        padding: Option<String>,
    },
    /// Edits an existing assembly msg
    SetAssemblyMsg {
        id: Uint128,
        name: Option<String>,
        msg: Option<String>,
        assemblies: Option<Vec<Uint128>>,
        padding: Option<String>,
    },
    AddAssemblyMsgAssemblies {
        id: Uint128,
        assemblies: Vec<Uint128>,
    },

    // Profiles
    /// Creates a new profile that can be added to assemblys
    AddProfile {
        profile: Profile,
        padding: Option<String>,
    },
    /// Edits an already existing profile and the assemblys using the profile
    SetProfile {
        id: Uint128,
        profile: UpdateProfile,
        padding: Option<String>,
    },

    // Contracts
    AddContract {
        name: String,
        metadata: String,
        contract: Contract,
        assemblies: Option<Vec<Uint128>>,
        padding: Option<String>,
    },
    SetContract {
        id: Uint128,
        name: Option<String>,
        metadata: Option<String>,
        contract: Option<Contract>,
        disable_assemblies: bool,
        assemblies: Option<Vec<Uint128>>,
        padding: Option<String>,
    },
    AddContractAssemblies {
        id: Uint128,
        assemblies: Vec<Uint128>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum HandleAnswer {
    SetConfig { status: ResponseStatus },
    SetRuntimeState { status: ResponseStatus },
    Proposal { status: ResponseStatus },
    ReceiveBalance { status: ResponseStatus },
    Trigger { status: ResponseStatus },
    Cancel { status: ResponseStatus },
    Update { status: ResponseStatus },
    Receive { status: ResponseStatus },
    ClaimFunding { status: ResponseStatus },
    AssemblyVote { status: ResponseStatus },
    AssemblyProposal { status: ResponseStatus },
    AddAssembly { status: ResponseStatus },
    SetAssembly { status: ResponseStatus },
    AddAssemblyMsg { status: ResponseStatus },
    SetAssemblyMsg { status: ResponseStatus },
    AddProfile { status: ResponseStatus },
    SetProfile { status: ResponseStatus },
    AddContract { status: ResponseStatus },
    SetContract { status: ResponseStatus },
}

#[cw_serde]
pub enum QueryMsg {
    // TODO: Query individual user vote with VK and permit
    Config {},

    TotalProposals {},

    Proposals { start: Uint128, end: Uint128 },

    TotalAssemblies {},

    Assemblies { start: Uint128, end: Uint128 },

    TotalAssemblyMsgs {},

    AssemblyMsgs { start: Uint128, end: Uint128 },

    TotalProfiles {},

    Profiles { start: Uint128, end: Uint128 },

    TotalContracts {},

    Contracts { start: Uint128, end: Uint128 },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },

    Proposals { props: Vec<Proposal> },

    Assemblies { assemblies: Vec<Assembly> },

    AssemblyMsgs { msgs: Vec<AssemblyMsg> },

    Profiles { profiles: Vec<Profile> },

    Contracts { contracts: Vec<AllowedContract> },

    Total { total: Uint128 },
}
