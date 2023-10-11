use std::collections::HashMap;

use crate::msg::{
    BalanceResponse, ExecuteMsg, FundsResponse, InstantiateMsg, MultiplierResponse, QueryMsg,
    TokenInfoResponse,
};
use crate::multitest::controller::Controller;
use crate::multitest::receiver::{QueryResp as ReceiverQueryResp, Receiver};
use anyhow::{anyhow, Result as AnyResult};

use shade_multi_test::multi::snip20::Snip20;
use shade_protocol::{
    c_std::{Addr, Binary, Coin as StdCoin, Decimal, Empty, Uint128},
    contract_interfaces::snip20::Snip20ReceiveMsg,
    multi_test::{App, AppResponse, BasicAppBuilder, Contract, ContractWrapper, Executor},
    secret_storage_plus::Item,
    snip20
};

use utils::{coin::Coin, token::Token};

pub const VIEWING_KEY: &str = "viewing_key";

fn contract_token() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );

    Box::new(contract)
}

pub fn init_snip20(chain: &App, init_msg: snip20::InstantiateMsg) -> ContractInfo {
    let snip20 = chain
        .instantiate_contract(stored_code, None, &init_msg, &[], "admin", None)
        .unwrap();

    snip20
}

/// Builder for test suite
#[derive(Debug)]
pub struct SuiteBuilder {
    /// WyndLend token name
    name: String,
    /// WyndLend token symbol
    symbol: String,
    /// WyndLend token precision
    decimals: u8,
    /// Amount of tokens controller would allow to transfer
    transferable: HashMap<String, Uint128>,
    /// Token distributed by this contract
    distributed_token: snip20::InstantiateMsg,
    /// Initial funds of native tokens
    funds: Vec<(Addr, Vec<StdCoin>)>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            name: "wynd_lend".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            transferable: HashMap::new(),
            distributed_token: snip20::InstantiateMsg {
                name: "Distribution Token".to_string(),
                admin: None,
                query_auth: None,
                symbol: "DIST".to_string(),
                decimals: decimals as u8,
                initial_balances: None,
                prng_seed: Binary::default(),
                config: None,
            },
            funds: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_symbol(mut self, symbol: impl ToString) -> Self {
        self.symbol = symbol.to_string();
        self
    }

    pub fn with_decimals(mut self, decimals: u8) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn with_transferable(mut self, sender: impl ToString, amount: Uint128) -> Self {
        *self.transferable.entry(sender.to_string()).or_default() += amount;
        self
    }


    pub fn with_distributed_cw20_token(
        mut self,
        decimals: u8,
        initial_balances: Vec<snip20::InitialBalance>,
    ) -> Self {
        self.distributed_token = snip20::InstantiateMsg {
                name: "Distribution Token".to_string(),
                admin: None,
                query_auth: None,
                symbol: "DIST".to_string(),
                decimals: decimals as u8,
                initial_balances: None,
                prng_seed: Binary::default(),
                config: None,
        }; // will be set when contract is instantiated
        self
    }

