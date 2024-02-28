use crate::{
    c_std::{Binary, Decimal, Uint128},
    lending_utils::{amount::token_to_base, Authentication},
    utils::{
        asset::{Contract, RawContract},
        Query,
    },
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token precision for displaying
    pub decimals: u8,
    /// Controller is contract allowed to ming, burn, rebase, and must be checked with to
    /// enable transfer. Usually it is an lend market contract.
    pub controller: RawContract,
    /// Token which will be distributed via this contract by cw2222 interface
    pub distributed_token: RawContract,
    /// Key used for reading data in queries
    pub viewing_key: String,
    /// Address of auth query contract
    pub query_auth: Contract,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Transfer is a base message to move tokens to another account without triggering actions.
    /// Requires check for transfer possibility by `ControllerQuery::CanTransfer` call to
    /// controller.
    Transfer { recipient: String, amount: Uint128 },
    /// TransferFrom allows to order transfer of tokens from source to destination.
    /// Proper authentication is in place - can be called only be controller.
    /// Requires check for transfer possibility by `ControllerQuery::CanTransfer` call to
    /// controller.
    TransferFrom {
        sender: String,
        recipient: String,
        amount: Uint128,
    },
    /// Like `TransferFrom`, but the `amount` is specified in base token amount, not amount of this token.
    ///
    /// Reserved for controller
    TransferBaseFrom {
        sender: String,
        recipient: String,
        amount: Uint128,
    },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    /// Requires check for transfer possibility by `ControllerQuery::CanTransfer` call to
    /// controller.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Reserved for controller
    Mint { recipient: String, amount: Uint128 },
    /// Like `Mint`, but the `amount` is specified in base token amount, not amount of this token.
    ///
    /// Reserved for controller
    MintBase { recipient: String, amount: Uint128 },
    /// Reserved for controller
    BurnFrom { owner: String, amount: Uint128 },
    /// Like `BurnFrom`, but the `amount` is specified in base token amount, not amount of this token.
    ///
    /// Reserved for controller
    BurnBaseFrom { owner: String, amount: Uint128 },
    /// Can only be called by the controller.
    /// multiplier *= ratio
    Rebase { ratio: Decimal },
    /// Distributed tokens using cw2222 mechanism. Tokens send with this message as distributed
    /// alongside with all tokens send until now which are not yet distributed.
    Distribute {
        /// Just for informational purposes - would overwrite message sender in generated event.
        sender: Option<String>,
    },
    /// Withdraw tokens distributed before
    WithdrawFunds {},
}

#[cw_serde]
pub enum ControllerQuery {
    TransferableAmount {
        /// Lend contract address that calls "CanTransfer"
        token: String,
        /// Address that wishes to transfer
        account: String,
    },
}

impl Query for ControllerQuery {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct TransferableAmountResp {
    pub transferable: Uint128,
}

#[cw_serde]
pub struct AuthPermit {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    #[returns(BalanceResponse)]
    Balance {
        address: String,
        authentication: Authentication,
    },
    /// Like `Balance`, but returns the amount of base tokens.
    #[returns(BalanceResponse)]
    BaseBalance {
        address: String,
        authentication: Authentication,
    },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(TokenInfoResponse)]
    TokenInfo {},
    /// Returns the global multiplier factor.
    #[returns(MultiplierResponse)]
    Multiplier {},
    /// Funds distributed by this contract.
    #[returns(FundsResponse)]
    DistributedFunds {},
    /// Funds send to this contact but not yet distributed.
    #[returns(FundsResponse)]
    UndistributedFunds {},
    /// Queries for funds distributed but not yet withdrawn by owner
    #[returns(WithdrawableFundsResponse)]
    WithdrawableFunds {
        owner: String,
        authentication: Authentication,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub multiplier: Decimal,
}

impl TokenInfoResponse {
    /// Returns the total supply, in base tokens.
    pub fn total_supply_base(&self) -> Uint128 {
        token_to_base(self.total_supply, self.multiplier)
    }
}

#[cw_serde]
pub struct MultiplierResponse {
    pub multiplier: Decimal,
}

#[cw_serde]
pub struct FundsResponse {
    pub token: Contract,
    pub amount: Uint128,
}

#[cw_serde]
pub struct WithdrawableFundsResponse {
    pub token: Contract,
    pub amount: Uint128,
}
