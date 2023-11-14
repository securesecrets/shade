use anyhow::{anyhow, Result as AnyResult};
use lending_utils::{amount::token_to_base, price::PriceRate};
use std::collections::HashMap;
use wyndex::{
    asset::{Asset, AssetInfo},
    factory::PairType,
    oracle::{SamplePeriod, HALF_HOUR},
    pair::{LsdInfo, PairInfo, QueryMsg as PairQueryMsg, StablePoolParams},
};
use wyndex_tests::builder::{WyndexSuite, WyndexSuiteBuilder};

use cosmwasm_std::{coin, to_binary, Addr, Binary, Coin, Decimal, Empty, StdResult, Uint128};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw_multi_test::{App, AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use cw20_base::msg::{ExecuteMsg as Cw20BaseExecuteMsg, InstantiateMsg as Cw20BaseInstantiateMsg};
use lending_utils::{
    credit_line::{CreditLineResponse, CreditLineValues},
    interest::Interest,
    token::Token,
};
use wyndex_oracle::msg::{
    ExecuteMsg as OracleExecuteMsg, InstantiateMsg as OracleInstantiateMsg,
    QueryMsg as OracleQueryMsg,
};

use super::ca_mock::{
    self, contract as contract_credit_agency, ExecuteMsg as CAExecuteMsg,
    InstantiateMsg as CAInstantiateMsg,
};
use crate::msg::{
    ApyResponse, ExecuteMsg, InstantiateMsg, InterestResponse, MigrateMsg, QueryMsg, ReceiveMsg,
    ReserveResponse, TokensBalanceResponse, TotalDebtResponse, TransferableAmountResponse,
};
use crate::state::Config;

// Tokens
pub const COMMON: &str = "common";
pub const MARKET_TOKEN: &str = "market";
pub const DISTRIBUTION_TOKEN: &str = "distribution";
pub const USDC: &str = "usdc";
pub const WYND: &str = "wynd";
// Addresses
pub const OWNER: &str = "owner";
pub const WHALE: &str = "whale";
pub const GOVERNANCE: &str = "governance";
pub const LENDER: &str = "lender";
pub const BORROWER: &str = "borrower";
pub const USER: &str = "user";

pub const DAY: u64 = 24 * 3600;

// TWAP parameters
pub const SAMPLE_PERIOD: SamplePeriod = SamplePeriod::HalfHour;
pub const TWAP_PERIOD: u64 = HALF_HOUR;

fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );

    Box::new(contract)
}

fn contract_oracle() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        wyndex_oracle::contract::execute,
        wyndex_oracle::contract::instantiate,
        wyndex_oracle::contract::query,
    );

    Box::new(contract)
}

pub fn contract_market() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply)
    .with_migrate(crate::contract::migrate);

    Box::new(contract)
}

