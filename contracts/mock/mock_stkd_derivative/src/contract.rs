use shade_protocol::{
    Contract,
    c_std::{
        shd_entry_point,
        to_binary,
        Addr,
        BankMsg,
        Binary,
        Coin,
        Decimal,
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
    contract_interfaces::snip20::ReceiverHandleMsg,
    utils::{
        ExecuteCallback,
        InstantiateCallback,
        storage::plus::{
            Item,
            ItemStorage,
            Map,
            MapStorage,
        }
    },
};

pub use shade_protocol::contract_interfaces::stkd::{
    HandleAnswer as ExecuteAnswer,
    HandleMsg as ExecuteMsg,
    QueryAnswer,
    QueryMsg,
    Unbond,
};

#[cw_serde]
struct Unbonding {
    amount: Uint128,
    // Time for maturity, when bonding is claimable
    maturity: u32,
}

// Keep track of a user's balance
#[cw_serde]
#[derive(Default)]
struct Balance (pub Uint128);

impl MapStorage<'static, Addr> for Balance {
    const MAP: Map<'static, Addr, Self> = Map::new("balance-");
}

// Keep track of a user's unbondings
#[cw_serde]
#[derive(Default)]
struct Unbondings(pub Vec<Unbonding>);

impl MapStorage<'static, Addr> for Unbondings {
    const MAP: Map<'static, Addr, Self> = Map::new("unbondings-");
}

#[cw_serde]
struct ViewingKey(pub String);

impl MapStorage<'static, Addr> for ViewingKey {
    const MAP: Map<'static, Addr, Self> = Map::new("vk-");
}

#[cw_serde]
struct Price(pub Uint128);

impl ItemStorage for Price {
    const ITEM: Item<'static, Self> = Item::new("item-price");
}

// Global time tracker
#[cw_serde]
struct Time(pub u32);

impl ItemStorage for Time {
    const ITEM: Item<'static, Self> = Item::new("item-time");
}

#[cw_serde]
pub struct Config {
    name: String,
    symbol: String,
    decimals: u8,
    admin: Addr,
    unbonding_time: u32,
    unbonding_batch_interval: u32,
    staking_commission: Decimal,
    unbond_commission: Decimal,
}

impl ItemStorage for Config {
    const ITEM: Item<'static, Self> = Item::new("item-config");
}

// INSTANTIATE

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub price: Uint128,
    pub unbonding_time: u32,
    pub unbonding_batch_interval: u32,
    pub staking_commission: Decimal,
    pub unbond_commission: Decimal,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
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
        decimals: msg.decimals,
        admin: info.sender.clone(),
        unbonding_time: msg.unbonding_time,
        unbonding_batch_interval: msg.unbonding_batch_interval,
        staking_commission: msg.staking_commission,
        unbond_commission: msg.unbond_commission,
    };
    config.save(deps.storage)?;

    // Adjust price relative to uscrt for the off chance that msg.decimals isn't 6
    let mut price = msg.price;
    if msg.decimals != 6 {
        if msg.decimals > 6 {
            price = price / Uint128::new(10).pow(msg.decimals as u32 - 6);
        } else {
            price = price * Uint128::new(10).pow(6 - msg.decimals as u32);
        }
    }
    Price(price).save(deps.storage)?;

    Time(0).save(deps.storage)?;

    Ok(Response::new())
}

