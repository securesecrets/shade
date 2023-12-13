#[cfg(not(feature = "library"))]
use shade_protocol::c_std::shd_entry_point;
use shade_protocol::{
    c_std::{
        to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
        StdResult, Uint128,
    },
    contract_interfaces::snip20::Snip20ReceiveMsg,
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20,
    utils::Query,
};

use crate::{
    error::ContractError,
    msg::{
        AuthPermit, Authentication, BalanceResponse, ControllerQuery, ExecuteMsg, FundsResponse,
        InstantiateMsg, MultiplierResponse, QueryMsg, TokenInfoResponse, TransferableAmountResp,
    },
    state::{
        Distribution, TokenInfo, WithdrawAdjustment, Withdrawable, BALANCES, CONTROLLER,
        DISTRIBUTION, MULTIPLIER, POINTS_SCALE, QUERY_AUTH, TOKEN_INFO, TOTAL_SUPPLY, VIEWING_KEY,
        WITHDRAW_ADJUSTMENT,
    },
};
use lending_utils::amount::{base_to_token, token_to_base};

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
    };

    TOKEN_INFO.save(deps.storage, &token_info)?;

    let distribution = Distribution {
        denom: msg.distributed_token.into_valid(deps.api)?,
        points_per_token: Uint128::zero(),
        points_leftover: Uint128::zero(),
        distributed_total: Uint128::zero(),
        withdrawable_total: Uint128::zero(),
    };
    DISTRIBUTION.save(deps.storage, &distribution)?;

    QUERY_AUTH.save(deps.storage, &msg.query_auth)?;

    TOTAL_SUPPLY.save(deps.storage, &Uint128::zero())?;
    CONTROLLER.save(deps.storage, &msg.controller.into_valid(deps.api)?)?;
    MULTIPLIER.save(deps.storage, &Decimal::from_ratio(1u128, 100_000u128))?;

    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;

    Ok(Response::new())
}

/// Execution entry point
#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        Transfer { recipient, amount } => {
            let recipient = deps.api.addr_validate(&recipient)?;
            transfer(deps, env, info, recipient, amount)
        }
        TransferFrom {
            sender,
            recipient,
            amount,
        } => {
            let recipient = deps.api.addr_validate(&recipient)?;
            let sender = deps.api.addr_validate(&sender)?;
            transfer_from(deps, info, sender, recipient, amount)
        }
        TransferBaseFrom {
            sender,
            recipient,
            amount,
        } => {
            let controller = CONTROLLER.load(deps.storage)?;

            if info.sender != controller.address {
                return Err(ContractError::Unauthorized {});
            }

            let recipient = deps.api.addr_validate(&recipient)?;
            let sender = deps.api.addr_validate(&sender)?;
            let multiplier = MULTIPLIER.load(deps.storage)?;
            let amount = base_to_token(amount, multiplier);
            transfer_from(deps, info, sender, recipient, amount)
        }
        Send {
            contract,
            amount,
            msg,
        } => {
            let recipient = deps.api.addr_validate(&contract)?;
            send(deps, env, info, recipient, amount, msg)
        }
        Mint { recipient, amount } => mint(deps, info, recipient, amount),
        MintBase { recipient, amount } => mint_base(deps, info, recipient, amount),
        BurnFrom { owner, amount } => burn_from(deps, info, owner, amount),
        BurnBaseFrom { owner, amount } => burn_base_from(deps, info, owner, amount),
        Rebase { ratio } => rebase(deps, info, ratio),
        Distribute { sender } => distribute(deps, env, info, sender),
        WithdrawFunds {} => withdraw_funds(deps, env, info),
    }
}

/// Ensures, that tokens can be transferred from given account
fn can_transfer(
    deps: Deps,
    env: &Env,
    account: String,
    amount: Uint128,
) -> Result<(), ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    let transferable: TransferableAmountResp = ControllerQuery::TransferableAmount {
        token: env.contract.address.to_string(),
        account,
    }
    .query(&deps.querier, &controller)?;

    if amount <= transferable.transferable {
        Ok(())
    } else {
        Err(ContractError::CannotTransfer {
            max_transferable: transferable.transferable,
        })
    }
}