pub fn contract_token() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        lend_token::contract::execute,
        lend_token::contract::instantiate,
        lend_token::contract::query,
    );

    Box::new(contract)
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
    /// Token used as the base asset for the market.
    market_token: Token,
    /// Token used as the distribution asset for the market.
    distribution_token: Token,
    /// An optional cap on total number of tokens deposited into the market
    cap: Option<Uint128>,
    /// Initial funds to provide for testing
    funds: Vec<(Addr, Vec<Coin>)>,
    /// Initial funds stored on contract
    contract_funds: Option<Coin>,
    /// Initial interest rate
    interest_base: Decimal,
    /// Initial interest slope
    interest_slope: Decimal,
    /// Interest charge period (in seconds)
    interest_charge_period: u64,
    /// Common Token that comes from Credit Agency (same for all markets)
    common_token: Token,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    collateral_ratio: Decimal,
    /// Defines the portion of borrower interest that is converted into reserves (0 <= x <= 1)
    reserve_factor: Decimal,
    /// Defines the how much of the credit limit can be borrowed (0 <= x <= 1)
    borrow_limit_ratio: Decimal,
    /// Native token pools created during Suite building. Cw20 tokens pools have to be created later
    /// with token addresses.
    pools: HashMap<u64, (lending_utils::coin::Coin, lending_utils::coin::Coin)>,
    /// Initial cw20 tokens distribution. The key is the token name and the value represents balances.
    initial_cw20: HashMap<String, Vec<Cw20Coin>>,
    credit_agency_funds: Option<Coin>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            name: "lend".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            market_token: Token::Native(MARKET_TOKEN.to_owned()),
            distribution_token: Token::Native(DISTRIBUTION_TOKEN.to_owned()),
            cap: None,
            funds: vec![],
            contract_funds: None,
            interest_base: Decimal::percent(3),
            interest_slope: Decimal::percent(20),
            interest_charge_period: 300,
            common_token: Token::Native(COMMON.to_owned()),
            collateral_ratio: Decimal::percent(50),
            borrow_limit_ratio: Decimal::one(),
            reserve_factor: Decimal::percent(0),
            pools: HashMap::new(),
            initial_cw20: HashMap::new(),
            credit_agency_funds: None,
        }
    }

    /// Allows to change the default market token.
    pub fn with_market_token(mut self, token: Token) -> Self {
        self.market_token = token;
        self
    }

    pub fn with_charge_period(mut self, charge_period: u64) -> Self {
        self.interest_charge_period = charge_period;
        self
    }

    pub fn with_cap(mut self, cap: impl Into<Uint128>) -> Self {
        self.cap = Some(cap.into());
        self
    }

    /// Sets initial amount of distributable tokens on address
    pub fn with_funds(mut self, addr: &str, funds: &[lending_utils::coin::Coin]) -> Self {
        let native_funds = funds
            .iter()
            .map(|c| Coin::try_from(c.clone()).unwrap())
            .collect();
        self.funds.push((Addr::unchecked(addr), native_funds));
        self
    }

    /// Sets initial balance of CW20 tokens,
    pub fn with_initial_cw20(mut self, denom: String, (address, amount): (&str, u64)) -> Self {
        let initial_balance = Cw20Coin {
            address: address.to_owned(),
            amount: Uint128::from(amount),
        };

        self.initial_cw20
            .entry(denom)
            .and_modify(|l| l.push(initial_balance.clone()))
            .or_insert_with(|| vec![initial_balance]);
        self
    }

    /// Sets initial amount of distributable tokens on market address
    pub fn with_contract_funds(mut self, funds: lending_utils::coin::Coin) -> Self {
        let funds = Coin::try_from(funds).unwrap();
        self.contract_funds = Some(funds);
        self
    }

    /// Sets initial amount of distributable tokens on credit agency address
    pub fn with_agency_funds(mut self, funds: lending_utils::coin::Coin) -> Self {
        let funds = Coin::try_from(funds).unwrap();
        self.credit_agency_funds = Some(funds);
        self
    }

    /// Sets initial interest base and slope (in percentage)
    pub fn with_interest(mut self, base: u64, slope: u64) -> Self {
        self.interest_base = Decimal::percent(base);
        self.interest_slope = Decimal::percent(slope);
        self
    }

    /// Sets initial collateral ratio
    pub fn with_collateral_ratio(mut self, collateral_ratio: Decimal) -> Self {
        self.collateral_ratio = collateral_ratio;
        self
    }

    pub fn with_borrow_limit_ratio(mut self, borrow_limit_ratio: Decimal) -> Self {
        self.borrow_limit_ratio = borrow_limit_ratio;
        self
    }

    pub fn with_reserve_factor(mut self, reserve_factor: u64) -> Self {
        self.reserve_factor = Decimal::percent(reserve_factor);
        self
    }

    /// Sets initial pools. Only pools with `Token::Native` can be created this way. Cw20 tokens
    /// requrie their address which is now only after suite built.
    pub fn with_pool(
        mut self,
        id: u64,
        pool: (lending_utils::coin::Coin, lending_utils::coin::Coin),
    ) -> Self {
        if pool.0.denom.is_cw20() || pool.1.denom.is_cw20() {
            return self;
        }
        self.pools.insert(id, pool);
        self
    }

    /// Helper to create a pair between native assets and register the pool's address into the oracle.
    pub fn create_pair_and_register_address(
        app: &mut App,
        wyndex_suite: &mut WyndexSuite,
        owner: Addr,
        oracle_contract: Addr,
        tokens: &[AssetInfo; 2],
        pair_type: PairType,
    ) {
        let address = wyndex_suite.create_pair(app, &tokens.clone(), pair_type, None);
        app.execute_contract(
            owner,
            oracle_contract.clone(),
            &OracleExecuteMsg::RegisterPool {
                pair_contract: address.clone().into(),
                denom1: tokens[0].clone(),
                denom2: tokens[1].clone(),
            },
            &[],
        )
        .unwrap();

        // Check that the address has been added correctly.
        let query: Addr = app
            .wrap()
            .query_wasm_smart(
                oracle_contract,
                &OracleQueryMsg::PoolAddress {
                    first_asset: tokens[0].clone(),
                    second_asset: tokens[1].clone(),
                },
            )
            .unwrap();

        assert_eq!(address, query);
    }

    #[track_caller]
    pub fn build(mut self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked(OWNER.to_owned());

        // Initialize wyndex test dependencies.
        let wyndex_builder = WyndexSuiteBuilder {
            owner: owner.clone(),
        };
        let mut wyndex_suite = wyndex_builder.init_wyndex(&mut app);

        // Store cw20 code and instantiate initial tokens.
        let cw20_id = app.store_code(contract_cw20());
        let mut starting_cw20 = HashMap::new();
        for (denom, cw20_funds) in self.initial_cw20 {
            let token_addr = app
                .instantiate_contract(
                    cw20_id,
                    Addr::unchecked(owner.clone()),
                    &Cw20BaseInstantiateMsg {
                        name: denom.to_owned(),
                        symbol: denom.to_owned(),
                        decimals: 6,
                        initial_balances: cw20_funds.clone(),
                        // Minter has possibility to mint any amount of tokens.
                        mint: Some(MinterResponse {
                            minter: owner.to_string(),
                            cap: None,
                        }),
                        marketing: None,
                    },
                    &[],
                    denom.to_owned(),
                    None,
                )
                .unwrap();
            // If market token is cw20 we have to change the denom with correct address.
            if self.market_token == Token::Cw20(denom.clone()) {
                self.market_token = Token::Cw20(token_addr.to_string());
            }
            starting_cw20.insert(denom, Token::Cw20(token_addr.to_string()));
        }

        // Initialize the oracle with the multi-hop contract info. This is required to query prices.
        let oracle_id = app.store_code(contract_oracle());
        let oracle_contract = app
            .instantiate_contract(
                oracle_id,
                owner.clone(),
                &OracleInstantiateMsg {
                    controller: owner.to_string(),
                    multi_hop: wyndex_suite.multi_hop.address.to_string(),
                    start_age: 1,
                    sample_period: SAMPLE_PERIOD,
                },
                &[],
                "oracle",
                Some(owner.to_string()),
            )
            .unwrap();

        // Instantiate credit agency contract.
        let ca_id = app.store_code(contract_credit_agency());
        let ca_contract = app
            .instantiate_contract(
                ca_id,
                owner.clone(),
                &CAInstantiateMsg {},
                &[],
                "credit-agency",
                Some(owner.to_string()),
            )
            .unwrap();

        // Store lend-token contract.
        let token_id = app.store_code(contract_token());

        // Instantiate lend market contract.
        let contract_id = app.store_code(contract_market());
        let contract = app
            .instantiate_contract(
                contract_id,
                // set credit agency mock as owner of market
                ca_contract.clone(),
                &InstantiateMsg {
                    name: self.name,
                    symbol: self.symbol,
                    decimals: self.decimals,
                    token_id,
                    market_token: self.market_token.clone(),
                    market_cap: self.cap,
                    interest_rate: Interest::Linear {
                        base: self.interest_base,
                        slope: self.interest_slope,
                    },
                    distributed_token: self.distribution_token,
                    interest_charge_period: self.interest_charge_period,
                    common_token: self.common_token.clone(),
                    collateral_ratio: self.collateral_ratio,
                    price_oracle: oracle_contract.to_string(),
                    reserve_factor: self.reserve_factor,
                    borrow_limit_ratio: self.borrow_limit_ratio,
                    gov_contract: GOVERNANCE.to_string(),
                },
                &[],
                "market",
                Some(owner.to_string()),
            )
            .unwrap();

        // During build we can only create pairs between native tokens since for cw20 we need the
        // contract address.
        for (_, (coin1, coin2)) in self.pools.clone() {
            Self::create_pair_and_register_address(
                &mut app,
                &mut wyndex_suite,
                owner.clone(),
                oracle_contract.clone(),
                &[coin1.denom.into(), coin2.denom.into()],
                PairType::Xyk {},
            );
        }

        // Distribute initial native tokens to users, lend market contract and credit agency.
        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (addr, coin) in self.funds.clone() {
                router.bank.init_balance(storage, &addr, coin)?;
            }
            for (addr, maybe_funds) in [
                (&contract, self.contract_funds.clone()),
                (&ca_contract, self.credit_agency_funds.clone()),
            ] {
                if let Some(funds) = maybe_funds {
                    router.bank.init_balance(storage, addr, vec![funds])?;
                }
            }

            Ok(())
        })
        .unwrap();

        // query for token contracts
        let config: Config = app
            .wrap()
            .query_wasm_smart(contract.clone(), &QueryMsg::Configuration {})
            .unwrap();

        Suite {
            app,
            wyndex: wyndex_suite,
            owner,
            contract,
            ctoken_contract: config.ctoken_contract,
            market_token: self.market_token,
            common_token: self.common_token,
            ca_contract,
            collateral_ratio: self.collateral_ratio,
            oracle_contract,
            starting_cw20,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Wyndex
    wyndex: WyndexSuite,
    owner: Addr,
    /// Address of Market contract
    pub contract: Addr,
    /// Address of CToken contract
    ctoken_contract: Addr,
    /// The market's token denom deposited and lended by the contract
    pub market_token: Token,
    /// Credit agency token's common denom (with other markets)
    common_token: Token,
    /// Credit Agency contract address
    ca_contract: Addr,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    collateral_ratio: Decimal,
    oracle_contract: Addr,
    /// Address of initial cw20 tokens
    pub starting_cw20: HashMap<String, Token>,
}

