use secret_toolkit::permit::Permit;
use serde::{Deserialize, Serialize};

use crate::{
    c_std::{to_binary, Addr, Binary, Coin, CosmosMsg, StdResult, Uint128, Uint256, WasmMsg},
    lb_libraries::lb_token::{
        expiration::Expiration,
        metadata::Metadata,
        permissions::{Permission, PermissionKey},
        state_structs::{CurateTokenId, LbPair, OwnerBalance, StoredTokenInfo, TokenAmount},
        txhistory::Tx,
    },
    schemars::JsonSchema,
    utils::{ExecuteCallback, InstantiateCallback, Query},
};

/////////////////////////////////////////////////////////////////////////////////
// Init messages
/////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)] //PartialEq
pub struct InstantiateMsg {
    /// if `false` the contract will instantiate permanently as a no-admin (permissionless) contract
    pub has_admin: bool,
    /// if `admin` == `None` && `has_admin` == `true`, the instantiator will be admin
    /// if `has_admin` == `false`, this field will be ignore (ie: there will be no admin)
    pub admin: Option<Addr>,
    /// sets initial list of curators, which can create new token_ids
    pub curators: Vec<Addr>,

    /// for `create_viewing_key` function
    pub entropy: String,
    pub lb_pair_info: LbPair,
    pub initial_tokens: Vec<CurateTokenId>,
}
impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

/////////////////////////////////////////////////////////////////////////////////
// Handle Messages
/////////////////////////////////////////////////////////////////////////////////

