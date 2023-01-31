use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Addr,
        BankMsg,
        Binary,
        Coin,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
        StdError,
        Uint128,
    },
    cosmwasm_schema::cw_serde,
    utils::storage::plus::{
        Item,
        ItemStorage,
        Map,
        MapStorage,
    },
    contract_interfaces::stkd::{
        HandleAnswer,
        QueryAnswer,
        Unbond,
    }
};

#[cw_serde]
pub struct Unbonding {
    amount: Uint128,

    // Time for maturity, 2 intervals away from when unbond started
    //  t  : Waiting for batch unbond
    //  t+1: Unbonding
    //  t+2: Claimable
    maturity: u32,
}

// Keep track of a user's balance
#[cw_serde]
#[derive(Default)]
pub struct Balance (pub Uint128);

impl MapStorage<'static, Addr> for Balance {
    const MAP: Map<'static, Addr, Self> = Map::new("balance-");
}

// Keep track of a user's unbondings
#[cw_serde]
#[derive(Default)]
pub struct Unbondings(pub Vec<Unbonding>);

impl MapStorage<'static, Addr> for Unbondings {
    const MAP: Map<'static, Addr, Self> = Map::new("unbondings-");
}

#[cw_serde]
pub struct ViewingKey(pub String);

impl MapStorage<'static, Addr> for ViewingKey {
    const MAP: Map<'static, Addr, Self> = Map::new("vk-");
}

#[cw_serde]
pub struct Price(pub Uint128);

impl ItemStorage for Price {
    const ITEM: Item<'static, Self> = Item::new("item-price");
}

// Global time tracker
#[cw_serde]
pub struct Time(pub u32);

impl ItemStorage for Time {
    const ITEM: Item<'static, Self> = Item::new("item-time");
}

#[cw_serde]
pub struct Config {
    name: String,
    symbol: String,
    admin: Addr,
    decimals: u8,
}

impl ItemStorage for Config {
    const ITEM: Item<'static, Self> = Item::new("item-config");
}

// INSTANTIATE

pub struct InstantiateMsg {
    name: String,
    symbol: String,
    decimals: u8,
    price: Uint128,
}

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        name: msg.name,
        symbol: msg.symbol,
        admin: info.sender,
        decimals: msg.decimals,
    };
    config.save(deps.storage)?;
    Price(msg.price).save(deps.storage)?;

    Ok(Response::new())
}

// EXECUTE