impl Suite {
    pub fn app(&mut self) -> &mut App {
        &mut self.app
    }

    /// Helper function to create an empty pool and register its address to the Oracle.
    pub fn set_pool(
        &mut self,
        pools: &[(u64, (lending_utils::coin::Coin, lending_utils::coin::Coin))],
        pair_type: PairType,
    ) -> AnyResult<()> {
        let owner = self.owner.clone();
        let oracle = self.oracle_contract.clone();
        let hub = self.wyndex.mock_hub.clone().address.to_string();

        for (_, (coin1, coin2)) in pools {
            let asset_1: Asset = coin1.clone().into();
            let asset_2: Asset = coin2.clone().into();

            let mut init_params: Option<StablePoolParams> = None;
            if let PairType::Lsd {} = pair_type {
                // Create a default configuration for `init_params`.
                init_params = Some(StablePoolParams {
                    amp: 100,
                    owner: Some(OWNER.to_owned()),
                    lsd: Some(LsdInfo {
                        asset: asset_2.info.clone(),
                        hub: hub.clone(),
                        target_rate_epoch: DAY,
                    }),
                });
            }

            // Create the pair contract.
            let address = self.wyndex.create_pair(
                &mut self.app,
                &[asset_1.info.clone(), asset_2.info.clone()],
                pair_type.clone(),
                init_params,
            );

            self.app
                .execute_contract(
                    owner.clone(),
                    oracle.clone(),
                    &OracleExecuteMsg::RegisterPool {
                        pair_contract: address.clone().into(),
                        denom1: asset_1.info,
                        denom2: asset_2.info,
                    },
                    &[],
                )
                .unwrap();
        }
        Ok(())
    }