/// Performs tokens transfer.
fn transfer_tokens(
    mut deps: DepsMut,
    sender: &Addr,
    recipient: &Addr,
    amount: Uint128,
) -> Result<(), ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let distribution = DISTRIBUTION.load(deps.storage)?;
    let ppt = distribution.points_per_token.u128();
    BALANCES.update(
        deps.storage,
        sender,
        |balance: Option<Uint128>| -> Result<_, ContractError> {
            let balance = balance.unwrap_or_default();
            balance
                .checked_sub(amount)
                .map_err(|_| ContractError::insufficient_tokens(balance, amount))
        },
    )?;
    apply_points_correction(deps.branch(), sender, ppt, -(amount.u128() as i128))?;

    BALANCES.update(
        deps.storage,
        recipient,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;
    apply_points_correction(deps.branch(), recipient, ppt, amount.u128() as _)?;

    Ok(())
}

/// Handler for `ExecuteMsg::Transfer`
fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    can_transfer(deps.as_ref(), &env, info.sender.to_string(), amount)?;

    transfer_tokens(deps, &info.sender, &recipient, amount)?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);

    Ok(res)
}

/// Handler for `ExecuteMsg::TransferFrom`
fn transfer_from(
    deps: DepsMut,
    info: MessageInfo,
    sender: Addr,
    recipient: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    if info.sender != controller.address {
        return Err(ContractError::Unauthorized {});
    }

    transfer_tokens(deps, &sender, &recipient, amount)?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);

    Ok(res)
}

/// Handler for `ExecuteMsg::Send`
fn send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    can_transfer(deps.as_ref(), &env, info.sender.to_string(), amount)?;

    transfer_tokens(deps, &info.sender, &recipient, amount)?;

    let res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", &info.sender)
        .add_attribute("to", &recipient)
        .add_attribute("amount", amount)
        .add_message(
            Snip20ReceiveMsg {
                sender: info.sender.clone().into(),
                from: info.sender.into(),
                amount,
                memo: None,
                msg: Some(msg),
            }
            .into_cosmos_msg(env.contract.code_hash, recipient.to_string())?,
        );

    Ok(res)
}

pub fn mint_base(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let multiplier = MULTIPLIER.load(deps.storage)?;
    mint(deps, info, recipient, base_to_token(amount, multiplier))
}

/// Handler for `ExecuteMsg::Mint`
pub fn mint(
    mut deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;

    if info.sender != controller.address {
        return Err(ContractError::Unauthorized {});
    }

    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let ppt = DISTRIBUTION.load(deps.storage)?.points_per_token.u128();

    let recipient_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &recipient_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;
    apply_points_correction(deps.branch(), &recipient_addr, ppt, amount.u128() as _)?;

    TOTAL_SUPPLY.update(deps.storage, |supply| -> StdResult<_> {
        Ok(supply + amount)
    })?;

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

/// Handler for `ExecuteMsg::BurnBaseFrom`
pub fn burn_base_from(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // convert amount from base to token amount
    let multiplier = MULTIPLIER.load(deps.storage)?;
    let amount = base_to_token(amount, multiplier);

    burn_from(deps, info, owner, amount)
}

/// Handler for `ExecuteMsg::BurnFrom`
pub fn burn_from(
    mut deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    let owner = deps.api.addr_validate(&owner)?;

    if info.sender != controller.address {
        return Err(ContractError::Unauthorized {});
    }

    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let ppt = DISTRIBUTION.load(deps.storage)?.points_per_token;

    BALANCES.update(
        deps.storage,
        &owner,
        |balance: Option<Uint128>| -> Result<_, ContractError> {
            let balance = balance.unwrap_or_default();
            balance
                .checked_sub(amount)
                .map_err(|_| ContractError::insufficient_tokens(balance, amount))
        },
    )?;
    apply_points_correction(
        deps.branch(),
        &owner,
        ppt.u128() as _,
        -(amount.u128() as i128),
    )?;

    TOTAL_SUPPLY.update(deps.storage, |supply| -> Result<_, ContractError> {
        supply
            .checked_sub(amount)
            .map_err(|_| ContractError::insufficient_tokens(supply, amount))
    })?;

    let res = Response::new()
        .add_attribute("action", "burn_from")
        .add_attribute("from", owner)
        .add_attribute("by", info.sender)
        .add_attribute("amount", amount);
    Ok(res)
}

/// Handler for `ExecuteMsg::Rebase`
pub fn rebase(deps: DepsMut, info: MessageInfo, ratio: Decimal) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    if info.sender != controller.address {
        return Err(ContractError::Unauthorized {});
    }

    MULTIPLIER.update(deps.storage, |multiplier: Decimal| -> StdResult<_> {
        Ok(multiplier * ratio)
    })?;

    let res = Response::new()
        .add_attribute("action", "rebase")
        .add_attribute("ratio", ratio.to_string());

    Ok(res)
}