// EXECUTE

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Send { recipient, amount, recipient_code_hash, msg, .. } => {
            let my_balance = Balance::load(deps.storage, info.sender.clone())
                .map_err(|_| StdError::generic_err("Insufficient funds"))?.0;
            let their_balance = Balance::load(deps.storage, recipient.clone())
                .unwrap_or_default().0;

            Balance(my_balance.checked_sub(amount)
                    .map_err(|_| StdError::generic_err("Insufficient funds"))?)
                .save(deps.storage, info.sender.clone())?;
            Balance(their_balance + amount).save(deps.storage, recipient.clone())?;

            let mut messages = vec![];
            if let Some(receiver_hash) = recipient_code_hash {
                let recipient_addr = Addr::unchecked(recipient);
                messages.push(
                    ReceiverHandleMsg::new(
                        info.sender.to_string(), 
                        info.sender.to_string(), 
                        amount, 
                        None, 
                        msg
                    ).to_cosmos_msg(
                        &Contract {
                            address: recipient_addr,
                            code_hash: receiver_hash,
                        },
                        vec![],
                    )?
                );
            }
            Ok(Response::default()
               .add_messages(messages))
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

            let config = Config::load(deps.storage)?;
            let amount = amount - (amount * config.staking_commission);
            let deriv_amount = amount.multiply_ratio(Uint128::from(1_000_000u32), Price::load(deps.storage)?.0);
            
            let balance = Balance::load(deps.storage, info.sender.clone())
                .unwrap_or_default().0;
            Balance(balance + deriv_amount).save(deps.storage, info.sender)?;

            Ok(Response::default()
                .set_data(to_binary(&ExecuteAnswer::Stake {
                   scrt_staked: amount,
                   tokens_returned: deriv_amount,
                })?)
            )
        },
        ExecuteMsg::Unbond { redeem_amount } => {
            let balance = Balance::load(deps.storage, info.sender.clone())
                .unwrap_or_default().0;
            if balance < redeem_amount {
                return Err(StdError::generic_err(format!(
                    "insufficient funds to burn: balance={}, required={}", balance, redeem_amount
                )));
            }
            
            let config = Config::load(deps.storage)?;
            let time = Time::load(deps.storage)?.0;
            let maturity = time + config.unbonding_time 
                + config.unbonding_batch_interval - (time % config.unbonding_batch_interval);
            let unbond_amount = redeem_amount - (redeem_amount * config.unbond_commission);
            let unbonding = Unbonding {
                amount: unbond_amount,
                maturity,
            };

            let mut unbondings = Unbondings::load(deps.storage, info.sender.clone())
                .unwrap_or_default().0;
            unbondings.push(unbonding);
            Unbondings(unbondings).save(deps.storage, info.sender.clone())?;

            Balance(balance - redeem_amount).save(deps.storage, info.sender)?;
            let scrt_amount = redeem_amount
                .multiply_ratio(Price::load(deps.storage)?.0, Uint128::from(1_000_000u32));
            Ok(Response::default()
                .set_data(to_binary(&ExecuteAnswer::Unbond {
                    tokens_redeemed: redeem_amount,
                    scrt_to_be_received: scrt_amount,
                    estimated_time_of_maturity: maturity as u64,
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
            let returned = claimable.multiply_ratio(Price::load(deps.storage)?.0, Uint128::new(1_000_000));
            Unbondings(new_unbondings).save(deps.storage, info.sender.clone())?;
            
            Ok(Response::default()
               .set_data(to_binary(&ExecuteAnswer::Claim {
                    withdrawn: claimable,
                    fees: Uint128::zero(), // no fees
               })?)
               .add_message(BankMsg::Send {
                   to_address: info.sender.to_string(),
                   amount: vec![Coin {
                       amount: returned,
                       denom: "uscrt".to_string(),
                   }]
               }))
        },
        ExecuteMsg::SetViewingKey { key, .. } => {
            ViewingKey(key).save(deps.storage, info.sender)?;
            Ok(Response::default())
        },
        ExecuteMsg::MockFastForward { steps } => {
            let time = Time::load(deps.storage)?.0;
            Time(time + steps).save(deps.storage)?;
            Ok(Response::default())
        }
    }
}

// QUERY

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
                amount: Balance::load(deps.storage, address).unwrap_or_default().0,
            })
        },
        QueryMsg::StakingInfo { .. } => {
            let time = Time::load(deps.storage)?.0;
            let config = Config::load(deps.storage)?;
            let next_unbonding_batch_time = time + config.unbonding_batch_interval 
                - (time % config.unbonding_batch_interval);

            // Convert back to basis of 6 decimals
            let mut price = Price::load(deps.storage)?.0;
            if config.decimals != 6 {
                if config.decimals > 6 {
                    price = price * Uint128::new(10).pow(config.decimals as u32 - 6);
                } else {
                    price = price / Uint128::new(10).pow(6 - config.decimals as u32);
                }
            }

            to_binary(&QueryAnswer::StakingInfo {
                validators: vec![],
                unbonding_time: config.unbonding_time,
                unbonding_batch_interval: config.unbonding_batch_interval,
                next_unbonding_batch_time: next_unbonding_batch_time as u64,
                // Not supported by mock stkd
                unbond_amount_of_next_batch: Uint128::zero(),
                batch_unbond_in_progress: false, 
                bonded_scrt: Uint128::zero(),
                reserved_scrt: Uint128::zero(),
                available_scrt: Uint128::zero(),
                rewards: Uint128::zero(),
                total_derivative_token_supply: Uint128::zero(),
                price,
            })
        },
        QueryMsg::Unbonding { address, key, .. } => {
            if key != ViewingKey::load(deps.storage, address.clone())?.0 {
                return Err(StdError::generic_err("unauthorized"));
            }
            
            let mut count: u64 = 0;
            let mut unbonds = vec![];
            let mut amount_in_next_batch = Uint128::zero();
            let time = Time::load(deps.storage)?.0;
            let config = Config::load(deps.storage)?;
            let unbondings = Unbondings::load(deps.storage, address).unwrap_or_default().0;
            for unbonding in unbondings {
                if unbonding.maturity <= time + config.unbonding_time {
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