    /// Helper function to create an LSD pair, mint required tokens, add liquidity to the pool, and
    /// register its address to the oracle. This function assumes the second token is the LSD token,
    /// and also that the WHALE is the address that creates all pools.
    /// Returns a vector containing addresses of the created pools.
    pub fn create_pool_and_provide_liquidity(
        &mut self,
        (coin1, coin2): (lending_utils::coin::Coin, lending_utils::coin::Coin),
        pair_type: PairType,
    ) -> AnyResult<Addr> {
        let owner = self.owner.clone();
        let oracle = self.oracle_contract.clone();
        let hub = self.wyndex.mock_hub.clone().address.to_string();

        let asset_1: Asset = coin1.into();
        let asset_2: Asset = coin2.into();

        let mut init_params: Option<StablePoolParams> = None;
        if let PairType::Lsd {} = pair_type {
            // Create a default configuration for `init_params`.
            init_params = Some(StablePoolParams {
                amp: 100,
                owner: Some(OWNER.to_owned()),
                lsd: Some(LsdInfo {
                    asset: asset_2.info.clone(),
                    hub,
                    target_rate_epoch: DAY,
                }),
            });
        }

        // Create the pair contract.
        let address = self.wyndex.create_pair(
            &mut self.app,
            &[asset_1.info.clone(), asset_2.info.clone()],
            pair_type.clone(),
            init_params,
        );

        // Just to be 100% sure the pair type is correct.
        let resp: PairInfo = self
            .app
            .wrap()
            .query_wasm_smart(address.clone(), &PairQueryMsg::Pair {})
            .unwrap();
        assert_eq!(resp.pair_type, pair_type);

        let native_tokens =
            self.mint_tokens_and_allowance(asset_1.clone(), asset_2.clone(), &address);

        self.wyndex
            .provide_liquidity(
                &mut self.app,
                WHALE,
                &address,
                &[
                    Asset {
                        info: asset_1.info.clone(),
                        amount: asset_1.amount,
                    },
                    Asset {
                        info: asset_2.info.clone(),
                        amount: asset_2.amount,
                    },
                ],
                &native_tokens, // for native token you need to transfer tokens manually
            )
            .unwrap();

        // Register the pool in the oracle.
        self.app
            .execute_contract(
                owner,
                oracle,
                &OracleExecuteMsg::RegisterPool {
                    pair_contract: address.clone().into(),
                    denom1: asset_1.info,
                    denom2: asset_2.info,
                },
                &[],
            )
            .unwrap();

        Ok(address)
    }

    /// Helper function to mint required tokens, and add them to the liquidity pool. The function
    /// assumes the WHALE is the address that creates all pools.
    /// Returns a vector containing addresses of the created pools.
    pub fn provide_liquidity(
        &mut self,
        pair: &Addr,
        (coin1, coin2): (lending_utils::coin::Coin, lending_utils::coin::Coin),
    ) -> AnyResult<()> {
        let asset_1: Asset = coin1.into();
        let asset_2: Asset = coin2.into();

        let native_tokens = self.mint_tokens_and_allowance(asset_1.clone(), asset_2.clone(), pair);

        self.wyndex
            .provide_liquidity(
                &mut self.app,
                WHALE,
                pair,
                &[
                    Asset {
                        info: asset_1.info.clone(),
                        amount: asset_1.amount,
                    },
                    Asset {
                        info: asset_2.info.clone(),
                        amount: asset_2.amount,
                    },
                ],
                &native_tokens, // for native token you need to transfer tokens manually
            )
            .unwrap();

        Ok(())
    }