/// Handler for `ExecuteMsg::Distribute`
pub fn distribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let total_supply = TOTAL_SUPPLY.load(deps.storage)?.u128();

    if total_supply == 0 {
        return Err(ContractError::NoHoldersToDistributeTo {});
    }

    let sender = sender
        .map(|sender| deps.api.addr_validate(&sender))
        .transpose()?
        .unwrap_or(info.sender);

    let mut distribution = DISTRIBUTION.load(deps.storage)?;

    let withdrawable: u128 = distribution.withdrawable_total.into();

    let balance = snip20::helpers::balance_query(
        &deps.querier,
        env.contract.address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &distribution.denom.clone(),
    )?
    .u128();

    let amount = balance - withdrawable;
    if amount == 0 {
        return Ok(Response::new());
    }

    // Distribution calculation:
    // 1. Distributed amount is turned into points by scalling them by POINTS_SCALE;
    // 2. The leftover from any previous distribution is added to be distributed now;
    // 3. Calculating how much points would be distributed to receivers per token they own;
    // 4. It is very much possible, that non-whole points should be paid for single token. To
    //    overcome this, we distribute as much points as it is possible without non-whole division,
    //    and leftover is stored for next distribution.
    // 5. Distributed points per token are accumulated;
    let leftover: u128 = distribution.points_leftover.into();
    let points = amount * POINTS_SCALE + leftover;
    let points_per_token = points / total_supply;
    distribution.points_leftover = (points % total_supply).into();

    // Everything goes back to 128-bits/16-bytes
    // Full amount is added here to total withdrawable, as it should not be considered on its own
    // on future distributions - even if because of calculation offsets it is not fully
    // distributed, the error is handled by leftover.
    distribution.points_per_token += Uint128::from(points_per_token);
    distribution.distributed_total += Uint128::from(amount);
    distribution.withdrawable_total += Uint128::from(amount);

    DISTRIBUTION.save(deps.storage, &distribution)?;

    let resp = Response::new()
        .add_attribute("action", "distribute_tokens")
        .add_attribute("sender", sender.as_str())
        .add_attribute("amount", amount.to_string())
        .add_attribute("snip20_address", distribution.denom.address);

    Ok(resp)
}

/// Handler for `ExecuteMsg::WithdrawFunds`
fn withdraw_funds(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut distribution = DISTRIBUTION.load(deps.storage)?;
    let mut adjustment = WITHDRAW_ADJUSTMENT
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    let token = withdrawable_funds(deps.as_ref(), &info.sender, &distribution, &adjustment)?;
    if token.amount.is_zero() {
        // Just do nothing
        return Ok(Response::new());
    }

    adjustment.withdrawn_funds += token.amount;
    WITHDRAW_ADJUSTMENT.save(deps.storage, &info.sender, &adjustment)?;
    distribution.withdrawable_total -= token.amount;
    DISTRIBUTION.save(deps.storage, &distribution)?;

    let resp = Response::new()
        .add_attribute("action", "withdraw_tokens")
        .add_attribute("owner", info.sender.as_str())
        .add_attribute("amount", token.amount.to_string())
        .add_message(
            Snip20ReceiveMsg {
                sender: env.contract.address.clone().into(),
                from: env.contract.address.into(),
                amount: token.amount,
                memo: None,
                msg: None,
            }
            .into_cosmos_msg(env.contract.code_hash, info.sender.to_string())?,
        )
        .add_attribute("snip20_address", distribution.denom.address);

    Ok(resp)
}