    pub fn with_funds(mut self, addr: &str, tokens: impl IntoIterator<Item = StdCoin>) -> Self {
        self.funds
            .push((Addr::unchecked(addr), tokens.into_iter().collect()));
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let funds = self.funds;
        let mut app = BasicAppBuilder::new().build(move |router, _api, storage| {
            for (addr, tokens) in funds {
                router.bank.init_balance(storage, &addr, tokens).unwrap();
            }
        });
        let owner = Addr::unchecked("owner");

        let controller_contract = Controller::new(self.transferable);
        let controller_id = app.store_code(Box::new(controller_contract));
        let controller = app
            .instantiate_contract(
                controller_id,
                owner.clone(),
                &Empty {},
                &[],
                "Controller",
                None,
            )
            .unwrap();

        let snip20_id = app.store_code(Snip20::default().contract());
        let distributed_token = init_snip20(&app, self.distributed_token);

        let token_id = app.store_code(contract_token());
        let token = app
            .instantiate_contract(
                token_id,
                owner.clone(),
                &InstantiateMsg {
                    name: self.name,
                    symbol: self.symbol,
                    decimals: self.decimals,
                    controller,
                    distributed_token: distributed_token.clone(),
                    viewing_key: viewing_key.to_string(),
                },
                &[],
                "WyndLend",
                None,
            )
            .unwrap();

        let receiver_id = app.store_code(Box::new(Receiver::new()));
        let receiver = app
            .instantiate_contract(receiver_id, owner, &Empty {}, &[], "Receiver", None)
            .unwrap();

        Suite {
            app,
            controller: controller.address,
            token: token.address,
            receiver: token.address,
            distributed_token: distributed_token.address,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Address of controller contract
    controller: Addr,
    /// Address of token contract
    token: Addr,
    /// The token that is distributed by the contract
    distributed_token: Token,
    /// Address of cw1 contract
    receiver: Addr,
}

impl Suite {
    /// Builds test suite with default configuration
    pub fn new() -> Self {
        SuiteBuilder::new().build()
    }

    /// Gives controller address back
    pub fn controller(&self) -> Addr {
        self.controller.clone()
    }

    /// Gives receiver address back
    pub fn receiver(&self) -> Addr {
        self.receiver.clone()
    }

    /// Gives token address back
    pub fn token(&self) -> Addr {
        self.token.clone()
    }

    /// Gives distributed token back
    pub fn distributed_token(&self) -> Token {
        self.distributed_token.clone()
    }

    /// Executes transfer on token contract
    pub fn transfer(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::Transfer {
                    recipient: recipient.to_owned(),
                    amount,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes transfer using base amount on token contract
    pub fn transfer_base(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                self.controller.clone(),
                self.token.clone(),
                &ExecuteMsg::TransferBaseFrom {
                    sender: sender.to_owned(),
                    recipient: recipient.to_owned(),
                    amount,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes send on token contract
    pub fn send(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
        msg: Binary,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::Send {
                    contract: recipient.to_owned(),
                    amount,
                    msg,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes mint on token contract
    pub fn mint(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::Mint {
                    recipient: recipient.to_owned(),
                    amount,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes mint using base amount on token contract
    pub fn mint_base(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::MintBase {
                    recipient: recipient.to_owned(),
                    amount,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes burn on token contract
    pub fn burn(&mut self, sender: &str, account: &str, amount: Uint128) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::BurnFrom {
                    owner: account.to_string(),
                    amount,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes burn using base amount on token contract
    pub fn burn_base(
        &mut self,
        sender: &str,
        account: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::BurnBaseFrom {
                    owner: account.to_string(),
                    amount,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes rebase on token contract
    pub fn rebase(&mut self, executor: &str, ratio: Decimal) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(executor),
                self.token.clone(),
                &ExecuteMsg::Rebase { ratio },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes distribute on token contract
    pub fn distribute<'a>(
        &mut self,
        executor: &str,
        sender: impl Into<Option<&'a str>>,
        funds: &[StdCoin],
    ) -> AnyResult<AppResponse> {
        let sender = sender.into().map(str::to_owned);
        self.app
            .execute_contract(
                Addr::unchecked(executor),
                self.token.clone(),
                &ExecuteMsg::Distribute { sender },
                funds,
            )
            .map_err(|err| anyhow!(err))
    }

    /// Execute withdraw_funds on token contract
    pub fn withdraw_funds(&mut self, executor: &str) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(executor),
                self.token.clone(),
                &ExecuteMsg::WithdrawFunds {},
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Queries token contract for balance
    pub fn query_base_balance(&self, address: &str) -> AnyResult<Uint128> {
        let resp: BalanceResponse = self.app.wrap().query_wasm_smart(
            self.token.clone(),
            &QueryMsg::BaseBalance {
                address: address.to_owned(),
            },
        )?;
        Ok(resp.balance)
    }

    /// Queries token contract for balance
    pub fn query_balance(&self, address: &str) -> AnyResult<Uint128> {
        let resp: BalanceResponse = self.app.wrap().query_wasm_smart(
            self.token.clone(),
            &QueryMsg::Balance {
                address: address.to_owned(),
            },
        )?;
        Ok(resp.balance)
    }

    /// Queries token contract for token info
    pub fn query_token_info(&self) -> AnyResult<TokenInfoResponse> {
        self.app
            .wrap()
            .query_wasm_smart(self.token.clone(), &QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }

    /// Queries receiver for count of valid messages it received
    pub fn query_receiver(&self) -> AnyResult<u128> {
        let resp: ReceiverQueryResp = self
            .app
            .wrap()
            .query_wasm_smart(self.receiver.clone(), &Empty {})
            .map_err(|err| anyhow!(err))?;

        Ok(resp.counter.into())
    }

    /// Queries multiplier
    pub fn query_multiplier(&self) -> AnyResult<Decimal> {
        let resp: MultiplierResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.token.clone(), &QueryMsg::Multiplier {})
            .map_err(|err| anyhow!(err))?;

        Ok(resp.multiplier)
    }

    /// Queries distributed funds
    pub fn query_distributed_funds(&self) -> AnyResult<Coin> {
        let resp: FundsResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.token.clone(), &QueryMsg::DistributedFunds {})
            .map_err(|err| anyhow!(err))?;

        Ok(resp.funds)
    }

    /// Queries undistributed funds
    pub fn query_undistributed_funds(&self) -> AnyResult<Coin> {
        let resp: FundsResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.token.clone(), &QueryMsg::UndistributedFunds {})
            .map_err(|err| anyhow!(err))?;

        Ok(resp.funds)
    }

    /// Queries withdrawable funds
    pub fn query_withdrawable_funds(&self, addr: &str) -> AnyResult<Coin> {
        let resp: FundsResponse = self
            .app
            .wrap()
            .query_wasm_smart(
                self.token.clone(),
                &QueryMsg::WithdrawableFunds {
                    owner: addr.to_owned(),
                },
            )
            .map_err(|err| anyhow!(err))?;

        Ok(resp.funds)
    }

    /// Queries for balance of native token
    pub fn native_balance(&self, addr: &str, token: &str) -> AnyResult<u128> {
        let amount = self
            .app
            .wrap()
            .query_balance(Addr::unchecked(addr), token)?
            .amount;
        Ok(amount.into())
    }

    /// Sends the given amount of snip20 token from `sender` to the token contract
    pub fn snip20_send_to_token_contract(
        &mut self,
        snip20_contract: &ContractInfo,
        sender: &str,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            snip20_contract,
            &snip20::ExecuteMsg::Send {
                recipient: self.token.to_string(),
                recipient_code_hash: None,
                amount: amount.into(),
                msg: None,
                memo: None,
                padding: None,
            },
            &[],
        )
    }

    /// Queries the balance of the given cw20 token of the address
    pub fn snip20_balance(&mut self, snip20_contract: &str, address: &str) -> AnyResult<u128> {
        Ok(self
            .app
            .wrap()
            .query::<snip20::QueryAnswer::Balance>(
                &snip20::QueryMsg::Balance {
                    address: address.to_string(),
                    key: VIEWING_KEY.to_string(),
                },
            )?
            .u128())
    }
}