    /// Helper function to mint cw20 or native tokens to `WHALE` address. For minted cw20, give allowance
    /// to pair address. For minted native, return minted tokens in `native_tokens`.
    pub fn mint_tokens_and_allowance(
        &mut self,
        asset_1: Asset,
        asset_2: Asset,
        pair: &Addr,
    ) -> Vec<Coin> {
        let mut native_tokens: Vec<Coin> = vec![];

        match asset_1.info.clone() {
            AssetInfo::Token(addr) => {
                // Mint some initial balances for whale user.
                self.mint_cw20(&Addr::unchecked(&addr), asset_1.amount.u128(), WHALE)
                    .unwrap();

                // Increases allowances for given LP contracts in order to provide liquidity to pool.
                self.increase_allowance(
                    WHALE,
                    &Addr::unchecked(addr),
                    pair.as_str(),
                    asset_1.amount.u128(),
                )
                .unwrap();
            }
            AssetInfo::Native(denom) => {
                self.app
                    .sudo(SudoMsg::Bank(BankSudo::Mint {
                        to_address: WHALE.to_string(),
                        amount: vec![coin(asset_1.amount.u128(), denom.clone())],
                    }))
                    .unwrap();

                native_tokens.push(coin(asset_1.amount.u128(), denom));
            }
        };
        match asset_2.info.clone() {
            AssetInfo::Token(addr) => {
                // Mint some initial balances for whale user.
                self.mint_cw20(&Addr::unchecked(&addr), asset_2.amount.u128(), WHALE)
                    .unwrap();

                // Increases allowances for given LP contracts in order to provide liquidity to pool.
                self.increase_allowance(
                    WHALE,
                    &Addr::unchecked(addr),
                    pair.as_str(),
                    asset_2.amount.u128(),
                )
                .unwrap();
            }
            AssetInfo::Native(denom) => {
                self.app
                    .sudo(SudoMsg::Bank(BankSudo::Mint {
                        to_address: WHALE.to_string(),
                        amount: vec![coin(asset_2.amount.u128(), denom.clone())],
                    }))
                    .unwrap();

                native_tokens.push(coin(asset_2.amount.u128(), denom));
            }
        };

        native_tokens
    }