/// Handle messages to SNIP1155 contract.
///
/// Mostly responds with `HandleAnswer { <variant_name>: { status: success }}` if successful.
/// See [HandleAnswer](crate::msg::HandleAnswer) for the response messages for each variant.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // /// curates new token_ids. Only curators can access this function.
    // CurateTokenIds {
    //     initial_tokens: Vec<CurateTokenId>,
    //     memo: Option<String>,
    //     padding: Option<String>,
    // },
    /// mints additional tokens of existing fungible token_ids, if configuration allows this, ie
    /// `enable_mint == true`.
    /// Only minters can access this function
    MintTokens {
        mint_tokens: Vec<TokenAmount>,
        memo: Option<String>,
        padding: Option<String>,
    },
    /// burns existing tokens, if configuration allows this, ie
    /// `enable_burn == true`.
    /// Only owners can burn their own tokens in the base specifications. Flexibility is built
    /// into the contract functions to allow other addresses to burn tokens, allowed in additional specifications.
    BurnTokens {
        burn_tokens: Vec<TokenAmount>,
        memo: Option<String>,
        padding: Option<String>,
    },
    /// allows owner or minter to change metadata if allowed by token_id configuration.
    ChangeMetadata {
        token_id: String,
        /// does not attempt to change if left blank. Can effectively remove metadata by setting
        /// metadata to `Some(Metadata {token_uri: None, extension: None})`
        /// used Box<T> to reduce the total size of the enum variant, to decrease size difference
        /// between variants. Not strictly necessary.
        public_metadata: Box<Option<Metadata>>,
        /// does not attempt to change if left blank. Can effectively remove metadata by setting
        /// metadata to `Some(Metadata {token_uri: None, extension: None})`
        /// used Box<T> to reduce the total size of the enum variant, to decrease size difference
        /// between variants. Not strictly necessary.
        private_metadata: Box<Option<Metadata>>,
    },
    /// transfers one or more tokens of a single token_id. Other third address can perform this function
    /// if it has permission to transfer. ie: if addr3 can call this function to transfer tokens from addr0
    /// to addr2, if addr0 gives addr3 enough transfer allowance.
    Transfer {
        token_id: String,
        // equivalent to `owner` in SNIP20. Tokens are sent from this address.
        from: Addr,
        recipient: Addr,
        amount: Uint256,
        memo: Option<String>,
        padding: Option<String>,
    },
    /// performs `transfer`s of multiple token_ids in a single transaction
    BatchTransfer {
        actions: Vec<TransferAction>,
        padding: Option<String>,
    },
    /// similar to transfer, but also sends a cosmos message. The recipient needs to be a contract that
    /// has a SNIP1155Receive handle function. See [receiver](crate::receiver) for more information.
    Send {
        token_id: String,
        // equivalent to `owner` in SNIP20. Tokens are sent from this address.
        from: Addr,
        recipient: Addr,
        recipient_code_hash: Option<String>,
        amount: Uint256,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    /// performs `send` of multiple token_ids in a single transaction
    BatchSend {
        actions: Vec<SendAction>,
        padding: Option<String>,
    },
    /// allows an owner of token_ids to change transfer or viewership permissions to other addresses.  
    ///
    /// The base specification has three types of permissions:
    /// * view balance permission: owner can allow another address to view owner's balance of specific token_ids
    /// * view private metadata: owner can allow another address to view private metadata of specific token_ids
    /// * transfer allowance: owner can give permission to another address to transfer tokens up to a certain limit (cumulatively)
    /// Owners can set an [expiry](crate::state::expiration) for each of these permissions.
    ///
    /// SNIP1155 gives flexibility for permissions to have any combination of
    /// * type of permission granted
    /// * on which token_ids
    GivePermission {
        /// address being granted/revoked permission
        allowed_address: Addr,
        /// token id to apply approval/revocation to.
        /// Additional Spec feature: if == None, perform action for all owner's `token_id`s
        token_id: String,
        /// optional permission level for viewing balance. If ignored, leaves current permission settings
        view_balance: Option<bool>,
        view_balance_expiry: Option<Expiration>,
        /// optional permission level for viewing private metadata. If ignored, leaves current permission settings
        view_private_metadata: Option<bool>,
        view_private_metadata_expiry: Option<Expiration>,
        /// set allowance by for transfer approvals. If ignored, leaves current permission settings
        transfer: Option<Uint256>,
        transfer_expiry: Option<Expiration>,
        /// optional message length padding
        padding: Option<String>,
    },
    /// Removes all permissions that a specific owner has granted to a specific address, for a specific token_id.
    /// A permission grantee can use this function to renounce a permission it has been given.
    /// For owners, the `GivePermission` message can be used instead to have the same effect as `RevokePermission`.
    RevokePermission {
        token_id: String,
        /// token owner
        owner: Addr,
        /// address which has permission
        allowed_address: Addr,
        padding: Option<String>,
    },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
    /// disallow the use of a query permit
    RevokePermit {
        permit_name: String,
        padding: Option<String>,
    },
    // AddCurators {
    //     add_curators: Vec<Addr>,
    //     padding: Option<String>,
    // },
    // RemoveCurators {
    //     remove_curators: Vec<Addr>,
    //     padding: Option<String>,
    // },
    // AddMinters {
    //     token_id: String,
    //     add_minters: Vec<Addr>,
    //     padding: Option<String>,
    // },
    // RemoveMinters {
    //     token_id: String,
    //     remove_minters: Vec<Addr>,
    //     padding: Option<String>,
    // },
    // ChangeAdmin {
    //     new_admin: Addr,
    //     padding: Option<String>,
    // },
    // /// Permanently breaks admin keys for this contract. No admin function can be called after this
    // /// action. Any existing curators or minters will remain as curators or minters; no new curators can be
    // /// added and no current curator can be removed.
    // ///
    // /// Requires caller to input current admin address and contract address. These inputs are not strictly
    // /// necessary, but as a safety precaution to reduce the chances of accidentally calling this function.
    // RemoveAdmin {
    //     current_admin: Addr,
    //     contract_address: Addr,
    //     padding: Option<String>,
    // },
    RegisterReceive {
        code_hash: String,
        padding: Option<String>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

impl ExecuteMsg {
    pub fn to_cosmos_msg(
        &self,
        code_hash: String,
        contract_addr: String,
        send_amount: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        let mut msg = to_binary(self)?;
        space_pad(256, &mut msg.0);
        let mut funds = Vec::new();
        if let Some(amount) = send_amount {
            funds.push(Coin {
                amount,
                denom: String::from("uscrt"),
            });
        }
        let execute = WasmMsg::Execute {
            contract_addr,
            code_hash,
            msg,
            funds,
        };
        Ok(execute.into())
    }
}

/// Handle answers in the `data` field of `HandleResponse`. See
/// [HandleMsg](crate::msg::HandleMsg), which has more details
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteAnswer {
    CurateTokenIds { status: ResponseStatus },
    MintTokens { status: ResponseStatus },
    BurnTokens { status: ResponseStatus },
    ChangeMetadata { status: ResponseStatus },
    Transfer { status: ResponseStatus },
    BatchTransfer { status: ResponseStatus },
    Send { status: ResponseStatus },
    BatchSend { status: ResponseStatus },
    GivePermission { status: ResponseStatus },
    RevokePermission { status: ResponseStatus },
    CreateViewingKey { key: String },
    SetViewingKey { status: ResponseStatus },
    RevokePermit { status: ResponseStatus },
    AddCurators { status: ResponseStatus },
    RemoveCurators { status: ResponseStatus },
    AddMinters { status: ResponseStatus },
    RemoveMinters { status: ResponseStatus },
    ChangeAdmin { status: ResponseStatus },
    RemoveAdmin { status: ResponseStatus },
    RegisterReceive { status: ResponseStatus },
}

/////////////////////////////////////////////////////////////////////////////////
// Query messages
/////////////////////////////////////////////////////////////////////////////////

/// Query messages to SNIP1155 contract. See [QueryAnswer](crate::msg::QueryAnswer)
/// for the response messages for each variant, which has more detail.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// returns public information of the SNIP1155 contract
    TokenContractInfo {},
    IdTotalBalance {
        id: String,
    },
    Balance {
        owner: Addr,
        viewer: Addr,
        key: String,
        token_id: String,
    },
    AllBalances {
        owner: Addr,
        key: String,
        tx_history_page: Option<u32>,
        tx_history_page_size: Option<u32>,
    },
    TransactionHistory {
        address: Addr,
        key: String,
        page: Option<u32>,
        page_size: u32,
    },
    Permission {
        owner: Addr,
        allowed_address: Addr,
        key: String,
        token_id: String,
    },
    /// displays all permissions that a given address has granted
    AllPermissions {
        /// address that has granted permissions to others
        address: Addr,
        key: String,
        page: Option<u32>,
        page_size: u32,
    },
    TokenIdPublicInfo {
        token_id: String,
    },
    TokenIdPrivateInfo {
        address: Addr,
        key: String,
        token_id: String,
    },
    RegisteredCodeHash {
        contract: Addr,
    },
    WithPermit {
        permit: Permit,
        query: QueryWithPermit,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> StdResult<(Vec<&Addr>, String)> {
        match self {
            Self::Balance {
                owner, viewer, key, ..
            } => Ok((vec![owner, viewer], key.clone())),
            Self::AllBalances { owner, key, .. } => Ok((vec![owner], key.clone())),
            Self::TransactionHistory { address, key, .. } => Ok((vec![address], key.clone())),
            Self::Permission {
                owner,
                allowed_address,
                key,
                ..
            } => Ok((vec![owner, allowed_address], key.clone())),
            Self::AllPermissions { address, key, .. } => Ok((vec![address], key.clone())),
            Self::TokenIdPrivateInfo { address, key, .. } => Ok((vec![address], key.clone())),
            Self::TokenContractInfo {}
            | Self::IdTotalBalance { .. }
            | Self::TokenIdPublicInfo { .. }
            | Self::RegisteredCodeHash { .. }
            | Self::WithPermit { .. } => {
                unreachable!("This query type does not require viewing key authentication")
            }
        }
    }
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    Balance {
        owner: Addr,
        token_id: String,
    },
    AllBalances {
        tx_history_page: Option<u32>,
        tx_history_page_size: Option<u32>,
    },
    TransactionHistory {
        page: Option<u32>,
        page_size: u32,
    },
    Permission {
        owner: Addr,
        allowed_address: Addr,
        token_id: String,
    },
    AllPermissions {
        page: Option<u32>,
        page_size: u32,
    },
    TokenIdPrivateInfo {
        token_id: String,
    },
}

/// the query responses for each [QueryMsg](crate::msg::QueryMsg) variant
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    /// returns contract-level information:
    TokenContractInfo {
        // the address of the admin, or `None` for an admin-free contract
        admin: Option<Addr>,
        /// the list of curators in the contract
        curators: Vec<Addr>,
        /// the list of all token_ids that have been curated
        all_token_ids: Vec<String>,
    },
    IdTotalBalance {
        amount: Uint256,
    },
    /// returns balance of a specific token_id. Owners can give permission to other addresses to query their balance
    Balance {
        amount: Uint256,
    },
    /// returns all token_id balances owned by an address. Only owners can use this query
    AllBalances(Vec<OwnerBalance>),
    /// all permissions related to a particular address. Note that "curation" is not recorded as a transaction per se, but
    /// the tokens minted as part of the initial_balances set by the curator is recorded under `TxAction::Mint`  
    TransactionHistory {
        txs: Vec<Tx>,
        total: u64,
    },
    Permission(Option<Permission>),
    /// all permissions granted, viewable by the permission granter.
    /// Users or applications can match the permission_keys that corresponds to each permission as
    /// they have a similar order, ie: the index of `permission_keys` vector corresponds to the index
    /// of the `permissions` vector.
    AllPermissions {
        permission_keys: Vec<PermissionKey>,
        permissions: Vec<Permission>,
        /// the total number of permission entries stored for a given granter, which may include "blank"
        /// permissions, ie: where all permissions are set to `false` or `Uint256(0)`
        total: u64,
    },
    TokenIdPublicInfo {
        /// token_id_info.private_metadata will always = None
        token_id_info: StoredTokenInfo,
        /// if public_total_supply == false, total_supply = None
        total_supply: Option<Uint256>,
        /// if owner_is_public == false, total_supply = None
        owner: Option<Addr>,
    },
    TokenIdPrivateInfo {
        token_id_info: StoredTokenInfo,
        /// if public_total_supply == false, total_supply = None
        total_supply: Option<Uint256>,
        /// if owner_is_public == false, total_supply = None
        owner: Option<Addr>,
    },
    /// returns None if contract has not registered with SNIP1155 contract
    RegisteredCodeHash {
        code_hash: Option<String>,
    },
    /// returned when an viewing_key-specific errors occur during a user's attempt to
    /// perform an authenticated query
    ViewingKeyError {
        msg: String,
    },
}

/////////////////////////////////////////////////////////////////////////////////
// Structs, Enums and other functions
/////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TransferAction {
    pub token_id: String,
    // equivalent to `owner` in SNIP20. Tokens are sent from this address.
    pub from: Addr,
    pub recipient: Addr,
    pub amount: Uint256,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SendAction {
    pub token_id: String,
    // equivalent to `owner` in SNIP20. Tokens are sent from this address.
    pub from: Addr,
    pub recipient: Addr,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint256,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

// Take a Vec<u8> and pad it up to a multiple of `block_size`, using spaces at the end.
pub fn space_pad(block_size: usize, message: &mut Vec<u8>) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}