pub enum ExecuteMsg {
    Send {
        recipient: String,
        recipient_code_hash: Option<String>,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    Stake {},
    Unbond {
        redeem_amount: Uint128,
    },
    Claim {},
    SetViewingKey {
        key: String,
    },
    FastForward {
        steps: u32,
    },
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Send { recipient, amount, .. } => {
            let my_balance = Balance::load(deps.storage, info.sender.clone())
                .map_err(|_| StdError::generic_err("Insufficient funds"))?.0;
            let their_balance = Balance::load(deps.storage, Addr::unchecked(recipient.clone()))
                .unwrap_or_default().0;

            Balance(my_balance - amount).save(deps.storage, info.sender)?;
            Balance(their_balance + amount).save(deps.storage, Addr::unchecked(recipient))?;
            Ok(Response::default())
        }
        // TODO: fees
        ExecuteMsg::Stake {} => {
            let mut amount = Uint128::zero();
            for coin in info.funds {
                if coin.denom == "uscrt".to_string() {
                    amount += coin.amount;
                }
            }
            if amount.is_zero() {
                return Err(StdError::generic_err("No SCRT was sent for staking"));
            }

            let deriv_amount = amount.multiply_ratio(Uint128::from(1_000_000u32), Price::load(deps.storage)?.0);
            
            let balance = Balance::load(deps.storage, info.sender.clone())
                .unwrap_or_default().0;
            Balance(balance + deriv_amount).save(deps.storage, info.sender)?;

            Ok(Response::default()
                .set_data(to_binary(&HandleAnswer::Stake {
                   scrt_staked: amount,
                   tokens_returned: deriv_amount,
                })?)
            )
        },
        ExecuteMsg::Unbond { redeem_amount } => {
            let balance = Balance::load(deps.storage, info.sender.clone())?.0;
            if balance < redeem_amount {
                return Err(StdError::generic_err(format!(
                    "insufficient funds to burn: balance={}, required={}", balance, redeem_amount
                )));
            }
            
            let time = Time::load(deps.storage)?.0;
            let unbonding = Unbonding {
                amount: redeem_amount,
                maturity: time,
            };

            let mut unbondings = Unbondings::load(deps.storage, info.sender.clone())
                .unwrap_or_default().0;
            unbondings.push(unbonding);
            Unbondings(unbondings).save(deps.storage, info.sender.clone())?;

            Balance(balance - redeem_amount).save(deps.storage, info.sender)?;
            let scrt_amount = redeem_amount
                .multiply_ratio(Price::load(deps.storage)?.0, Uint128::from(1_000_000u32));
            Ok(Response::default()
                .set_data(to_binary(&HandleAnswer::Unbond {
                    tokens_redeemed: redeem_amount,
                    scrt_to_be_received: scrt_amount,
                    estimated_time_of_maturity: time as u64 + 2,
                })?)
            )

        },
        ExecuteMsg::Claim {} => {
            let mut claimable = Uint128::zero();
            let unbondings = Unbondings::load(deps.storage, info.sender.clone())?.0;
            let time = Time::load(deps.storage)?.0;
            let mut new_unbondings = vec![];
            for unbonding in unbondings {
                if unbonding.maturity <= time {
                    claimable += unbonding.amount;
                } else {
                    new_unbondings.push(unbonding);
                }
            }
            Unbondings(new_unbondings).save(deps.storage, info.sender.clone())?;
            
            Ok(Response::default()
               .set_data(to_binary(&HandleAnswer::Claim {
                    withdrawn: claimable,
                    fees: Uint128::zero(), // no fees
               })?)
               .add_message(BankMsg::Send {
                   to_address: info.sender.to_string(),
                   amount: vec![Coin {
                       amount: claimable,
                       denom: "uscrt".to_string(),
                   }]
               }))
        },
        ExecuteMsg::SetViewingKey { key } => {
            ViewingKey(key).save(deps.storage, info.sender)?;
            Ok(Response::default())
        },
        ExecuteMsg::FastForward { steps } => {
            let time = Time::load(deps.storage)?.0;
            Time(time + steps).save(deps.storage)?;
            Ok(Response::default())
        }
    }
}

// QUERY

pub enum QueryMsg {
    Balance {
        address: Addr,
        key: String,
    },
    StakingInfo {
        time: u64, 
    },
    Unbonding {
        address: Addr,
        key: String,
    },
}

pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address, key } => {
            if key != ViewingKey::load(deps.storage, address.clone())?.0 {
                return Err(StdError::generic_err("unauthorized"));
            }
            
            to_binary(&QueryAnswer::Balance {
                amount: Balance::load(deps.storage, address)?.0,
            })
        },
        QueryMsg::StakingInfo { .. } => {
            let time = Time::load(deps.storage)?.0;
            to_binary(&QueryAnswer::StakingInfo {
                validators: vec![],
                unbonding_time: 2u32,
                unbonding_batch_interval: 1u32,
                next_unbonding_batch_time: (time + 1) as u64,
                // Not supported by mock stkd
                unbond_amount_of_next_batch: Uint128::zero(),
                batch_unbond_in_progress: false, 
                bonded_scrt: Uint128::zero(),
                reserved_scrt: Uint128::zero(),
                available_scrt: Uint128::zero(),
                rewards: Uint128::zero(),
                total_derivative_token_supply: Uint128::zero(),
                price: Price::load(deps.storage)?.0,
            })
        },
        QueryMsg::Unbonding { address, key } => {
            if key != ViewingKey::load(deps.storage, address.clone())?.0 {
                return Err(StdError::generic_err("unauthorized"));
            }
            
            let mut count: u64 = 0;
            let mut unbonds = vec![];
            let mut amount_in_next_batch = Uint128::zero();
            let time = Time::load(deps.storage)?.0;
            let unbondings = Unbondings::load(deps.storage, address)?.0;
            for unbonding in unbondings {
                if unbonding.maturity < time + 2 {
                    count += 1;
                    unbonds.push(Unbond {
                        amount: unbonding.amount,
                        unbonds_at: unbonding.maturity as u64,
                        is_mature: None,
                    });
                } else {
                    amount_in_next_batch += unbonding.amount;
                }
            }

            to_binary(&QueryAnswer::Unbonding { 
                count,
                claimable_scrt: None,
                unbondings: unbonds,
                unbond_amount_in_next_batch: amount_in_next_batch,
                estimated_time_of_maturity_for_next_batch: None,
            })
        },
    }
}