    pub fn increase_allowance(
        &mut self,
        sender: &str,
        contract: &Addr,
        spender: &str,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            contract.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.to_owned(),
                amount: amount.into(),
                expires: None,
            },
            &[],
        )
    }

    /// Helper function to:
    /// * Create a pool given by `coin1.denom` and `coin2.denom`.
    /// * Add to pool `coin1.amount` and `coin2.amount`
    /// * Advance time by twap period.
    /// * Add again to pool `coin1.amount` and `coin2.amount` to have a twap.
    /// * Advance time by twap period.
    /// Returns the address of the created pair.
    pub fn create_pool_with_liquidity_and_twap_price(
        &mut self,
        coin1: lending_utils::coin::Coin,
        coin2: lending_utils::coin::Coin,
        pair_type: PairType,
    ) -> Addr {
        let pair = self
            .create_pool_and_provide_liquidity((coin1.clone(), coin2.clone()), pair_type)
            .unwrap();

        self.advance_seconds(TWAP_PERIOD);

        self.provide_liquidity(&pair, (coin1, coin2)).unwrap();

        self.advance_seconds(TWAP_PERIOD);

        pair
    }

    pub fn credit_agency(&self) -> String {
        self.ca_contract.to_string()
    }

    pub fn advance_seconds(&mut self, seconds: u64) {
        self.app.update_block(|block| {
            block.time = block.time.plus_seconds(seconds);
            block.height += std::cmp::max(1, seconds / 5); // block time
        });
    }

    /// Gives ctoken contract address back
    pub fn ctoken(&self) -> Addr {
        self.ctoken_contract.clone()
    }

    /// The denom of the common token
    pub fn common_token(&self) -> Token {
        self.common_token.clone()
    }

    pub fn deposit_multiple_native(
        &mut self,
        sender: &str,
        funds: &[(Token, u128)],
    ) -> AnyResult<AppResponse> {
        let funds: Vec<_> = funds
            .iter()
            .map(|(denom, amount)| Coin::new(amount.to_owned(), denom.denom()))
            .collect();

        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Deposit {},
            &funds,
        )
    }

    /// Deposit base asset in the lending pool and mint c-token
    pub fn deposit(&mut self, sender: &str, token: Token, amount: u128) -> AnyResult<AppResponse> {
        use Token::*;
        match token {
            Native(denom) => {
                let deposit_coin = Coin::new(amount, denom);
                self.app.execute_contract(
                    Addr::unchecked(sender),
                    self.contract.clone(),
                    &ExecuteMsg::Deposit {},
                    &[deposit_coin],
                )
            }
            Cw20(address) => self.execute_deposit_through_cw20(sender, address, amount),
        }
    }

    pub fn execute_deposit_through_cw20(
        &mut self,
        sender: &str,
        address: String,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        let msg: Binary = to_binary(&ReceiveMsg::Deposit)?;

        self.app.execute_contract(
            Addr::unchecked(sender),
            Addr::unchecked(address),
            &Cw20BaseExecuteMsg::Send {
                contract: self.contract.to_string(),
                amount: Uint128::from(amount),
                msg,
            },
            &[],
        )
    }

    pub fn mint_cw20(
        &mut self,
        token: &Addr,
        amount: u128,
        recipient: &str,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(OWNER),
            token.clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: recipient.to_owned(),
                amount: amount.into(),
            },
            &[],
        )
    }

    /// Helper function to mint `amount` of `cw20_address` tokens to the market contract.
    pub fn mint_cw20_to_market(
        &mut self,
        cw20_address: String,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(OWNER.to_owned()),
            Addr::unchecked(cw20_address),
            &Cw20BaseExecuteMsg::Mint {
                recipient: self.contract.to_string(),
                amount: Uint128::from(amount),
            },
            &[],
        )
    }

    /// Helper function to mint `amount` of `cw20_address` tokens to the market contract.
    pub fn mint_cw20_to_agency(
        &mut self,
        cw20_address: String,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        let agency = self.credit_agency();
        self.app.execute_contract(
            Addr::unchecked(OWNER.to_owned()),
            Addr::unchecked(cw20_address),
            &Cw20BaseExecuteMsg::Mint {
                recipient: agency,
                amount: Uint128::from(amount),
            },
            &[],
        )
    }

    /// Withdraw base asset from the lending pool and burn c-token
    pub fn withdraw(&mut self, sender: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Withdraw {
                amount: amount.into(),
            },
            &[],
        )
    }

    /// Attempts to withdraw the full "withdrawable" amount (as determined by the withdrawable query),
    /// then performs a couple checks to make sure nothing more than that could be withdrawn.
    pub fn attempt_withdraw_max(&mut self, sender: &str) -> AnyResult<()> {
        let withdrawable = self.query_withdrawable(sender)?;
        let withdrawable_in_common =
            withdrawable.amount * self.query_price_market_per_common()?.rate_sell_per_buy;
        self.withdraw(sender, withdrawable.amount.u128())?;

        // mock the change in credit line
        let mut crl = self
            .query_total_credit_line(sender)?
            .validate(&self.common_token())?;
        crl.collateral = crl.collateral.saturating_sub(withdrawable_in_common);
        crl.credit_line = crl
            .credit_line
            .saturating_sub(withdrawable_in_common * self.collateral_ratio);
        crl.borrow_limit = crl
            .borrow_limit
            .saturating_sub(withdrawable_in_common * self.collateral_ratio);
        self.set_credit_line(sender, crl)?;

        // double check we cannot withdraw anything above this amount
        self.assert_withdrawable(sender, 0u128);
        assert!(self.withdraw(sender, 1).is_err());

        Ok(())
    }

    /// Borrow base asset from the lending pool and get debt
    pub fn borrow(&mut self, sender: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Borrow {
                amount: amount.into(),
            },
            &[],
        )
    }

    /// Attempts to borrow the full "borrowable" amount (as determined by the borrowable query),
    /// then performs a couple checks to make sure nothing more than that could be borrowed.
    pub fn attempt_borrow_max(&mut self, sender: &str) -> AnyResult<()> {
        let borrowable = self.query_borrowable(sender)?;
        let borrowable_in_common =
            borrowable.amount * self.query_price_market_per_common()?.rate_sell_per_buy;
        self.borrow(sender, borrowable.amount.u128())?;

        // mock the change in credit line
        let mut crl = self
            .query_total_credit_line(sender)?
            .validate(&self.common_token())?;
        crl.debt += borrowable_in_common;
        self.set_credit_line(sender, crl)?;

        // double check we cannot borrow anything above this amount
        self.assert_borrowable(sender, 0u128);
        assert!(self.borrow(sender, 1).is_err());

        Ok(())
    }

    /// Repay borrowed tokens from the lending pool and remove debt
    pub fn repay(
        &mut self,
        sender: &str,
        funds: lending_utils::coin::Coin,
    ) -> AnyResult<AppResponse> {
        use Token::*;
        match funds.denom {
            Native(denom) => self.app.execute_contract(
                Addr::unchecked(sender),
                self.contract.clone(),
                &ExecuteMsg::Repay {},
                &[Coin::new(funds.amount.u128(), denom)],
            ),
            Cw20(address) => self.execute_repay_through_cw20(sender, address, funds.amount.u128()),
        }
    }

    pub fn repay_to(
        &mut self,
        sender: &str,
        account: &str,
        funds: lending_utils::coin::Coin,
    ) -> AnyResult<AppResponse> {
        use Token::*;
        match funds.denom {
            Native(denom) => self.app.execute_contract(
                Addr::unchecked(sender),
                self.contract.clone(),
                &ExecuteMsg::RepayTo {
                    account: account.to_owned(),
                },
                &[Coin::new(funds.amount.u128(), denom)],
            ),
            Cw20(address) => {
                self.execute_repay_to_through_cw20(sender, address, funds.amount.u128(), account)
            }
        }
    }

    pub fn execute_repay_through_cw20(
        &mut self,
        sender: &str,
        address: String,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        let msg: Binary = to_binary(&ReceiveMsg::Repay)?;

        self.app.execute_contract(
            Addr::unchecked(sender),
            Addr::unchecked(address),
            &Cw20BaseExecuteMsg::Send {
                contract: self.contract.to_string(),
                amount: Uint128::from(amount),
                msg,
            },
            &[],
        )
    }

    pub fn execute_repay_to_through_cw20(
        &mut self,
        sender: &str,
        address: String,
        amount: u128,
        account: &str,
    ) -> AnyResult<AppResponse> {
        let msg: Binary = to_binary(&ReceiveMsg::RepayTo {
            account: account.to_owned(),
        })?;

        self.app.execute_contract(
            Addr::unchecked(sender),
            Addr::unchecked(address),
            &Cw20BaseExecuteMsg::Send {
                contract: self.contract.to_string(),
                amount: Uint128::from(amount),
                msg,
            },
            &[],
        )
    }

    pub fn swap_withdraw_from(
        &mut self,
        sender: impl Into<String>,
        account: impl Into<String>,
        sell_limit: Uint128,
        buy: lending_utils::coin::Coin,
    ) -> AnyResult<AppResponse> {
        self.swap_withdraw_from_with_multiplier(
            sender,
            account,
            sell_limit,
            buy,
            Decimal::percent(101u64),
        )
    }

    pub fn swap_withdraw_from_with_multiplier(
        &mut self,
        sender: impl Into<String>,
        account: impl Into<String>,
        sell_limit: Uint128,
        buy: lending_utils::coin::Coin,
        estimate_multiplier: Decimal,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::SwapWithdrawFrom {
                account: account.into(),
                buy,
                sell_limit,
                estimate_multiplier,
            },
            &[],
        )
    }

    pub fn adjust_common_token(
        &mut self,
        sender: &str,
        new_token: Token,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::AdjustCommonToken { new_token },
            &[],
        )
    }

    /// Shortcut for querying base asset balance in the market contract
    pub fn query_asset_balance(&self, owner: &str, denom: String) -> StdResult<u128> {
        let amount = self.app.wrap().query_balance(owner, denom)?.amount;
        Ok(amount.into())
    }

    pub fn query_cw20_balance(&self, owner: &str, contract: String) -> StdResult<u128> {
        let balance: BalanceResponse = self.app.wrap().query_wasm_smart(
            contract,
            &Cw20QueryMsg::Balance {
                address: owner.to_owned(),
            },
        )?;

        Ok(balance.balance.into())
    }

    /// Shortcut for querying base asset balance in the market contract
    pub fn query_contract_asset_balance(&self) -> StdResult<u128> {
        use Token::*;
        match self.market_token.clone() {
            Native(denom) => return self.query_asset_balance(self.contract.as_str(), denom),
            Cw20(address) => return self.query_cw20_balance(self.contract.as_str(), address),
        }
    }

    pub fn query_transferable_amount(
        &self,
        token: impl ToString,
        account: impl ToString,
    ) -> AnyResult<TransferableAmountResponse> {
        let resp: TransferableAmountResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::TransferableAmount {
                token: token.to_string(),
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    fn query_token_balance(
        &self,
        contract_address: &Addr,
        address: impl ToString,
    ) -> AnyResult<Uint128> {
        let response: BalanceResponse = self.app.wrap().query_wasm_smart(
            contract_address,
            &lend_token::QueryMsg::Balance {
                address: address.to_string(),
            },
        )?;
        Ok(response.balance)
    }

    /// Queries ctoken contract for balance
    pub fn query_ctoken_balance(&self, account: impl ToString) -> AnyResult<Uint128> {
        self.query_token_balance(&self.ctoken_contract, account)
    }

    /// Queries market contract for account's amount of debt
    pub fn query_debt(&self, account: impl ToString) -> AnyResult<Uint128> {
        Ok(self.query_tokens_balance(account)?.debt.amount)
    }

    /// Queries current interest and utilisation rates
    pub fn query_interest(&self) -> AnyResult<InterestResponse> {
        let resp: InterestResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Interest {})?;
        Ok(resp)
    }

    /// Queries current interest and utilisation rates
    pub fn query_credit_line(&self, account: impl ToString) -> AnyResult<CreditLineResponse> {
        let resp: CreditLineResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::CreditLine {
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    /// Queries the total credit line from the mock CA
    pub fn query_total_credit_line(&self, account: impl ToString) -> AnyResult<CreditLineResponse> {
        let resp: CreditLineResponse = self.app.wrap().query_wasm_smart(
            self.credit_agency(),
            &ca_mock::QueryMsg::TotalCreditLine {
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    /// Queries the tokens balance of the account
    pub fn query_tokens_balance(&self, account: impl ToString) -> AnyResult<TokensBalanceResponse> {
        let resp: TokensBalanceResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::TokensBalance {
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    /// Queries market contract for total amount of debt
    pub fn query_total_debt(&self) -> AnyResult<TotalDebtResponse> {
        self.app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::TotalDebt {})
            .map_err(|err| anyhow!(err))
    }

    /// Queries ctoken contract for token info
    pub fn query_ctoken_info(&self) -> AnyResult<lend_token::msg::TokenInfoResponse> {
        let ctoken = self.ctoken_contract.clone();
        self.app
            .wrap()
            .query_wasm_smart(ctoken, &lend_token::msg::QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }

    /// Helper to convert the given amount of c-tokens to the equivalent amount of base tokens
    pub fn ctokens_to_base(&self, amount: Uint128) -> u128 {
        let token_info = self.query_ctoken_info().unwrap();

        token_to_base(amount, token_info.multiplier).u128()
    }

    /// Queries for APY
    pub fn query_apy(&self) -> AnyResult<ApyResponse> {
        self.app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Apy {})
            .map_err(|err| anyhow!(err))
    }

    /// Sets TotalCreditLine response for CA mock
    pub fn set_credit_line(
        &mut self,
        account: impl ToString,
        credit_line: CreditLineValues,
    ) -> AnyResult<AppResponse> {
        let common_token = self.common_token();
        self.app.execute_contract(
            Addr::unchecked(account.to_string()),
            self.ca_contract.clone(),
            &CAExecuteMsg::SetCreditLine {
                credit_line: credit_line.make_response(common_token),
            },
            &[],
        )
    }

    /// Sets TotalCreditLine with arbitrary high credit line and no debt
    pub fn set_high_credit_line(&mut self, account: impl ToString) -> AnyResult<AppResponse> {
        self.set_credit_line(
            account,
            CreditLineValues {
                collateral: Uint128::new(10_000_000_000_000_000_000),
                credit_line: Uint128::new(10_000_000_000_000_000_000),
                borrow_limit: Uint128::new(10_000_000_000_000_000_000),
                debt: Uint128::zero(),
            },
        )
    }

    /// Queries reserves
    pub fn query_reserve(&self) -> AnyResult<Uint128> {
        let response: ReserveResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Reserve {})?;
        Ok(response.reserve)
    }

    pub fn query_config(&self) -> AnyResult<Config> {
        let response: Config = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Configuration {})?;
        Ok(response)
    }

    pub fn query_withdrawable(
        &self,
        account: impl ToString,
    ) -> AnyResult<lending_utils::coin::Coin> {
        let response: lending_utils::coin::Coin = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Withdrawable {
                account: account.to_string(),
            },
        )?;
        Ok(response)
    }

    pub fn query_borrowable(&self, account: impl ToString) -> AnyResult<lending_utils::coin::Coin> {
        let response: lending_utils::coin::Coin = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Borrowable {
                account: account.to_string(),
            },
        )?;
        Ok(response)
    }

    /// Queries the tokens balance of the account
    pub fn query_price_market_per_common(&self) -> AnyResult<PriceRate> {
        let resp: PriceRate = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::PriceMarketLocalPerCommon {},
        )?;
        Ok(resp)
    }

    /// Migrates the contract, possibly changing some cfg values via MigrateMsg.
    pub fn migrate(&mut self, new_code_id: u64, msg: &MigrateMsg) -> AnyResult<AppResponse> {
        let owner = self.owner.clone();
        self.app
            .migrate_contract(owner, self.contract.clone(), msg, new_code_id)
    }

    /// Changes collateral ratio parmeter in config through sudo. Pass new ratio as percentage.
    pub fn sudo_adjust_collateral_ratio(&mut self, new_ratio: u64) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.execute_contract(
            Addr::unchecked(GOVERNANCE),
            contract,
            &ExecuteMsg::AdjustCollateralRatio {
                new_ratio: Decimal::percent(new_ratio),
            },
            &[],
        )
    }

    /// Changes reserve factor parmeter in config through sudo. Pass new ratio as percentage.
    pub fn sudo_adjust_reserve_factor(&mut self, new_factor: u64) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.execute_contract(
            Addr::unchecked(GOVERNANCE),
            contract,
            &ExecuteMsg::AdjustReserveFactor {
                new_factor: Decimal::percent(new_factor),
            },
            &[],
        )
    }

    pub fn sudo_adjust_price_oracle(&mut self, new_oracle: &str) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.execute_contract(
            Addr::unchecked(GOVERNANCE),
            contract,
            &ExecuteMsg::AdjustPriceOracle {
                new_oracle: new_oracle.to_owned(),
            },
            &[],
        )
    }

    pub fn sudo_adjust_market_cap(
        &mut self,
        new_cap: impl Into<Option<Uint128>>,
    ) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.execute_contract(
            Addr::unchecked(GOVERNANCE),
            contract,
            &ExecuteMsg::AdjustMarketCap {
                new_cap: new_cap.into(),
            },
            &[],
        )
    }

    pub fn sudo_adjust_interest_rates(
        &mut self,
        new_interest_rates: Interest,
    ) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.execute_contract(
            Addr::unchecked(GOVERNANCE),
            contract,
            &ExecuteMsg::AdjustInterestRates { new_interest_rates },
            &[],
        )
    }

    pub fn assert_ctoken_balance(&self, account: impl ToString, amount: impl Into<Uint128>) {
        let balance = self.query_tokens_balance(account).unwrap();
        assert_eq!(balance.collateral.amount, amount.into());
    }

    pub fn assert_debt_balance(&self, account: impl ToString, amount: impl Into<Uint128>) {
        let balance = self.query_tokens_balance(account).unwrap();
        assert_eq!(balance.debt.amount, amount.into());
    }

    pub fn assert_debt(&self, account: impl ToString, amount: impl Into<Uint128>) {
        let crl = self.query_credit_line(account).unwrap();
        assert_eq!(crl.debt.amount, amount.into());
    }

    pub fn assert_collateral(&self, account: impl ToString, amount: impl Into<Uint128>) {
        let crl = self.query_credit_line(account).unwrap();
        assert_eq!(crl.collateral.amount, amount.into());
    }

    pub fn assert_withdrawable(&self, account: impl ToString, amount: impl Into<Uint128>) {
        let withdrawable = self.query_withdrawable(account).unwrap();
        assert_eq!(withdrawable.amount, amount.into());
    }

    pub fn assert_borrowable(&self, account: impl ToString, amount: impl Into<Uint128>) {
        let borrowable = self.query_borrowable(account).unwrap();
        assert_eq!(borrowable.amount, amount.into());
    }
}
