pub mod assembly;
pub mod contract;
pub mod errors;
pub mod profile;
pub mod proposal;
#[cfg(feature = "governance-impl")]
pub mod stored_id;
pub mod vote;

use crate::{
    c_std::{Addr, Binary, Uint128},
    contract_interfaces::governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::{Profile, UpdateProfile},
        proposal::{Proposal, ProposalMsg},
        vote::Vote,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::{
    governance::proposal::Funding,
    query_auth::QueryPermit,
    utils::{ExecuteCallback, InstantiateCallback, Query},
};
use cosmwasm_schema::cw_serde;
use secret_storage_plus::{Item, Json};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::plus::ItemStorage;

// TODO: add errors

// Admin command variable spot
pub const MSG_VARIABLE: &str = "{~}";

#[cw_serde]
pub struct Config {
    pub query: Contract,
    pub treasury: Addr,
    // When public voting is enabled, a voting token is expected
    pub vote_token: Option<Contract>,
    // When funding is enabled, a funding token is expected
    pub funding_token: Option<Contract>,

    // Migration information
    pub migrated_from: Option<Contract>,
    pub migrated_to: Option<Contract>,
}

#[cfg(feature = "governance-impl")]
impl ItemStorage for Config {
    const ITEM: Item<'static, Self, Json> = Item::new("config-");
}

// Used for original instantiation
#[cw_serde]
pub struct AssemblyInit {
    pub admin_members: Vec<Addr>,
    pub admin_profile: Profile,
    pub public_profile: Profile,
}

// Used for migration instantiation
#[cw_serde]
pub struct MigrationInit {
    pub source: Contract,
    pub assembly: u16,
    pub assembly_msg: u16,
    pub profile: u16,
    pub contract: u16,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub treasury: Addr,
    pub query_auth: Contract,

    // Admin rules
    pub assemblies: Option<AssemblyInit>,

    // Token rules
    pub funding_token: Option<Contract>,
    pub vote_token: Option<Contract>,

    // Migration data
    pub migrator: Option<MigrationInit>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum RuntimeState {
    // Run like normal
    Normal,
    // Allow only specific assemblies and admin
    SpecificAssemblies { assemblies: Vec<u16> },
    // Migrated - points to the new version
    Migrated,
}

#[cfg(feature = "governance-impl")]
impl ItemStorage for RuntimeState {
    const ITEM: Item<'static, Self, Json> = Item::new("runtime-state-");
}

#[cw_serde]
pub enum MigrationDataAsk {
    Assembly,
    AssemblyMsg,
    Profile,
    Contract,
}

#[cw_serde]
pub enum MigrationData {
    Assembly { data: Vec<(u16, Assembly)> },
    AssemblyMsg { data: Vec<(u16, AssemblyMsg)> },
    Profile { data: Vec<(u16, Profile)> },
    Contract { data: Vec<(u16, AllowedContract)> },
}

#[cw_serde]
pub enum ExecuteMsg {
    // Internal config
    SetConfig {
        query_auth: Option<Contract>,
        treasury: Option<Addr>,
        funding_token: Option<Contract>,
        vote_token: Option<Contract>,
        padding: Option<String>,
    },
    SetRuntimeState {
        state: RuntimeState,
        padding: Option<String>,
    },

    // Proposal interaction
    /// Triggers the proposal when the MSG is approved
    Trigger {
        //TODO: Must be deprecated for v1
        proposal: u32,
        padding: Option<String>,
    },
    /// Cancels the proposal if the msg keeps failing
    Cancel {
        //TODO: Must be deprecated for v1
        proposal: u32,
        padding: Option<String>,
    },
    /// Forces a proposal update,
    /// proposals automatically update on interaction
    /// but this is a cheaper alternative
    Update {
        proposal: u32,
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
        id: u32,
    },
    /// Votes on a assembly vote
    AssemblyVote {
        proposal: u32,
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
        assembly: u16,
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
        profile: u16,
        padding: Option<String>,
    },
    /// Edits an existing assembly
    SetAssembly {
        id: u16,
        name: Option<String>,
        metadata: Option<String>,
        members: Option<Vec<Addr>>,
        profile: Option<u16>,
        padding: Option<String>,
    },

    // AssemblyMsgs
    /// Creates a new assembly message and its allowed users
    AddAssemblyMsg {
        name: String,
        msg: String,
        assemblies: Vec<u16>,
        padding: Option<String>,
    },
    /// Edits an existing assembly msg
    SetAssemblyMsg {
        id: u16,
        name: Option<String>,
        msg: Option<String>,
        assemblies: Option<Vec<u16>>,
        padding: Option<String>,
    },
    AddAssemblyMsgAssemblies {
        id: u16,
        assemblies: Vec<u16>,
    },

    // Profiles
    /// Creates a new profile that can be added to assemblys
    AddProfile {
        profile: Profile,
        padding: Option<String>,
    },
    /// Edits an already existing profile and the assemblys using the profile
    SetProfile {
        id: u16,
        profile: UpdateProfile,
        padding: Option<String>,
    },

    // Contracts
    AddContract {
        name: String,
        metadata: String,
        contract: Contract,
        assemblies: Option<Vec<u16>>,
        padding: Option<String>,
    },
    SetContract {
        id: u16,
        name: Option<String>,
        metadata: Option<String>,
        contract: Option<Contract>,
        disable_assemblies: bool,
        assemblies: Option<Vec<u16>>,
        padding: Option<String>,
    },
    AddContractAssemblies {
        id: u16,
        assemblies: Vec<u16>,
    },
    // Migrations
    // Export total numeric IDs
    // Committee, msg, profile and contract keys must be exported
    // Create a struct that stores the last migrated IDs
    // Enum for migration targets
    // migrate gives an array of items with their appropriate IDs
    // Migrate Committee, Msg, Profile and Contract
    // When receiving migration data, if given ID is greater then ignore

    // Add functions for exporting lists of data into the needed contracts
    Migrate {
        id: u64,
        label: String,
        code_hash: String,
    },
    MigrateData {
        data: MigrationDataAsk,
        total: u16,
    },
    ReceiveMigrationData {
        data: MigrationData,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
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
    AddContractAssemblies { status: ResponseStatus },
    Migrate { status: ResponseStatus },
    MigrateData { status: ResponseStatus },
    ReceiveMigrationData { status: ResponseStatus },
}

#[cw_serde]
pub struct Pagination {
    pub page: u16,
    pub amount: u32,
}

#[cw_serde]
pub enum AuthQuery {
    Proposals { pagination: Pagination },
    AssemblyVotes { pagination: Pagination },
    Funding { pagination: Pagination },
    Votes { pagination: Pagination },
}

#[remain::sorted]
#[cw_serde]
pub struct QueryData {}

#[cw_serde]
pub enum QueryMsg {
    Config {},

    TotalProposals {},

    Proposals {
        start: u32,
        end: u32,
    },

    TotalAssemblies {},

    Assemblies {
        start: u16,
        end: u16,
    },

    TotalAssemblyMsgs {},

    AssemblyMsgs {
        start: u16,
        end: u16,
    },

    TotalProfiles {},

    Profiles {
        start: u16,
        end: u16,
    },

    TotalContracts {},

    Contracts {
        start: u16,
        end: u16,
    },

    WithVK {
        user: Addr,
        key: String,
        query: AuthQuery,
    },

    WithPermit {
        permit: QueryPermit,
        query: AuthQuery,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct ResponseWithID<T> {
    pub prop_id: u32,
    pub data: T,
}

#[cw_serde]
pub enum QueryAnswer {
    Config {
        config: Config,
    },

    Proposals {
        props: Vec<Proposal>,
    },

    Assemblies {
        assemblies: Vec<Assembly>,
    },

    AssemblyMsgs {
        msgs: Vec<AssemblyMsg>,
    },

    Profiles {
        profiles: Vec<Profile>,
    },

    Contracts {
        contracts: Vec<AllowedContract>,
    },

    Total {
        total: u32,
    },

    UserProposals {
        props: Vec<ResponseWithID<Proposal>>,
        total: u32,
    },

    UserAssemblyVotes {
        votes: Vec<ResponseWithID<Vote>>,
        total: u32,
    },

    UserFunding {
        funds: Vec<ResponseWithID<Funding>>,
        total: u32,
    },

    UserVotes {
        votes: Vec<ResponseWithID<Vote>>,
        total: u32,
    },
}