/// Handler for `QueryMsg::BaseBalance`
/// Returns the amount of `address`'s tokens in terms of base token.
pub fn query_base_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    let multiplier = MULTIPLIER.load(deps.storage)?;
    let balance = token_to_base(balance, multiplier);
    Ok(BalanceResponse { balance })
}

/// Handler for `QueryMsg::Balance`
pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}

/// Handler for `QueryMsg::TokenInfo`
pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let token_info = TOKEN_INFO.load(deps.storage)?;
    let total_supply = TOTAL_SUPPLY.load(deps.storage)?;
    let multiplier = MULTIPLIER.load(deps.storage)?;

    Ok(TokenInfoResponse {
        name: token_info.name,
        symbol: token_info.symbol,
        decimals: token_info.decimals,
        total_supply,
        multiplier,
    })
}

/// Handler for `QueryMsg::Multiplier`
pub fn query_multiplier(deps: Deps) -> StdResult<MultiplierResponse> {
    let multiplier = MULTIPLIER.load(deps.storage)?;

    Ok(MultiplierResponse { multiplier })
}

/// Handler for `QueryMsg::DistributedFunds`
pub fn query_distributed_funds(deps: Deps) -> StdResult<FundsResponse> {
    let distribution = DISTRIBUTION.load(deps.storage)?;
    Ok(FundsResponse {
        token: distribution.denom,
        amount: distribution.distributed_total.into(),
    })
}

/// Handler for `QueryMsg::UndistributedFunds`
pub fn query_undistributed_funds(deps: Deps, env: Env) -> StdResult<FundsResponse> {
    let distribution = DISTRIBUTION.load(deps.storage)?;
    let balance = snip20::helpers::balance_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        &distribution.denom.clone(),
    )?;
    Ok(FundsResponse {
        token: distribution.denom,
        amount: balance - distribution.withdrawable_total,
    })
}

/// Handler for `QueryMsg::WithdrawableFunds`
pub fn query_withdrawable_funds(deps: Deps, owner: String) -> StdResult<FundsResponse> {
    let owner = Addr::unchecked(owner);
    let distribution = DISTRIBUTION.load(deps.storage)?;
    let adjustment = WITHDRAW_ADJUSTMENT
        .may_load(deps.storage, &owner)?
        .unwrap_or_default();

    let withdrawable_funds = withdrawable_funds(deps, &owner, &distribution, &adjustment)?;
    Ok(FundsResponse {
        token: withdrawable_funds.denom,
        amount: withdrawable_funds.amount.into(),
    })
}

pub fn authenticate(deps: Deps, auth: Authentication, account: Addr) -> StdResult<()> {
    match auth {
        Authentication::ViewingKey { key, address } => {
            let address = deps.api.addr_validate(&address)?;
            if !authenticate_vk(
                address.clone(),
                key,
                &deps.querier,
                &QUERY_AUTH.load(deps.storage)?,
            )? {
                return Err(StdError::generic_err("Invalid Viewing Key"));
            }
            if address != account {
                return Err(StdError::generic_err(
                    "Trying to query using viewing key not matching the account",
                ));
            }
            Ok(())
        }
        Authentication::Permit(permit) => {
            let res: PermitAuthentication<AuthPermit> =
                authenticate_permit(permit, &deps.querier, QUERY_AUTH.load(deps.storage)?)?;
            if res.revoked {
                return Err(StdError::generic_err("Permit Revoked"));
            }
            if res.sender != CONTROLLER.load(deps.storage)?.address {
                return Err(StdError::generic_err(
                    "Unauthorized: Only proper market contract can query using permit",
                ));
            }
            Ok(())
        }
    }
}

/// `QueryMsg` entry point
#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Balance {
            address,
            authentication,
        } => {
            authenticate(deps, authentication, deps.api.addr_validate(&address)?)?;
            to_binary(&query_balance(deps, address)?)
        }
        BaseBalance {
            address,
            authentication,
        } => {
            authenticate(deps, authentication, deps.api.addr_validate(&address)?)?;
            to_binary(&query_base_balance(deps, address)?)
        }
        TokenInfo {} => to_binary(&query_token_info(deps)?),
        Multiplier {} => to_binary(&query_multiplier(deps)?),
        DistributedFunds {} => to_binary(&query_distributed_funds(deps)?),
        UndistributedFunds {} => to_binary(&query_undistributed_funds(deps, env)?),
        WithdrawableFunds {
            owner,
            authentication,
        } => {
            authenticate(deps, authentication, deps.api.addr_validate(&owner)?)?;
            to_binary(&query_withdrawable_funds(deps, owner)?)
        }
    }
}

