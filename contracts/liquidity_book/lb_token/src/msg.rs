use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, Uint128, Uint256, WasmMsg};

use libraries::transfer::space_pad;

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub lb_pair: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    ApproveForAll {
        spender: Addr,
        approved: bool,
    },
    BatchTransferFrom {
        from: Addr,
        to: Addr,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    },
    Mint {
        recipient: Addr,
        id: u32,
        amount: Uint256,
    },
    Burn {
        owner: Addr,
        id: u32,
        amount: Uint256,
    },
}

impl ExecuteMsg {
    /// Returns a StdResult<CosmosMsg> used to execute a SNIP20 contract function
    ///
    /// # Arguments
    ///
    /// * `block_size` - pad the message to blocks of this size
    /// * `callback_code_hash` - String holding the code hash of the contract being called
    /// * `contract_addr` - address of the contract being called
    /// * `send_amount` - Optional Uint128 amount of native coin to send with the callback message
    ///                 NOTE: Only a Deposit message should have an amount sent with it
    pub fn to_cosmos_msg(
        &self,
        code_hash: String,
        contract_addr: String,
        send_amount: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, 256);
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(NameResponse)]
    Name {},
    #[returns(SymbolResponse)]
    Symbol {},
    #[returns(DecimalsResponse)]
    Decimals {},
    #[returns(TotalSupplyResponse)]
    TotalSupply { id: u32 },
    #[returns(BalanceOfResponse)]
    BalanceOf { owner: Addr, id: u32 },
    #[returns(BalanceOfBatchResponse)]
    BalanceOfBatch { owners: Vec<Addr>, ids: Vec<u32> },
    #[returns(IsApprovedForAllResponse)]
    IsApprovedForAll { owner: Addr, spender: Addr },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct NameResponse {
    pub name: String,
}

#[cw_serde]
pub struct SymbolResponse {
    pub symbol: String,
}

#[cw_serde]
pub struct DecimalsResponse {
    pub decimals: u8,
}

#[cw_serde]
pub struct TotalSupplyResponse {
    pub total_supply: Uint256,
}

#[cw_serde]
pub struct BalanceOfResponse {
    pub balance: Uint256,
}

#[cw_serde]
pub struct BalanceOfBatchResponse {
    pub balances: Vec<Uint256>,
}

#[cw_serde]
pub struct IsApprovedForAllResponse {
    pub is_approved: bool,
}