/// Calculates withdrawable funds from distribution and adjustment info.
pub fn withdrawable_funds(
    deps: Deps,
    owner: &Addr,
    distribution: &Distribution,
    adjustment: &WithdrawAdjustment,
) -> StdResult<Withdrawable> {
    let ppt: u128 = distribution.points_per_token.into();
    let tokens: u128 = BALANCES
        .may_load(deps.storage, owner)?
        .unwrap_or_default()
        .into();
    let correction: i128 = adjustment.points_correction.into();
    let withdrawn: u128 = adjustment.withdrawn_funds.into();
    let points = (ppt * tokens) as i128;
    let points = points + correction;
    let amount = points as u128 / POINTS_SCALE;
    let amount = amount - withdrawn;

    Ok(Withdrawable {
        amount: Uint128::new(amount),
        denom: distribution.denom.clone(),
    })
}

/// Applies points correction for given address.
/// `ppt` is current value from `POINTS_PER_TOKEN` - not loaded in function, to
/// avoid multiple queries on bulk updates.
/// `diff` is the weight change
pub fn apply_points_correction(deps: DepsMut, addr: &Addr, ppt: u128, diff: i128) -> StdResult<()> {
    WITHDRAW_ADJUSTMENT.update(deps.storage, addr, |old| -> StdResult<_> {
        let mut old = old.unwrap_or_default();
        let points_correction: i128 = old.points_correction.into();
        old.points_correction = (points_correction - ppt as i128 * diff).into();
        Ok(old)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use shade_protocol::{
        c_std::testing::{mock_dependencies, mock_env, mock_info},
        utils::asset::RawContract,
    };

    use super::*;

    #[test]
    fn rebase_works() {
        let mut deps = mock_dependencies();
        let controller = RawContract::new(&"controller".to_string(), &"".to_string());
        let distributed_token = RawContract::new(&"SHADE".to_string(), &"".to_string());
        let instantiate_msg = InstantiateMsg {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            controller,
            distributed_token,
            viewing_key: "VIEWIENG_KEY".to_string(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let basic_mul = Decimal::from_ratio(1u128, 100_000u128);
        // Multiplier is 1.0 at first
        assert_eq!(basic_mul, MULTIPLIER.load(&deps.storage).unwrap());

        // We rebase by 1.2, multiplier is 1.2
        let info = mock_info(&controller.address, &[]);
        rebase(deps.as_mut(), info.clone(), Decimal::percent(120)).unwrap();
        assert_eq!(
            basic_mul * Decimal::percent(120),
            MULTIPLIER.load(&deps.storage).unwrap()
        );

        // We rebase by 1.2, multiplier is 1.44
        rebase(deps.as_mut(), info, Decimal::percent(120)).unwrap();
        assert_eq!(
            basic_mul * Decimal::percent(144),
            MULTIPLIER.load(&deps.storage).unwrap()
        );
    }

    #[test]
    fn rebase_query_works() {
        let mut deps = mock_dependencies();
        let controller = RawContract::new(&"controller".to_string(), &"".to_string());
        let distributed_token = RawContract::new(&"SHADE".to_string(), &"".to_string());
        let instantiate_msg = InstantiateMsg {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            controller,
            distributed_token,
            viewing_key: "VIEWIENG_KEY".to_string(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let basic_mul = MULTIPLIER.load(&deps.storage).unwrap();

        let info = mock_info(&controller.address, &[]);
        rebase(deps.as_mut(), info, Decimal::percent(120)).unwrap();
        assert_eq!(
            basic_mul * Decimal::percent(120),
            MULTIPLIER.load(&deps.storage).unwrap()
        );

        let res = query_multiplier(deps.as_ref()).unwrap();
        assert_eq!(
            MultiplierResponse {
                multiplier: basic_mul * Decimal::percent(120)
            },
            res
        );
    }
}
