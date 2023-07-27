use crate::{
    batch,
    distributors,
    distributors::{
        get_distributor,
        try_add_distributors,
        try_set_distributors,
        try_set_distributors_status,
    },
    expose_balance::{try_expose_balance, try_expose_balance_with_cooldown},
    msg::{
        space_pad,
        status_level_to_u8,
        ContractStatusLevel,
        HandleAnswer,
        ExecuteMsg,
        InstantiateMsg,
        QueryAnswer,
        QueryMsg,
        QueryWithPermit,
        ResponseStatus::Success,
    },
    rand::sha_256,
    receiver::Snip20ReceiveMsg,
    stake::{
        claim_rewards,
        remove_from_cooldown,
        shares_per_token,
        try_claim_rewards,
        try_claim_unbond,
        try_receive,
        try_stake_rewards,
        try_unbond,
        try_update_stake_config,
    },
    stake_queries,
    state::{
        get_receiver_hash,
        read_allowance,
        read_viewing_key,
        set_receiver_hash,
        write_allowance,
        write_viewing_key,
        Balances,
        Config,
        Constants,
        ReadonlyBalances,
        ReadonlyConfig,
    },
    state_staking::{
        DailyUnbondingQueue,
        Distributors,
        DistributorsEnabled,
        TotalShares,
        TotalTokens,
        TotalUnbonding,
        UnsentStakedTokens,
        UserCooldown,
        UserShares,
    },
    transaction_history::{get_transfers, get_txs, store_claim_reward, store_mint, store_transfer},
    viewing_key::{ViewingKey, VIEWING_KEY_SIZE},
};
use shade_protocol::c_std::{Deps, MessageInfo, Uint128, Uint256};
/// This contract implements SNIP-20 standard:
/// https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md
use shade_protocol::c_std::{
    from_binary,
    to_binary,
    Api,
    Binary,
    CanonicalAddr,
    CosmosMsg,
    Env,
    DepsMut,
    Addr,
    Response,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::{
    // permit::{validate, Permission, Permit, RevokedPermits},
    contract_interfaces::snip20::helpers::{register_receive, send_msg, token_info},
};
use shade_protocol::{Contract, contract_interfaces::staking::snip20_staking::{
    stake::{Cooldown, StakeConfig, VecQueue},
    ReceiveType,
}, utils::storage::default::{BucketStorage, SingletonStorage}};
use shade_protocol::query_authentication::permit::Permit;
use shade_protocol::snip20::Permission;

/// We make sure that responses from `execute` are padded to a multiple of this size.
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // Check name, symbol, decimals
    if !is_valid_name(&msg.name) {
        return Err(StdError::generic_err(
            "Name is not in the expected format (3-30 UTF-8 bytes)",
        ));
    }
    if !is_valid_symbol(&msg.symbol) {
        return Err(StdError::generic_err(
            "Ticker symbol is not in expected format [A-Z]{3,6}",
        ));
    }

    let init_config = msg.config();
    let admin = msg.admin.unwrap_or(info.sender);

    let total_supply: u128 = 0;

    let prng_seed_hashed = sha_256(&msg.prng_seed.0);

    // Set stake config
    let staked_token_decimals: u8;
    if let Some(decimals) = msg.decimals {
        staked_token_decimals = decimals;
    } else {
        staked_token_decimals = token_info_query(
            &deps.querier,
            256,
            msg.staked_token.code_hash.clone(),
            msg.staked_token.address.clone(),
        )?
        .decimals;
    }

    let mut config = Config::from_storage(deps.storage);
    config.set_constants(&Constants {
        name: msg.name,
        symbol: "STKD-".to_string() + &msg.symbol,
        decimals: staked_token_decimals,
        admin,
        prng_seed: prng_seed_hashed.to_vec(),
        total_supply_is_public: init_config.public_total_supply(),
        contract_address: env.contract.address,
    })?;
    config.set_total_supply(total_supply);
    config.set_contract_status(ContractStatusLevel::NormalRun);

    // Set distributors
    Distributors(msg.distributors.unwrap_or_default()).save(deps.storage)?;
    DistributorsEnabled(msg.limit_transfer).save(deps.storage)?;

    if staked_token_decimals * 2 > msg.share_decimals {
        return Err(StdError::generic_err(
            "Share decimals must be two times greater than the token decimals",
        ));
    }

    StakeConfig {
        unbond_time: msg.unbond_time,
        staked_token: msg.staked_token.clone(),
        decimal_difference: msg.share_decimals - staked_token_decimals,
        treasury: msg.treasury.clone(),
    }
    .save(deps.storage)?;

    // Set shares state to 0
    TotalShares(Uint256::zero()).save(deps.storage)?;

    // Initialize unbonding queue
    DailyUnbondingQueue(VecQueue::new(vec![])).save(deps.storage)?;

    // Set tokens
    TotalTokens(Uint128::zero()).save(deps.storage)?;

    TotalUnbonding(Uint128::zero()).save(deps.storage)?;

    UnsentStakedTokens(Uint128::zero()).save(deps.storage)?;

    // Register receive if necessary
    let mut messages = vec![];
    if let Some(addr) = msg.treasury {
        if let Some(code_hash) = msg.treasury_code_hash {
            messages.push(register_receive(
                env.contract.code_hash.clone(),
                None,
                &Contract {
                    address: addr,
                    code_hash
                }
            )?);
        }
    }

    messages.push(register_receive(
        env.contract.code_hash,
        None,
        msg.staked_token
    )?);

    Ok(Response::new())
}

fn pad_response(response: StdResult<Response>) -> StdResult<Response> {
    response.map(|mut response| {
        response.data = response.data.map(|mut data| {
            space_pad(RESPONSE_BLOCK_SIZE, &mut data.0);
            data
        });
        response
    })
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let contract_status = ReadonlyConfig::from_storage(deps.storage).contract_status();

    match contract_status {
        ContractStatusLevel::NormalRun => {} // If it's a normal run just continue
        _ => {
            let mut not_authorized = false;
            let status_code = status_level_to_u8(contract_status);

            match msg.clone() {
                // This is always allowed
                ExecuteMsg::SetContractStatus { .. } => {}
                ExecuteMsg::UpdateStakeConfig { .. } => {}

                // If receive check that msg is not bonding or reward
                ExecuteMsg::Receive { msg, .. } => {
                    let receive_type: ReceiveType;
                    if let Some(msg) = msg {
                        receive_type = from_binary(&msg)?;
                    } else {
                        return Err(StdError::generic_err("No receive type supplied in message"));
                    }

                    match receive_type {
                        ReceiveType::Bond { .. } | ReceiveType::Reward => not_authorized = true,
                        _ => {}
                    }
                }
                // Relates to bonding
                ExecuteMsg::StakeRewards { .. } => {
                    if status_code > 0 {
                        not_authorized = true;
                    }
                }

                ExecuteMsg::ClaimRewards { .. } => {
                    if status_code > 1 {
                        not_authorized = true;
                    }
                }
                // If unbonding check that msg is not stop all
                ExecuteMsg::Unbond { .. } => {
                    if status_code > 2 {
                        not_authorized = true;
                    }
                }
                ExecuteMsg::ClaimUnbond { .. } => {
                    if status_code > 2 {
                        not_authorized = true;
                    }
                }
                // All other msgs can only work if status is 1 or below
                _ => {
                    if status_code > 1 {
                        not_authorized = true;
                    }
                }
            }

            if not_authorized {
                return pad_response(Err(StdError::generic_err(
                    "This contract is stopped and this action is not allowed",
                )));
            }
        }
    };

    let response = match msg {
        // Staking
        ExecuteMsg::UpdateStakeConfig {
            unbond_time,
            disable_treasury,
            treasury,
            ..
        } => try_update_stake_config(deps, env, info, unbond_time, disable_treasury, treasury),
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            memo,
            ..
        } => try_receive(deps, env, info, sender, from, amount, msg, memo),
        ExecuteMsg::Unbond { amount, .. } => try_unbond(deps, env, info, amount),
        ExecuteMsg::ClaimUnbond { .. } => try_claim_unbond(deps, env, info),
        ExecuteMsg::ClaimRewards { .. } => try_claim_rewards(deps, env, info),
        ExecuteMsg::StakeRewards { .. } => try_stake_rewards(deps, env, info),

        // Balance
        ExecuteMsg::ExposeBalance {
            recipient,
            code_hash,
            msg,
            memo,
            ..
        } => try_expose_balance(deps, env, info, recipient, code_hash, msg, memo),
        ExecuteMsg::ExposeBalanceWithCooldown {
            recipient,
            code_hash,
            msg,
            memo,
            ..
        } => try_expose_balance_with_cooldown(deps, env, info, recipient, code_hash, msg, memo),

        // Distributors
        ExecuteMsg::SetDistributorsStatus { enabled, .. } => {
            try_set_distributors_status(deps, env, info, enabled)
        }
        ExecuteMsg::AddDistributors { distributors, .. } => {
            try_add_distributors(deps, env, info, distributors)
        }
        ExecuteMsg::SetDistributors { distributors, .. } => {
            try_set_distributors(deps, env, info, distributors)
        }

        // Base
        ExecuteMsg::Transfer {
            recipient,
            amount,
            memo,
            ..
        } => try_transfer(deps, env, info, recipient, amount, memo),
        ExecuteMsg::Send {
            recipient,
            recipient_code_hash,
            amount,
            msg,
            memo,
            ..
        } => try_send(deps, env, info, recipient, recipient_code_hash, amount, memo, msg),
        ExecuteMsg::BatchTransfer { actions, .. } => try_batch_transfer(deps, env, info, actions),
        ExecuteMsg::BatchSend { actions, .. } => try_batch_send(deps, env, info, actions),
        ExecuteMsg::RegisterReceive { code_hash, .. } => try_register_receive(deps, env, info, code_hash),
        ExecuteMsg::CreateViewingKey { entropy, .. } => try_create_key(deps, env, info, entropy),
        ExecuteMsg::SetViewingKey { key, .. } => try_set_key(deps, env, info, key),

        // Allowance
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expiration,
            ..
        } => try_increase_allowance(deps, env, info, spender, amount, expiration),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expiration,
            ..
        } => try_decrease_allowance(deps, env, info, spender, amount, expiration),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
            memo,
            ..
        } => try_transfer_from(deps, &env, &owner, &recipient, amount, memo),
        ExecuteMsg::SendFrom {
            owner,
            recipient,
            recipient_code_hash,
            amount,
            msg,
            memo,
            ..
        } => try_send_from(
            deps,
            env,
            info,
            owner,
            recipient,
            recipient_code_hash,
            amount,
            memo,
            msg,
        ),
        ExecuteMsg::BatchTransferFrom { actions, .. } => {
            try_batch_transfer_from(deps, &env, actions)
        }
        ExecuteMsg::BatchSendFrom { actions, .. } => try_batch_send_from(deps, env, info, actions),

        // Other
        ExecuteMsg::ChangeAdmin { address, .. } => change_admin(deps, env, info, address),
        ExecuteMsg::SetContractStatus { level, .. } => set_contract_status(deps, env, info, level),
        ExecuteMsg::RevokePermit { permit_name, .. } => revoke_permit(deps, env, info, permit_name),
    };

    pad_response(response)
}

pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::StakeConfig {} => stake_queries::stake_config(deps),
        QueryMsg::TotalStaked {} => stake_queries::total_staked(deps),
        QueryMsg::StakeRate {} => stake_queries::stake_rate(deps),
        QueryMsg::Unbonding {} => stake_queries::unbonding(deps),
        QueryMsg::Unfunded { start, total } => stake_queries::unfunded(deps, start, total),
        QueryMsg::Distributors {} => distributors::distributors(deps),
        QueryMsg::TokenInfo {} => query_token_info(deps.storage),
        QueryMsg::TokenConfig {} => query_token_config(deps.storage),
        QueryMsg::ContractStatus {} => query_contract_status(deps.storage),
        QueryMsg::WithPermit { permit, query } => permit_queries(deps, permit, query),
        _ => viewing_keys_queries(deps, msg),
    }
}

fn permit_queries(
    deps: Deps,
    permit: Permit,
    query: QueryWithPermit,
) -> Result<Binary, StdError> {
    // Validate permit content
    let token_address = ReadonlyConfig::from_storage(deps.storage)
        .constants()?
        .contract_address;

    let account = validate(deps, PREFIX_REVOKED_PERMITS, &permit, token_address)?;

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::Staked { time } => {
            if !permit.check_permission(&Permission::Balance) {
                return Err(StdError::generic_err(format!(
                    "No permission to query balance / stake, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            stake_queries::staked(deps, account, time)
        }
        QueryWithPermit::Balance {} => {
            if !permit.check_permission(&Permission::Balance) {
                return Err(StdError::generic_err(format!(
                    "No permission to query balance, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_balance(deps, &account)
        }
        QueryWithPermit::TransferHistory { page, page_size } => {
            if !permit.check_permission(&Permission::History) {
                return Err(StdError::generic_err(format!(
                    "No permission to query history, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_transfers(deps, &account, page.unwrap_or(0), page_size)
        }
        QueryWithPermit::TransactionHistory { page, page_size } => {
            if !permit.check_permission(&Permission::History) {
                return Err(StdError::generic_err(format!(
                    "No permission to query history, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_transactions(deps, &account, page.unwrap_or(0), page_size)
        }
        QueryWithPermit::Allowance { owner, spender } => {
            if !permit.check_permission(&Permission::Allowance) {
                return Err(StdError::generic_err(format!(
                    "No permission to query allowance, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            if account != owner && account != spender {
                return Err(StdError::generic_err(format!(
                    "Cannot query allowance. Requires permit for either owner {:?} or spender {:?}, got permit for {:?}",
                    owner.as_str(),
                    spender.as_str(),
                    account.as_str()
                )));
            }

            query_allowance(deps, owner, spender)
        }
    }
}

pub fn viewing_keys_queries(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    let (addresses, key) = msg.get_validation_params();

    for address in addresses {
        let canonical_addr = deps.api.canonical_address(address)?;

        let expected_key = read_viewing_key(deps.storage, &canonical_addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {
            return match msg {
                // Base
                QueryMsg::Staked { address, time, .. } => {
                    stake_queries::staked(deps, address, time)
                }
                QueryMsg::Balance { address, .. } => query_balance(deps, &address),
                QueryMsg::TransferHistory {
                    address,
                    page,
                    page_size,
                    ..
                } => query_transfers(deps, &address, page.unwrap_or(0), page_size),
                QueryMsg::TransactionHistory {
                    address,
                    page,
                    page_size,
                    ..
                } => query_transactions(deps, &address, page.unwrap_or(0), page_size),
                QueryMsg::Allowance { owner, spender, .. } => query_allowance(deps, owner, spender),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    to_binary(&QueryAnswer::ViewingKeyError {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })
}

fn query_token_info<S: ReadonlyStorage>(storage: &dyn Storage) -> StdResult<Binary> {
    let config = ReadonlyConfig::from_storage(storage);
    let constants = config.constants()?;

    let total_supply = if constants.total_supply_is_public {
        Some(Uint128::new(config.total_supply()))
    } else {
        None
    };

    to_binary(&QueryAnswer::TokenInfo {
        name: constants.name,
        symbol: constants.symbol,
        decimals: constants.decimals,
        total_supply,
    })
}

fn query_token_config<S: ReadonlyStorage>(storage: &dyn Storage) -> StdResult<Binary> {
    let config = ReadonlyConfig::from_storage(storage);
    let constants = config.constants()?;

    to_binary(&QueryAnswer::TokenConfig {
        public_total_supply: constants.total_supply_is_public,
    })
}

fn query_contract_status<S: ReadonlyStorage>(storage: &dyn Storage) -> StdResult<Binary> {
    let config = ReadonlyConfig::from_storage(storage);

    to_binary(&QueryAnswer::ContractStatus {
        status: config.contract_status(),
    })
}

pub fn query_transfers(
    deps: Deps,
    account: &Addr,
    page: u32,
    page_size: u32,
) -> StdResult<Binary> {
    let address = deps.api.canonical_address(account)?;
    let (txs, total) = get_transfers(&deps.api, deps.storage, &address, page, page_size)?;

    let result = QueryAnswer::TransferHistory {
        txs,
        total: Some(total),
    };
    to_binary(&result)
}

pub fn query_transactions(
    deps: Deps,
    account: &Addr,
    page: u32,
    page_size: u32,
) -> StdResult<Binary> {
    let address = deps.api.canonical_address(account)?;
    let (txs, total) = get_txs(&deps.api, deps.storage, &address, page, page_size)?;

    let result = QueryAnswer::TransactionHistory {
        txs,
        total: Some(total),
    };
    to_binary(&result)
}

pub fn query_balance(
    deps: Deps,
    account: &Addr,
) -> StdResult<Binary> {
    let address = deps.api.canonical_address(account)?;

    let amount =
        Uint128::new(ReadonlyBalances::from_storage(deps.storage).account_amount(&address));
    let response = QueryAnswer::Balance { amount };
    to_binary(&response)
}

fn change_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Addr,
) -> StdResult<Response> {
    let mut config = Config::from_storage(deps.storage);

    check_if_admin(&config, &info.sender)?;

    let mut consts = config.constants()?;
    consts.admin = address;
    config.set_constants(&consts)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::ChangeAdmin { status: Success })?))
}

pub fn try_mint_impl(
    storage: &mut dyn Storage,
    minter: &CanonicalAddr,
    recipient: &CanonicalAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    let raw_amount = amount.u128();

    let mut balances = Balances::from_storage(storage);

    let mut account_balance = balances.balance(recipient);

    if let Some(new_balance) = account_balance.checked_add(raw_amount) {
        account_balance = new_balance;
    } else {
        // This error literally can not happen, since the account's funds are a subset
        // of the total supply, both are stored as u128, and we check for overflow of
        // the total supply just a couple lines before.
        // Still, writing this to cover all overflows.
        return Err(StdError::generic_err(
            "This mint attempt would increase the account's balance above the supported maximum",
        ));
    }

    balances.set_account_balance(recipient, account_balance);

    store_mint(storage, minter, recipient, amount, denom, memo, block)?;

    Ok(())
}

pub fn try_set_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    let vk = ViewingKey(key);

    let message_sender = deps.api.canonical_address(&info.sender)?;
    write_viewing_key(deps.storage, &message_sender, &vk);

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetViewingKey { status: Success })?))
}

pub fn try_create_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> StdResult<Response> {
    let constants = ReadonlyConfig::from_storage(deps.storage).constants()?;
    let prng_seed = constants.prng_seed;

    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.canonical_address(&info.sender)?;
    write_viewing_key(deps.storage, &message_sender, &key);

    Ok(Response::new().set_data(to_binary(&HandleAnswer::CreateViewingKey { key })?))
}

fn set_contract_status(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    status_level: ContractStatusLevel,
) -> StdResult<Response> {
    let mut config = Config::from_storage(deps.storage);

    check_if_admin(&config, &info.sender)?;

    config.set_contract_status(status_level);

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetContractStatus {
            status: Success,
        })?))
}

pub fn query_allowance(
    deps: Deps,
    owner: Addr,
    spender: Addr,
) -> StdResult<Binary> {
    let owner_address = deps.api.canonical_address(&owner)?;
    let spender_address = deps.api.canonical_address(&spender)?;

    let allowance = read_allowance(deps.storage, &owner_address, &spender_address)?;

    let response = QueryAnswer::Allowance {
        owner,
        spender,
        allowance: Uint128::new(allowance.amount),
        expiration: allowance.expiration,
    };
    to_binary(&response)
}

#[allow(clippy::too_many_arguments)]
fn try_transfer_impl(
    deps: DepsMut,
    messages: &mut Vec<CosmosMsg>,
    sender: &Addr,
    sender_canon: &CanonicalAddr,
    recipient: &Addr,
    recipient_canon: &CanonicalAddr,
    amount: Uint128,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,

    distributors: &Option<Vec<Addr>>,
    time: u64,
) -> StdResult<()> {
    // Verify that this transfer is allowed
    if let Some(distributors) = distributors {
        if !distributors.contains(sender) && !distributors.contains(recipient) {
            return Err(StdError::generic_err("unauthorized"));
        }
    }

    let symbol = Config::from_storage(deps.storage).constants()?.symbol;

    let stake_config = StakeConfig::load(deps.storage)?;
    let claim = claim_rewards(deps.storage, &stake_config, sender, sender_canon)?;
    if !claim.is_zero() {
        messages.push(send_msg(
            sender.clone(),
            claim.into(),
            None,
            None,
            None,
            256,
            stake_config.staked_token.code_hash,
            stake_config.staked_token.address,
        )?);

        store_claim_reward(
            deps.storage,
            sender_canon,
            claim,
            symbol.clone(),
            None,
            block,
        )?;
    }

    perform_transfer(
        deps.storage,
        sender,
        sender_canon,
        recipient,
        recipient_canon,
        amount,
        time,
    )?;

    store_transfer(
        deps.storage,
        sender_canon,
        sender_canon,
        recipient_canon,
        amount,
        symbol,
        memo,
        block,
    )?;

    Ok(())
}

fn try_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    let sender = info.sender;
    let sender_canon = deps.api.canonical_address(&sender)?;
    let recipient_canon = deps.api.canonical_address(&recipient)?;

    let distributor = get_distributor(deps)?;

    let mut messages = vec![];

    try_transfer_impl(
        deps,
        &mut messages,
        &sender,
        &sender_canon,
        &recipient,
        &recipient_canon,
        amount,
        memo,
        &env.block,
        &distributor,
        env.block.time.seconds(),
    )?;

    let res = Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Transfer { status: Success })?),
    };
    Ok(res)
}

fn try_batch_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::TransferAction>,
) -> StdResult<Response> {
    let sender = info.sender;
    let sender_canon = deps.api.canonical_address(&sender)?;

    let distributor = get_distributor(deps)?;

    let mut messages = vec![];

    for action in actions {
        let recipient = action.recipient;
        let recipient_canon = deps.api.canonical_address(&recipient)?;
        try_transfer_impl(
            deps,
            &mut messages,
            &sender,
            &sender_canon,
            &recipient,
            &recipient_canon,
            action.amount,
            action.memo,
            &env.block,
            &distributor,
            env.block.time.seconds(),
        )?;
    }

    let res = Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchTransfer { status: Success })?),
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
fn try_add_receiver_api_callback<S: ReadonlyStorage>(
    storage: &dyn Storage,
    messages: &mut Vec<CosmosMsg>,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    msg: Option<Binary>,
    sender: Addr,
    from: Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<()> {
    if let Some(receiver_hash) = recipient_code_hash {
        let receiver_msg = Snip20ReceiveMsg::new(sender, from, amount, memo, msg);
        let callback_msg = receiver_msg.into_cosmos_msg(receiver_hash, recipient)?;

        messages.push(callback_msg);
        return Ok(());
    }

    let receiver_hash = get_receiver_hash(storage, &recipient);
    if let Some(receiver_hash) = receiver_hash {
        let receiver_hash = receiver_hash?;
        let receiver_msg = Snip20ReceiveMsg::new(sender, from, amount, memo, msg);
        let callback_msg = receiver_msg.into_cosmos_msg(receiver_hash, recipient)?;

        messages.push(callback_msg);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn try_send_impl(
    deps: DepsMut,
    messages: &mut Vec<CosmosMsg>,
    sender: Addr,
    sender_canon: &CanonicalAddr, // redundant but more efficient
    recipient: Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>,
    block: &shade_protocol::c_std::BlockInfo,

    distributors: &Option<Vec<Addr>>,
    time: u64,
) -> StdResult<()> {
    let recipient_canon = deps.api.canonical_address(&recipient)?;
    try_transfer_impl(
        deps,
        messages,
        &sender,
        sender_canon,
        &recipient,
        &recipient_canon,
        amount,
        memo.clone(),
        block,
        distributors,
        time,
    )?;

    try_add_receiver_api_callback(
        deps.storage,
        messages,
        recipient,
        recipient_code_hash,
        msg,
        sender.clone(),
        sender,
        amount,
        memo,
    )?;

    Ok(())
}

fn try_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let mut messages = vec![];
    let sender = info.sender;
    let sender_canon = deps.api.canonical_address(&sender)?;

    let distributor = get_distributor(deps)?;

    try_send_impl(
        deps,
        &mut messages,
        sender,
        &sender_canon,
        recipient,
        recipient_code_hash,
        amount,
        memo,
        msg,
        &env.block,
        &distributor,
        env.block.time.seconds(),
    )?;

    let res = Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Send { status: Success })?),
    };
    Ok(res)
}

fn try_batch_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::SendAction>,
) -> StdResult<Response> {
    let mut messages = vec![];
    let sender = info.sender;
    let sender_canon = deps.api.canonical_address(&sender)?;

    let distributor = get_distributor(deps)?;

    for action in actions {
        try_send_impl(
            deps,
            &mut messages,
            sender.clone(),
            &sender_canon,
            action.recipient,
            action.recipient_code_hash,
            action.amount,
            action.memo,
            action.msg,
            &env.block,
            &distributor,
            env.block.time.seconds(),
        )?;
    }

    let res = Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchSend { status: Success })?),
    };
    Ok(res)
}

fn try_register_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_hash: String,
) -> StdResult<Response> {
    set_receiver_hash(deps.storage, &info.sender, code_hash);
    let res = Response {
        messages: vec![],
        log: vec![log("register_status", "success")],
        data: Some(to_binary(&HandleAnswer::RegisterReceive {
            status: Success,
        })?),
    };
    Ok(res)
}

fn insufficient_allowance(allowance: u128, required: u128) -> StdError {
    StdError::generic_err(format!(
        "insufficient allowance: allowance={}, required={}",
        allowance, required
    ))
}

fn use_allowance(
    storage: &mut dyn Storage,
    env: &Env,
    owner: &CanonicalAddr,
    spender: &CanonicalAddr,
    amount: u128,
) -> StdResult<()> {
    let mut allowance = read_allowance(storage, owner, spender)?;

    if allowance.is_expired_at(&env.block) {
        return Err(insufficient_allowance(0, amount));
    }
    if let Some(new_allowance) = allowance.amount.checked_sub(amount) {
        allowance.amount = new_allowance;
    } else {
        return Err(insufficient_allowance(allowance.amount, amount));
    }

    write_allowance(storage, owner, spender, allowance)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn try_transfer_from_impl(
    deps: DepsMut,
    env: &Env,
    spender: &Addr,
    spender_canon: &CanonicalAddr,
    owner: &Addr,
    owner_canon: &CanonicalAddr,
    recipient: &Addr,
    recipient_canon: &CanonicalAddr,
    amount: Uint128,
    memo: Option<String>,

    distributors: &Option<Vec<Addr>>,
    time: u64,
) -> StdResult<()> {
    // Verify that this transfer is allowed
    if let Some(distributors) = distributors {
        if !distributors.contains(spender)
            && !distributors.contains(owner)
            && !distributors.contains(recipient)
        {
            return Err(StdError::generic_err("unauthorized"));
        }
    }

    let raw_amount = amount.u128();

    use_allowance(
        deps.storage,
        env,
        owner_canon,
        spender_canon,
        raw_amount,
    )?;

    perform_transfer(
        deps.storage,
        owner,
        owner_canon,
        recipient,
        recipient_canon,
        amount,
        time,
    )?;

    let symbol = Config::from_storage(deps.storage).constants()?.symbol;

    store_transfer(
        deps.storage,
        owner_canon,
        spender_canon,
        recipient_canon,
        amount,
        symbol,
        memo,
        &env.block,
    )?;

    Ok(())
}

fn try_transfer_from(
    deps: DepsMut,
    env: &Env,
    owner: &Addr,
    recipient: &Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    let spender = &info.sender;
    let spender_canon = deps.api.canonical_address(spender)?;
    let owner_canon = deps.api.canonical_address(owner)?;
    let recipient_canon = deps.api.canonical_address(recipient)?;
    try_transfer_from_impl(
        deps,
        env,
        spender,
        &spender_canon,
        owner,
        &owner_canon,
        recipient,
        &recipient_canon,
        amount,
        memo,
        &get_distributor(deps)?,
        env.block.time.seconds(),
    )?;

    let res = Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::TransferFrom { status: Success })?),
    };
    Ok(res)
}

fn try_batch_transfer_from(
    deps: DepsMut,
    env: &Env,
    actions: Vec<batch::TransferFromAction>,
) -> StdResult<Response> {
    let spender = &info.sender;
    let spender_canon = deps.api.canonical_address(spender)?;

    let distributor = get_distributor(deps)?;

    for action in actions {
        let owner_canon = deps.api.canonical_address(&action.owner)?;
        let recipient_canon = deps.api.canonical_address(&action.recipient)?;
        try_transfer_from_impl(
            deps,
            env,
            spender,
            &spender_canon,
            &action.owner,
            &owner_canon,
            &action.recipient,
            &recipient_canon,
            action.amount,
            action.memo,
            &distributor,
            env.block.time.seconds(),
        )?;
    }

    let res = Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchTransferFrom {
            status: Success,
        })?),
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
fn try_send_from_impl(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    messages: &mut Vec<CosmosMsg>,
    spender: &Addr,
    spender_canon: &CanonicalAddr, // redundant but more efficient
    owner: Addr,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>,

    distributors: &Option<Vec<Addr>>,
) -> StdResult<()> {
    let owner_canon = deps.api.canonical_address(&owner)?;
    let recipient_canon = deps.api.canonical_address(&recipient)?;
    try_transfer_from_impl(
        deps,
        &env,
        spender,
        spender_canon,
        &owner,
        &owner_canon,
        &recipient,
        &recipient_canon,
        amount,
        memo.clone(),
        distributors,
        env.block.time.seconds(),
    )?;

    try_add_receiver_api_callback(
        deps.storage,
        messages,
        recipient,
        recipient_code_hash,
        msg,
        info.sender,
        owner,
        amount,
        memo,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn try_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Addr,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let spender = &info.sender.clone();
    let spender_canon = deps.api.canonical_address(spender)?;

    let mut messages = vec![];
    try_send_from_impl(
        deps,
        env,
        &mut messages,
        spender,
        &spender_canon,
        owner,
        recipient,
        recipient_code_hash,
        amount,
        memo,
        msg,
        &get_distributor(deps)?,
    )?;

    let res = Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SendFrom { status: Success })?),
    };
    Ok(res)
}

fn try_batch_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::SendFromAction>,
) -> StdResult<Response> {
    let spender = &info.sender;
    let spender_canon = deps.api.canonical_address(spender)?;
    let mut messages = vec![];

    let distributor = get_distributor(deps)?;

    for action in actions {
        try_send_from_impl(
            deps,
            env.clone(),
            &mut messages,
            spender,
            &spender_canon,
            action.owner,
            action.recipient,
            action.recipient_code_hash,
            action.amount,
            action.memo,
            action.msg,
            &distributor,
        )?;
    }

    let res = Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchSendFrom { status: Success })?),
    };
    Ok(res)
}

fn try_increase_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: Addr,
    amount: Uint128,
    expiration: Option<u64>,
) -> StdResult<Response> {
    let owner_address = deps.api.canonical_address(&info.sender)?;
    let spender_address = deps.api.canonical_address(&spender)?;

    let mut allowance = read_allowance(deps.storage, &owner_address, &spender_address)?;

    // If the previous allowance has expired, reset the allowance.
    // Without this users can take advantage of an expired allowance given to
    // them long ago.
    if allowance.is_expired_at(&env.block) {
        allowance.amount = amount.u128();
        allowance.expiration = None;
    } else {
        allowance.amount = allowance.amount.saturating_add(amount.u128());
    }

    if expiration.is_some() {
        allowance.expiration = expiration;
    }
    let new_amount = allowance.amount;
    write_allowance(
        deps.storage,
        &owner_address,
        &spender_address,
        allowance,
    )?;

    let res = Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::IncreaseAllowance {
            owner: info.sender,
            spender,
            allowance: Uint128::new(new_amount),
        })?),
    };
    Ok(res)
}

fn try_decrease_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: Addr,
    amount: Uint128,
    expiration: Option<u64>,
) -> StdResult<Response> {
    let owner_address = deps.api.canonical_address(&info.sender)?;
    let spender_address = deps.api.canonical_address(&spender)?;

    let mut allowance = read_allowance(deps.storage, &owner_address, &spender_address)?;

    // If the previous allowance has expired, reset the allowance.
    // Without this users can take advantage of an expired allowance given to
    // them long ago.
    if allowance.is_expired_at(&env.block) {
        allowance.amount = 0;
        allowance.expiration = None;
    } else {
        allowance.amount = allowance.amount.saturating_sub(amount.u128());
    }

    if expiration.is_some() {
        allowance.expiration = expiration;
    }
    let new_amount = allowance.amount;
    write_allowance(
        deps.storage,
        &owner_address,
        &spender_address,
        allowance,
    )?;

    let res = Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::DecreaseAllowance {
            owner: info.sender,
            spender,
            allowance: Uint128::new(new_amount),
        })?),
    };
    Ok(res)
}

fn perform_transfer<T: Storage>(
    store: &mut T,
    from: &Addr,
    from_canon: &CanonicalAddr,
    to: &Addr,
    to_canon: &CanonicalAddr,
    amount: Uint128,
    time: u64,
) -> StdResult<()> {
    let mut balances = Balances::from_storage(store);

    let mut from_balance = balances.balance(from_canon);
    let from_tokens = from_balance;

    if let Some(new_from_balance) = from_balance.checked_sub(amount.u128()) {
        from_balance = new_from_balance;
    } else {
        return Err(StdError::generic_err(format!(
            "insufficient funds: balance={}, required={}",
            from_balance, amount
        )));
    }
    balances.set_account_balance(from_canon, from_balance);

    let mut to_balance = balances.balance(to_canon);

    to_balance = to_balance.checked_add(amount.u128()).ok_or_else(|| {
        StdError::generic_err("This tx will literally make them too rich. Try transferring less")
    })?;
    balances.set_account_balance(to_canon, to_balance);

    // Transfer shares
    let total_tokens = TotalTokens::load(store)?;
    let total_shares = TotalShares::load(store)?;

    let config = StakeConfig::load(store)?;

    // calculate shares per token
    let transfer_shares = shares_per_token(&config, &amount, &total_tokens.0, &total_shares.0)?;

    // move shares from one user to another
    let mut from_shares = UserShares::load(store, from.as_str().as_bytes())?;

    from_shares.0 = from_shares.0.checked_sub(transfer_shares)?;
    from_shares.save(store, from.as_str().as_bytes())?;

    let mut to_shares =
        UserShares::may_load(store, to.as_str().as_bytes())?.unwrap_or(UserShares(Uint256::zero()));
    to_shares.0 += transfer_shares;
    to_shares.save(store, to.as_str().as_bytes())?;

    // check for what should be removed from the queue
    let wrapped_amount = amount;

    // Update from cooldown
    remove_from_cooldown(store, from, Uint128::new(from_tokens), wrapped_amount, time)?;

    // Update to cooldown
    {
        let mut to_cooldown =
            UserCooldown::may_load(store, to.as_str().as_bytes())?.unwrap_or(UserCooldown {
                total: Uint128::zero(),
                queue: VecQueue(vec![]),
            });
        // try to remove items that have already passed
        to_cooldown.update(time);
        // add the new cooldown
        to_cooldown.add_cooldown(Cooldown {
            amount: wrapped_amount,
            release: time + StakeConfig::load(store)?.unbond_time,
        });
        to_cooldown.save(store, to.as_str().as_bytes())?;
    }

    Ok(())
}

fn revoke_permit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    permit_name: String,
) -> StdResult<Response> {
    RevokedPermits::revoke_permit(
        deps.storage,
        PREFIX_REVOKED_PERMITS,
        &info.sender,
        &permit_name,
    );

    Ok(Response::new().set_data(to_binary(&HandleAnswer::RevokePermit { status: Success })?))
}

fn is_admin(config: &Config<S>, account: &Addr) -> StdResult<bool> {
    let consts = config.constants()?;
    if &consts.admin != account {
        return Ok(false);
    }

    Ok(true)
}

pub fn check_if_admin(config: &Config<S>, account: &Addr) -> StdResult<()> {
    if !is_admin(config, account)? {
        return Err(StdError::generic_err(
            "This is an admin command. Admin commands can only be run from admin address",
        ));
    }

    Ok(())
}

fn is_valid_name(name: &str) -> bool {
    let len = name.len();
    (3..=30).contains(&len)
}

fn is_valid_symbol(symbol: &str) -> bool {
    let len = symbol.len();
    let len_is_valid = (3..=6).contains(&len);

    len_is_valid && symbol.bytes().all(|byte| (b'A'..=b'Z').contains(&byte))
}

// pub fn migrate(
//     _deps: DepsMut,
//     _env: Env,
//     _msg: MigrateMsg,
// ) -> StdResult<MigrateResponse> {
//     Ok(MigrateResponse::default())
// }

#[cfg(test)]
mod staking_tests {
    use super::*;
    use crate::msg::{InitConfig, ResponseStatus};
    use shade_protocol::c_std::Uint256;
    use shade_protocol::c_std::{
        from_binary,
        testing::*,
        BlockInfo,
        ContractInfo,
        MessageInfo,
        Binary,
        WasmMsg,
    };
    use shade_protocol::{
        contract_interfaces::staking::snip20_staking::ReceiveType,
        utils::asset::Contract,
    };
    use std::any::Any;

    fn init_helper_staking() -> (
        StdResult<Response>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: "sec-sec".to_string(),
            admin: Some(Addr::unchecked("admin".to_string())),
            symbol: "SECSEC".to_string(),
            decimals: Some(8),
            share_decimals: 18,
            prng_seed: Binary::from("lolz fun yay".as_bytes()),
            config: None,
            unbond_time: 10,
            staked_token: Contract {
                address: Addr::unchecked("token".to_string()),
                code_hash: "hash".to_string(),
            },
            treasury: Some(Addr::unchecked("treasury".to_string())),
            treasury_code_hash: None,
            limit_transfer: true,
            distributors: Some(vec![Addr::unchecked("distributor".to_string())]),
        };

        (instantiate(&mut deps, env, info, init_msg), deps)
    }

    // Handle tests
    #[test]
    fn test_handle_update_stake_config() {
        let (init_result, mut deps) = init_helper_staking();

        let handle_msg = ExecuteMsg::UpdateStakeConfig {
            unbond_time: Some(100),
            disable_treasury: true,
            treasury: None,
            padding: None,
        };
        // Check that only admins can interact
        let handle_result = execute(&mut deps, mock_env("not_admin", &[]), handle_msg.clone());
        assert!(handle_result.is_err());
        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg);
        assert!(handle_result.is_ok());

        let query_balance_msg = QueryMsg::StakeConfig {};

        let query_response = query(&deps, query_balance_msg).unwrap();
        let config = match from_binary(&query_response).unwrap() {
            QueryAnswer::StakedConfig { config } => config,
            _ => panic!("Unexpected result from query"),
        };

        assert_eq!(config.treasury, None);
        assert_eq!(config.unbond_time, 100);
        assert_eq!(config.decimal_difference, 10);
    }

    fn new_staked_account(
        deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
        acc: &str,
        pwd: &str,
        stake: Uint128,
    ) {
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr(acc.to_string()),
            from: Default::default(),
            amount: stake,
            msg: Some(to_binary(&ReceiveType::Bond { use_from: None }).unwrap()),
            memo: None,
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());
        let handle_msg = ExecuteMsg::SetViewingKey {
            key: pwd.to_string(),
            padding: None,
        };
        let handle_result = execute(deps, mock_env(acc, &[]), handle_msg.clone());
    }

    fn check_staked_state(
        deps: &Extern<MockStorage, MockApi, MockQuerier>,
        expected_tokens: Uint128,
        expected_shares: Uint256,
    ) {
        let query_balance_msg = QueryMsg::TotalStaked {};

        let query_response = query(&deps, query_balance_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::TotalStaked { shares, tokens } => {
                assert_eq!(tokens, expected_tokens);
                assert_eq!(shares, expected_shares)
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_receive_bonding() {
        let (init_result, mut deps) = init_helper_staking();

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("foo".to_string()),
            from: Default::default(),
            amount: Uint128::new(100 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Bond { use_from: None }).unwrap()),
            memo: None,
            padding: None,
        };
        // Bond tokens with unsupported token
        let handle_result = execute(&mut deps, mock_env("not_token", &[]), handle_msg.clone());
        assert!(handle_result.is_err());
        // Bond tokens
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());
        let handle_msg = ExecuteMsg::SetViewingKey {
            key: "key".to_string(),
            padding: None,
        };
        // Bond tokens with unsupported token
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());

        check_staked_state(
            &deps,
            Uint128::new(100 * 10u128.pow(8)),
            Uint256::from(100 * 10u128.pow(18)),
        );

        new_staked_account(&mut deps, "bar", "key", Uint128::new(100 * 10u128.pow(8)));
        // Query user stake
        let query_balance_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_balance_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(100 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(100 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };
        check_staked_state(
            &deps,
            Uint128::new(200 * 10u128.pow(8)),
            Uint256::from(200 * 10u128.pow(18)),
        );
    }

    #[test]
    fn test_handle_unbond() {
        let (init_result, mut deps) = init_helper_staking();

        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));

        // Query unbonding queue
        let query_msg = QueryMsg::Unbonding {};

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Unbonding { total } => {
                assert_eq!(total, Uint128::zero());
            }
            _ => panic!("Unexpected result from query"),
        };

        // Unbond more than allowed
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(1000 * 10u128.pow(8)),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        // Unbond
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(50 * 10u128.pow(8)),
            padding: None,
        };
        // Set time for ease of prediction
        let mut env = mock_env("foo", &[]);
        env.block.time.seconds() = 10;
        let handle_result = execute(&mut deps, env, info, handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query unbonding queue
        let query_msg = QueryMsg::Unbonding {};

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Unbonding { total } => {
                assert_eq!(total, Uint128::new(50 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Query unbonding queue
        let query_msg = QueryMsg::Unfunded { start: 0, total: 1 };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Unfunded { total } => {
                assert_eq!(total, Uint128::new(50 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Query user stake
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(50 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };
        check_staked_state(
            &deps,
            Uint128::new(50 * 10u128.pow(8)),
            Uint256::from(50 * 10u128.pow(18)),
        );
    }

    #[test]
    fn test_handle_fund_unbond() {
        let (init_result, mut deps) = init_helper_staking();

        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));

        // Bond some amount
        // Unbond
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(50 * 10u128.pow(8)),
            padding: None,
        };
        // Set time for ease of prediction
        let mut env = mock_env("foo", &[]);
        env.block.time.seconds() = 10;
        let handle_result = execute(&mut deps, env, info, handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query unbonding queue
        let query_msg = QueryMsg::Unfunded { start: 0, total: 1 };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Unfunded { total } => {
                assert_eq!(total, Uint128::new(50 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Fund half the unbond
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("treasury".to_string()),
            from: Default::default(),
            amount: Uint128::new(25 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Unbond).unwrap()),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query unbonding queue
        let query_msg = QueryMsg::Unfunded { start: 0, total: 1 };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Unfunded { total } => {
                assert_eq!(total, Uint128::new(25 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Unbond in the middle of funding
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(25 * 10u128.pow(8)),
            padding: None,
        };
        // Set time for ease of prediction
        let mut env = mock_env("foo", &[]);
        env.block.time.seconds() = 10;
        let handle_result = execute(&mut deps, env, info, handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query unbonding queue
        let query_msg = QueryMsg::Unfunded { start: 0, total: 1 };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Unfunded { total } => {
                assert_eq!(total, Uint128::new(50 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Overflow unbond
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("treasury".to_string()),
            from: Default::default(),
            amount: Uint128::new(500 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Unbond).unwrap()),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query unbonding queue
        let query_msg = QueryMsg::Unfunded { start: 0, total: 1 };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Unfunded { total } => {
                assert_eq!(total, Uint128::zero());
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_claim_unbond() {
        let (init_result, mut deps) = init_helper_staking();

        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));

        // Bond some amount
        // Unbond
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(25 * 10u128.pow(8)),
            padding: None,
        };
        // Set time for ease of prediction
        let mut env = mock_env("foo", &[]);
        env.block.time.seconds() = 0;
        let handle_result = execute(&mut deps, env, info, handle_msg.clone());
        assert!(handle_result.is_ok());

        // Fund the unbond
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("treasury".to_string()),
            from: Default::default(),
            amount: Uint128::new(25 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Unbond).unwrap()),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query user stake
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(75 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(75 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::new(25 * 10u128.pow(8)));
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        // Try to claim when its funded but the date hasn't been reached
        let handle_msg = ExecuteMsg::ClaimUnbond { padding: None };
        let mut env = mock_env("foo", &[]);
        env.block.time.seconds() = 0;
        let handle_result = execute(&mut deps, env, info, handle_msg.clone());
        assert!(handle_result.is_err());

        // Query user stake
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: Some(10),
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(75 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(75 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, Some(Uint128::new(25 * 10u128.pow(8))));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Claim
        let handle_msg = ExecuteMsg::ClaimUnbond { padding: None };
        let mut env = mock_env("foo", &[]);
        env.block.time.seconds() = 11;
        let handle_result = execute(&mut deps, env, info, handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query user stake
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: Some(10),
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(75 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(75 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, Some(Uint128::zero()));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Try to claim when its not funded and the date has been reached
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(25 * 10u128.pow(8)),
            padding: None,
        };
        // Set time for ease of prediction
        let mut env = mock_env("foo", &[]);
        env.block.time.seconds() = 0;
        let handle_result = execute(&mut deps, env, info, handle_msg.clone());
        assert!(handle_result.is_ok());

        // Claim
        let handle_msg = ExecuteMsg::ClaimUnbond { padding: None };
        let mut env = mock_env("foo", &[]);
        env.block.time.seconds() = 11;
        let handle_result = execute(&mut deps, env, info, handle_msg.clone());
        assert!(handle_result.is_err());
    }

    #[test]
    fn test_handle_fund_and_claim_rewards() {
        let (init_result, mut deps) = init_helper_staking();

        // Foo should get 2x more rewards than bar
        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));
        new_staked_account(&mut deps, "bar", "key", Uint128::new(50 * 10u128.pow(8)));

        // Claim rewards
        let handle_msg = ExecuteMsg::ClaimRewards { padding: None };

        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        // Add rewards; foo should get 50 tkn and bar 25
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("treasury".to_string()),
            from: Default::default(),
            amount: Uint128::new(75 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Reward).unwrap()),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query user stake
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(100 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(100 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        // Query user stake
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(50 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::new(25 * 10u128.pow(8)));
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        // Total tokens should be total staked plus the rewards
        check_staked_state(
            &deps,
            Uint128::new(225 * 10u128.pow(8)),
            Uint256::from(150 * 10u128.pow(18)),
        );

        // Claim rewards
        let handle_msg = ExecuteMsg::ClaimRewards { padding: None };

        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(100 * 10u128.pow(8)));
                assert!(shares < Uint256::from(100 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_stake_rewards() {
        let (init_result, mut deps) = init_helper_staking();

        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));

        // Add rewards
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("treasury".to_string()),
            from: Default::default(),
            amount: Uint128::new(50 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Reward).unwrap()),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check account to confirm it works
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(100 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(100 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        let handle_msg = ExecuteMsg::StakeRewards { padding: None };
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(150 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(100 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_unbond_with_rewards() {
        let (init_result, mut deps) = init_helper_staking();

        // Foo should get 2x more rewards than bar
        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));
        new_staked_account(&mut deps, "bar", "key", Uint128::new(50 * 10u128.pow(8)));

        // Add rewards; foo should get 50 tkn and bar 25
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("treasury".to_string()),
            from: Default::default(),
            amount: Uint128::new(75 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Reward).unwrap()),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Query user stake
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(100 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(100 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        // Query user stake
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(50 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::new(25 * 10u128.pow(8)));
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        // Total tokens should be total staked plus the rewards
        check_staked_state(
            &deps,
            Uint128::new(225 * 10u128.pow(8)),
            Uint256::from(150 * 10u128.pow(18)),
        );

        // Unbond more than allowed
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(50 * 10u128.pow(8)),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(50 * 10u128.pow(8)));
                assert!(shares < Uint256::from(50 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_set_distributors_status() {
        let (init_result, mut deps) = init_helper_staking();
        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));

        let handle_msg = ExecuteMsg::SetDistributorsStatus {
            enabled: false,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("other", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());
    }

    #[test]
    fn test_handle_add_distributors() {
        let (init_result, mut deps) = init_helper_staking();

        let query_msg = QueryMsg::Distributors {};

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Distributors { distributors } => {
                assert_eq!(distributors.unwrap().len(), 1);
            }
            _ => panic!("Unexpected result from query"),
        };

        let handle_msg = ExecuteMsg::AddDistributors {
            distributors: vec![Addr::unchecked("new_distrib".to_string())],
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("not_admin", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        let query_msg = QueryMsg::Distributors {};

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Distributors { distributors } => {
                let distrib = distributors.unwrap();
                assert_eq!(distrib.len(), 2);
                assert_eq!(distrib[1], Addr::unchecked("new_distrib".to_string()));
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_set_distributors() {
        let (init_result, mut deps) = init_helper_staking();

        let handle_msg = ExecuteMsg::SetDistributors {
            distributors: vec![Addr::unchecked("new_distrib".to_string())],
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("not_admin", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        let query_msg = QueryMsg::Distributors {};

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Distributors { distributors } => {
                let distrib = distributors.unwrap();
                assert_eq!(distrib.len(), 1);
                assert_eq!(distrib[0], Addr::unchecked("new_distrib".to_string()));
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_send_with_distributors() {
        let (init_result, mut deps) = init_helper_staking();
        new_staked_account(
            &mut deps,
            "sender",
            "key",
            Uint128::new(100 * 10u128.pow(8)),
        );
        new_staked_account(
            &mut deps,
            "distrib",
            "key",
            Uint128::new(100 * 10u128.pow(8)),
        );
        new_staked_account(
            &mut deps,
            "not_distrib",
            "key",
            Uint128::new(100 * 10u128.pow(8)),
        );

        let handle_msg = ExecuteMsg::SetDistributors {
            distributors: vec![Addr::unchecked("distrib".to_string())],
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Distrib is sender
        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("someone".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(10 * 10u128.pow(8)),
            msg: None,
            memo: None,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("not_distrib", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        let handle_result = execute(&mut deps, mock_env("distrib", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Send to distrib
        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("distrib".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(10 * 10u128.pow(8)),
            msg: None,
            memo: None,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("sender", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("not_distrib".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(10 * 10u128.pow(8)),
            msg: None,
            memo: None,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("sender", &[]), handle_msg.clone());
        assert!(handle_result.is_err());
    }

    #[test]
    fn test_handle_send_with_rewards() {
        let (init_result, mut deps) = init_helper_staking();
        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));

        let handle_msg = ExecuteMsg::SetDistributorsStatus {
            enabled: false,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Add rewards
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("treasury".to_string()),
            from: Default::default(),
            amount: Uint128::new(50 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Reward).unwrap()),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check account to confirm it works
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(100 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(100 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::new(50 * 10u128.pow(8)));
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        // Send msg
        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("other".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(10 * 10u128.pow(8)),
            msg: None,
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check that it was autoclaimed
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("foo".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(90 * 10u128.pow(8)));
                assert!(shares < Uint256::from(90 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_send_cooldown() {
        let (init_result, mut deps) = init_helper_staking();
        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));
        new_staked_account(&mut deps, "bar", "key", Uint128::new(100 * 10u128.pow(8)));

        let handle_msg = ExecuteMsg::SetDistributorsStatus {
            enabled: false,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Send msg
        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("bar".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(10 * 10u128.pow(8)),
            msg: None,
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check that it was autoclaimed
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                cooldown,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(110 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(110 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
                assert_eq!(cooldown.0.len(), 1);
                assert_eq!(cooldown.0[0].amount, Uint128::new(10 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Send msg
        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("foo".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(100 * 10u128.pow(8)),
            msg: None,
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bar", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check that it was autoclaimed
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                cooldown,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(10 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(10 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
                assert_eq!(cooldown.0.len(), 1);
                assert_eq!(cooldown.0[0].amount, Uint128::new(10 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Send msg
        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("foo".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(10 * 10u128.pow(8)),
            msg: None,
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bar", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check that it was autoclaimed
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                cooldown,
                ..
            } => {
                assert_eq!(tokens, Uint128::zero());
                assert_eq!(shares, Uint256::zero());
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
                assert_eq!(cooldown.0.len(), 0);
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_unbond_cooldown() {
        let (init_result, mut deps) = init_helper_staking();
        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));
        new_staked_account(&mut deps, "bar", "key", Uint128::new(100 * 10u128.pow(8)));

        let handle_msg = ExecuteMsg::SetDistributorsStatus {
            enabled: false,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Send msg
        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("bar".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(10 * 10u128.pow(8)),
            msg: None,
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check that it was autoclaimed
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                cooldown,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(110 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(110 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
                assert_eq!(cooldown.0.len(), 1);
                assert_eq!(cooldown.0[0].amount, Uint128::new(10 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Unbond
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(100 * 10u128.pow(8)),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bar", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check that it was autoclaimed
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                cooldown,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(10 * 10u128.pow(8)));
                assert_eq!(shares, Uint256::from(10 * 10u128.pow(18)));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::new(100 * 10u128.pow(8)));
                assert_eq!(unbonded, None);
                assert_eq!(cooldown.0.len(), 1);
                assert_eq!(cooldown.0[0].amount, Uint128::new(10 * 10u128.pow(8)));
            }
            _ => panic!("Unexpected result from query"),
        };

        // Unbond
        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(10 * 10u128.pow(8)),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bar", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        // Check that it was autoclaimed
        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bar".to_string()),
            key: "key".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                cooldown,
                ..
            } => {
                assert_eq!(tokens, Uint128::zero());
                assert_eq!(shares, Uint256::zero());
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::new(110 * 10u128.pow(8)));
                assert_eq!(unbonded, None);
                assert_eq!(cooldown.0.len(), 0);
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_stop_bonding() {
        let (init_result, mut deps) = init_helper_staking();
        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));

        let handle_msg = ExecuteMsg::SetDistributorsStatus {
            enabled: false,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        let pause_msg = ExecuteMsg::SetContractStatus {
            level: ContractStatusLevel::StopBonding,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), pause_msg);
        assert!(handle_result.is_ok());

        let send_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("account".to_string()),
            amount: Uint128::new(123),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("foo", &[]), send_msg);
        assert!(handle_result.is_ok());

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("foo".to_string()),
            from: Default::default(),
            amount: Uint128::new(100 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Bond { use_from: None }).unwrap()),
            memo: None,
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("foo".to_string()),
            from: Default::default(),
            amount: Uint128::new(100 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Reward).unwrap()),
            memo: None,
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(10 * 10u128.pow(8)),
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());
    }

    #[test]
    fn test_handle_stop_all_but_unbond() {
        let (init_result, mut deps) = init_helper_staking();
        new_staked_account(&mut deps, "foo", "key", Uint128::new(100 * 10u128.pow(8)));

        let handle_msg = ExecuteMsg::SetDistributorsStatus {
            enabled: false,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());

        let pause_msg = ExecuteMsg::SetContractStatus {
            level: ContractStatusLevel::StopAllButUnbond,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), pause_msg);
        assert!(handle_result.is_ok());

        let send_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("account".to_string()),
            amount: Uint128::new(123),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("foo", &[]), send_msg);
        assert!(handle_result.is_err());

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("foo".to_string()),
            from: Default::default(),
            amount: Uint128::new(100 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Bond { use_from: None }).unwrap()),
            memo: None,
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("foo".to_string()),
            from: Default::default(),
            amount: Uint128::new(100 * 10u128.pow(8)),
            msg: Some(to_binary(&ReceiveType::Reward).unwrap()),
            memo: None,
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_err());

        let handle_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(10 * 10u128.pow(8)),
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(&mut deps, mock_env("foo", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());
    }
}

#[cfg(test)]
mod snip20_tests {
    use super::*;
    use crate::msg::{InitConfig, ResponseStatus};
    use shade_protocol::c_std::{
        from_binary,
        testing::*,
        BlockInfo,
        Coin,
        ContractInfo,
        MessageInfo,
        Binary,
        WasmMsg,
    };
    use shade_protocol::{
        contract_interfaces::staking::snip20_staking::ReceiveType,
        utils::asset::Contract,
    };
    use std::any::Any;

    // Helper functions
    #[derive(Clone)]
    struct InitBalance {
        pub acc: &'static str,
        pub pwd: &'static str,
        pub stake: Uint128,
    }

    fn new_staked_account(
        deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
        acc: &str,
        pwd: &str,
        stake: Uint128,
    ) {
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr(acc.to_string()),
            from: Default::default(),
            amount: stake,
            msg: Some(to_binary(&ReceiveType::Bond { use_from: None }).unwrap()),
            memo: None,
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_ok());
        let handle_msg = ExecuteMsg::SetViewingKey {
            key: pwd.to_string(),
            padding: None,
        };
        let handle_result = execute(deps, mock_env(acc, &[]), handle_msg.clone());
        assert!(handle_result.is_ok())
    }

    fn init_helper(
        initial_balances: Vec<InitBalance>,
    ) -> (
        StdResult<Response>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("instantiator", &[]);

        let init_msg = InstantiateMsg {
            name: "sec-sec".to_string(),
            admin: Some(Addr::unchecked("admin".to_string())),
            symbol: "SECSEC".to_string(),
            decimals: Some(8),
            share_decimals: 18,
            prng_seed: Binary::from("lolz fun yay".as_bytes()),
            config: None,
            unbond_time: 10,
            staked_token: Contract {
                address: Addr::unchecked("token".to_string()),
                code_hash: "hash".to_string(),
            },
            treasury: Some(Addr::unchecked("treasury".to_string())),
            treasury_code_hash: None,
            limit_transfer: false,
            distributors: None,
        };

        let instantiate = instantiate(&mut deps, env, info, init_msg);

        for account in initial_balances.iter() {
            new_staked_account(&mut deps, account.acc, account.pwd, account.stake);
        }

        (instantiate, deps)
    }

    fn init_helper_with_config(
        initial_balances: Vec<InitBalance>,
        enable_deposit: bool,
        enable_redeem: bool,
        enable_mint: bool,
        enable_burn: bool,
        contract_bal: u128,
    ) -> (
        StdResult<Response>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies(20, &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(contract_bal).into(),
        }]);

        let env = mock_env("instantiator", &[]);
        let init_config: InitConfig = from_binary(&Binary::from(
            format!(
                "{{\"public_total_supply\":false,
            \"enable_deposit\":{},
            \"enable_redeem\":{},
            \"enable_mint\":{},
            \"enable_burn\":{}}}",
                enable_deposit, enable_redeem, enable_mint, enable_burn
            )
            .as_bytes(),
        ))
        .unwrap();
        let init_msg = InstantiateMsg {
            name: "sec-sec".to_string(),
            admin: Some(Addr::unchecked("admin".to_string())),
            symbol: "SECSEC".to_string(),
            decimals: Some(8),
            share_decimals: 18,
            prng_seed: Binary::from("lolz fun yay".as_bytes()),
            config: Some(init_config),
            unbond_time: 10,
            staked_token: Contract {
                address: Addr::unchecked("token".to_string()),
                code_hash: "hash".to_string(),
            },
            treasury: Some(Addr::unchecked("treasury".to_string())),
            treasury_code_hash: None,
            limit_transfer: false,
            distributors: None,
        };

        let instantiate = instantiate(&mut deps, env, info, init_msg);

        for account in initial_balances.iter() {
            new_staked_account(&mut deps, account.acc, account.pwd, account.stake);
        }

        (instantiate, deps)
    }

    /// Will return a ViewingKey only for the first account in `initial_balances`
    fn _auth_query_helper(
        initial_balances: Vec<InitBalance>,
    ) -> (ViewingKey, Extern<MockStorage, MockApi, MockQuerier>) {
        let (init_result, mut deps) = init_helper(initial_balances.clone());
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let account = initial_balances[0].acc;
        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "42".to_string(),
            padding: None,
        };
        let handle_response = execute(&mut deps, mock_env(account, &[]), create_vk_msg).unwrap();
        let vk = match from_binary(&handle_response.data.unwrap()).unwrap() {
            HandleAnswer::CreateViewingKey { key } => key,
            _ => panic!("Unexpected result from execute"),
        };

        (vk, deps)
    }

    fn extract_error_msg<T: Any>(error: StdResult<T>) -> String {
        match error {
            Ok(response) => {
                let bin_err = (&response as &dyn Any)
                    .downcast_ref::<Binary>()
                    .expect("An error was expected, but no error could be extracted");
                match from_binary(bin_err).unwrap() {
                    QueryAnswer::ViewingKeyError { msg } => msg,
                    _ => panic!("Unexpected query answer"),
                }
            }
            Err(err) => match err {
                StdError::GenericErr { msg, .. } => msg,
                _ => panic!("Unexpected result from instantiate"),
            },
        }
    }

    fn ensure_success(handle_result: Response) -> bool {
        let handle_result: HandleAnswer = from_binary(&handle_result.data.unwrap()).unwrap();

        match handle_result {
            HandleAnswer::UpdateStakeConfig { status }
            | HandleAnswer::Receive { status }
            | HandleAnswer::Unbond { status }
            | HandleAnswer::ClaimUnbond { status }
            | HandleAnswer::ClaimRewards { status }
            | HandleAnswer::StakeRewards { status }
            | HandleAnswer::ExposeBalance { status }
            | HandleAnswer::AddDistributors { status }
            | HandleAnswer::SetDistributors { status }
            | HandleAnswer::Transfer { status }
            | HandleAnswer::Send { status }
            | HandleAnswer::RegisterReceive { status }
            | HandleAnswer::SetViewingKey { status }
            | HandleAnswer::TransferFrom { status }
            | HandleAnswer::SendFrom { status }
            | HandleAnswer::ChangeAdmin { status }
            | HandleAnswer::SetContractStatus { status } => {
                matches!(status, ResponseStatus::Success { .. })
            }
            _ => panic!(
                "HandleAnswer not supported for success extraction: {:?}",
                handle_result
            ),
        }
    }

    // Init tests

    #[test]
    fn test_init_sanity() {
        let (init_result, deps) = init_helper(vec![InitBalance {
            acc: "lebron",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);

        let config = ReadonlyConfig::from_storage(deps.storage);
        let constants = config.constants().unwrap();
        assert_eq!(config.total_supply(), 5000);
        assert_eq!(config.contract_status(), ContractStatusLevel::NormalRun);
        assert_eq!(constants.name, "sec-sec".to_string());
        assert_eq!(constants.admin, Addr::unchecked("admin".to_string()));
        assert_eq!(constants.symbol, "STKD-SECSEC".to_string());
        assert_eq!(constants.decimals, 8);
        assert_eq!(
            constants.prng_seed,
            sha_256("lolz fun yay".to_owned().as_bytes())
        );
        assert_eq!(constants.total_supply_is_public, false);
    }

    #[test]
    fn test_init_with_config_sanity() {
        let (init_result, deps) = init_helper_with_config(
            vec![InitBalance {
                acc: "lebron",
                pwd: "pwd",
                stake: Uint128::new(5000),
            }],
            true,
            true,
            true,
            true,
            0,
        );

        let config = ReadonlyConfig::from_storage(deps.storage);
        let constants = config.constants().unwrap();
        assert_eq!(config.total_supply(), 5000);
        assert_eq!(config.contract_status(), ContractStatusLevel::NormalRun);
        assert_eq!(constants.name, "sec-sec".to_string());
        assert_eq!(constants.admin, Addr::unchecked("admin".to_string()));
        assert_eq!(constants.symbol, "STKD-SECSEC".to_string());
        assert_eq!(constants.decimals, 8);
        assert_eq!(
            constants.prng_seed,
            sha_256("lolz fun yay".to_owned().as_bytes())
        );
        assert_eq!(constants.total_supply_is_public, false);
    }

    #[test]
    fn test_total_supply_overflow() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "lebron",
            pwd: "pwd",
            stake: Uint128::new(u128::MAX),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let (init_result, _deps) = init_helper(vec![InitBalance {
            acc: "lebron",
            pwd: "pwd",
            stake: Uint128::new(u128::MAX),
        }]);
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked("giannis".to_string()),
            from: Default::default(),
            amount: Uint128::new(1),
            msg: Some(to_binary(&ReceiveType::Bond { use_from: None }).unwrap()),
            memo: None,
            padding: None,
        };
        // Bond tokens
        let handle_result = execute(&mut deps, mock_env("token", &[]), handle_msg.clone());
        assert!(handle_result.is_err());
    }

    #[test]
    fn test_handle_transfer() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bob".to_string()),
            key: "pwd".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(5000));
                assert_eq!(shares, Uint256::from(50000000000000u128));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        let query_balance_msg = QueryMsg::TotalStaked {};

        let query_response = query(&deps, query_balance_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::TotalStaked { shares, tokens } => {
                assert_eq!(tokens, Uint128::new(5000));
                assert_eq!(shares, Uint256::from(50000000000000u128))
            }
            _ => panic!("Unexpected result from query"),
        };

        let handle_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(1000),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));
        let bob_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("bob".to_string()))
            .unwrap();
        let alice_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("alice".to_string()))
            .unwrap();
        let balances = ReadonlyBalances::from_storage(deps.storage);
        assert_eq!(5000 - 1000, balances.account_amount(&bob_canonical));
        assert_eq!(1000, balances.account_amount(&alice_canonical));

        let handle_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(10000),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains("insufficient funds"));

        let query_msg = QueryMsg::Staked {
            address: Addr::unchecked("bob".to_string()),
            key: "pwd".to_string(),
            time: None,
        };

        let query_response = query(&deps, query_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::Staked {
                tokens,
                shares,
                pending_rewards,
                unbonding,
                unbonded,
                ..
            } => {
                assert_eq!(tokens, Uint128::new(4000));
                assert_eq!(shares, Uint256::from(40000000000000u128));
                assert_eq!(pending_rewards, Uint128::zero());
                assert_eq!(unbonding, Uint128::zero());
                assert_eq!(unbonded, None);
            }
            _ => panic!("Unexpected result from query"),
        };

        let query_balance_msg = QueryMsg::TotalStaked {};

        let query_response = query(&deps, query_balance_msg).unwrap();
        match from_binary(&query_response).unwrap() {
            QueryAnswer::TotalStaked { shares, tokens } => {
                assert_eq!(tokens, Uint128::new(5000));
                assert_eq!(shares, Uint256::from(50000000000000u128))
            }
            _ => panic!("Unexpected result from query"),
        };
    }

    #[test]
    fn test_handle_send() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::RegisterReceive {
            code_hash: "this_is_a_hash_of_a_code".to_string(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("contract", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));

        let handle_msg = ExecuteMsg::Send {
            recipient: Addr::unchecked("contract".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(100),
            memo: Some("my memo".to_string()),
            padding: None,
            msg: Some(to_binary("hey hey you you").unwrap()),
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result.clone()));
        assert!(
            result.messages.contains(&CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: Addr::unchecked("contract".to_string()),
                callback_code_hash: "this_is_a_hash_of_a_code".to_string(),
                msg: Snip20ReceiveMsg::new(
                    Addr::unchecked("bob".to_string()),
                    Addr::unchecked("bob".to_string()),
                    Uint128::new(100),
                    Some("my memo".to_string()),
                    Some(to_binary("hey hey you you").unwrap())
                )
                .into_binary()
                .unwrap(),
                send: vec![]
            }))
        );
    }

    #[test]
    fn test_handle_register_receive() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::RegisterReceive {
            code_hash: "this_is_a_hash_of_a_code".to_string(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("contract", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));

        let hash = get_receiver_hash(deps.storage, &Addr::unchecked("contract".to_string()))
            .unwrap()
            .unwrap();
        assert_eq!(hash, "this_is_a_hash_of_a_code".to_string());
    }

    #[test]
    fn test_handle_create_viewing_key() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::CreateViewingKey {
            entropy: "".to_string(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );
        let answer: HandleAnswer = from_binary(&handle_result.unwrap().data.unwrap()).unwrap();

        let key = match answer {
            HandleAnswer::CreateViewingKey { key } => key,
            _ => panic!("NOPE"),
        };
        let bob_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("bob".to_string()))
            .unwrap();
        let saved_vk = read_viewing_key(deps.storage, &bob_canonical).unwrap();
        assert!(key.check_viewing_key(saved_vk.as_slice()));
    }

    #[test]
    fn test_handle_set_viewing_key() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Set VK
        let handle_msg = ExecuteMsg::SetViewingKey {
            key: "hi lol".to_string(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let unwrapped_result: HandleAnswer =
            from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        assert_eq!(
            to_binary(&unwrapped_result).unwrap(),
            to_binary(&HandleAnswer::SetViewingKey {
                status: ResponseStatus::Success
            })
            .unwrap(),
        );

        // Set valid VK
        let actual_vk = ViewingKey("x".to_string().repeat(VIEWING_KEY_SIZE));
        let handle_msg = ExecuteMsg::SetViewingKey {
            key: actual_vk.0.clone(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let unwrapped_result: HandleAnswer =
            from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        assert_eq!(
            to_binary(&unwrapped_result).unwrap(),
            to_binary(&HandleAnswer::SetViewingKey { status: Success }).unwrap(),
        );
        let bob_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("bob".to_string()))
            .unwrap();
        let saved_vk = read_viewing_key(deps.storage, &bob_canonical).unwrap();
        assert!(actual_vk.check_viewing_key(&saved_vk));
    }

    #[test]
    fn test_handle_transfer_from() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Transfer before allowance
        let handle_msg = ExecuteMsg::TransferFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2500),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("alice", &[]), handle_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains("insufficient allowance"));

        // Transfer more than allowance
        let handle_msg = ExecuteMsg::IncreaseAllowance {
            spender: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2000),
            padding: None,
            expiration: Some(1_571_797_420),
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );
        let handle_msg = ExecuteMsg::TransferFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2500),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("alice", &[]), handle_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains("insufficient allowance"));

        // Transfer after allowance expired
        let handle_msg = ExecuteMsg::TransferFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2000),
            memo: None,
            padding: None,
        };
        let handle_result = execute(
            &mut deps,
            Env {
                block: BlockInfo {
                    height: 12_345,
                    time: 1_571_797_420,
                    chain_id: "cosmos-testnet-14002".to_string(),
                },
                message: MessageInfo {
                    sender: Addr::unchecked("bob".to_string()),
                    sent_funds: vec![],
                },
                contract: ContractInfo {
                    address: Addr::unchecked(MOCK_CONTRACT_ADDR),
                },
                contract_key: Some("".to_string()),
                contract_code_hash: "".to_string(),
            },
            handle_msg,
        );
        let error = extract_error_msg(handle_result);
        assert!(error.contains("insufficient allowance"));

        // Sanity check
        let handle_msg = ExecuteMsg::TransferFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2000),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("alice", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );
        let bob_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("bob".to_string()))
            .unwrap();
        let alice_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("alice".to_string()))
            .unwrap();
        let bob_balance = crate::state::ReadonlyBalances::from_storage(deps.storage)
            .account_amount(&bob_canonical);
        let alice_balance = crate::state::ReadonlyBalances::from_storage(deps.storage)
            .account_amount(&alice_canonical);
        assert_eq!(bob_balance, 5000 - 2000);
        assert_eq!(alice_balance, 2000);
        let total_supply = ReadonlyConfig::from_storage(deps.storage).total_supply();
        assert_eq!(total_supply, 5000);

        // Second send more than allowance
        let handle_msg = ExecuteMsg::TransferFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(1),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("alice", &[]), handle_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains("insufficient allowance"));
    }

    #[test]
    fn test_handle_send_from() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Send before allowance
        let handle_msg = ExecuteMsg::SendFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("alice".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(2500),
            memo: None,
            msg: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("alice", &[]), handle_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains("insufficient allowance"));

        // Send more than allowance
        let handle_msg = ExecuteMsg::IncreaseAllowance {
            spender: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2000),
            padding: None,
            expiration: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );
        let handle_msg = ExecuteMsg::SendFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("alice".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(2500),
            memo: None,
            msg: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("alice", &[]), handle_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains("insufficient allowance"));

        // Sanity check
        let handle_msg = ExecuteMsg::RegisterReceive {
            code_hash: "lolz".to_string(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("contract", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );
        let send_msg = Binary::from(r#"{ "some_msg": { "some_key": "some_val" } }"#.as_bytes());
        let snip20_msg = Snip20ReceiveMsg::new(
            Addr::unchecked("alice".to_string()),
            Addr::unchecked("bob".to_string()),
            Uint128::new(2000),
            Some("my memo".to_string()),
            Some(send_msg.clone()),
        );
        let handle_msg = ExecuteMsg::SendFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("contract".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(2000),
            memo: Some("my memo".to_string()),
            msg: Some(send_msg),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("alice", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );
        assert!(
            handle_result.unwrap().messages.contains(
                &snip20_msg
                    .into_cosmos_msg("lolz".to_string(), Addr::unchecked("contract".to_string()))
                    .unwrap()
            )
        );
        let bob_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("bob".to_string()))
            .unwrap();
        let contract_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("contract".to_string()))
            .unwrap();
        let bob_balance = crate::state::ReadonlyBalances::from_storage(deps.storage)
            .account_amount(&bob_canonical);
        let contract_balance = crate::state::ReadonlyBalances::from_storage(deps.storage)
            .account_amount(&contract_canonical);
        assert_eq!(bob_balance, 5000 - 2000);
        assert_eq!(contract_balance, 2000);
        let total_supply = ReadonlyConfig::from_storage(deps.storage).total_supply();
        assert_eq!(total_supply, 5000);

        // Second send more than allowance
        let handle_msg = ExecuteMsg::SendFrom {
            owner: Addr::unchecked("bob".to_string()),
            recipient: Addr::unchecked("alice".to_string()),
            recipient_code_hash: None,
            amount: Uint128::new(1),
            memo: None,
            msg: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("alice", &[]), handle_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains("insufficient allowance"));
    }

    #[test]
    fn test_handle_decrease_allowance() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::DecreaseAllowance {
            spender: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2000),
            padding: None,
            expiration: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );

        let bob_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("bob".to_string()))
            .unwrap();
        let alice_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("alice".to_string()))
            .unwrap();

        let allowance = read_allowance(deps.storage, &bob_canonical, &alice_canonical).unwrap();
        assert_eq!(allowance, crate::state::Allowance {
            amount: 0,
            expiration: None
        });

        let handle_msg = ExecuteMsg::IncreaseAllowance {
            spender: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2000),
            padding: None,
            expiration: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::DecreaseAllowance {
            spender: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(50),
            padding: None,
            expiration: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );

        let allowance = read_allowance(deps.storage, &bob_canonical, &alice_canonical).unwrap();
        assert_eq!(allowance, crate::state::Allowance {
            amount: 1950,
            expiration: None
        });
    }

    #[test]
    fn test_handle_increase_allowance() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::IncreaseAllowance {
            spender: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2000),
            padding: None,
            expiration: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );

        let bob_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("bob".to_string()))
            .unwrap();
        let alice_canonical = deps
            .api
            .canonical_address(&Addr::unchecked("alice".to_string()))
            .unwrap();

        let allowance = read_allowance(deps.storage, &bob_canonical, &alice_canonical).unwrap();
        assert_eq!(allowance, crate::state::Allowance {
            amount: 2000,
            expiration: None
        });

        let handle_msg = ExecuteMsg::IncreaseAllowance {
            spender: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(2000),
            padding: None,
            expiration: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );

        let allowance = read_allowance(deps.storage, &bob_canonical, &alice_canonical).unwrap();
        assert_eq!(allowance, crate::state::Allowance {
            amount: 4000,
            expiration: None
        });
    }

    #[test]
    fn test_handle_change_admin() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::ChangeAdmin {
            address: Addr::unchecked("bob".to_string()),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );

        let admin = ReadonlyConfig::from_storage(deps.storage)
            .constants()
            .unwrap()
            .admin;
        assert_eq!(admin, Addr::unchecked("bob".to_string()));
    }

    #[test]
    fn test_handle_set_contract_status() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "admin",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::SetContractStatus {
            level: ContractStatusLevel::StopAll,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("admin", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );

        let contract_status = ReadonlyConfig::from_storage(deps.storage).contract_status();
        assert!(matches!(
            contract_status,
            ContractStatusLevel::StopAll { .. }
        ));
    }

    #[test]
    fn test_handle_admin_commands() {
        let admin_err = "Admin commands can only be run from admin address".to_string();
        let (init_result, mut deps) = init_helper_with_config(
            vec![InitBalance {
                acc: "lebron",
                pwd: "pwd",
                stake: Uint128::new(5000),
            }],
            false,
            false,
            true,
            false,
            0,
        );
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let pause_msg = ExecuteMsg::SetContractStatus {
            level: ContractStatusLevel::StopAll,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("not_admin", &[]), pause_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains(&admin_err.clone()));

        let change_admin_msg = ExecuteMsg::ChangeAdmin {
            address: Addr::unchecked("not_admin".to_string()),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("not_admin", &[]), change_admin_msg);
        let error = extract_error_msg(handle_result);
        assert!(error.contains(&admin_err.clone()));
    }

    #[test]
    fn test_handle_pause_all() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "lebron",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let pause_msg = ExecuteMsg::SetContractStatus {
            level: ContractStatusLevel::StopAll,
            padding: None,
        };

        let handle_result = execute(&mut deps, mock_env("admin", &[]), pause_msg);
        assert!(
            handle_result.is_ok(),
            "Pause execute failed: {}",
            handle_result.err().unwrap()
        );

        let send_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("account".to_string()),
            amount: Uint128::new(123),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("admin", &[]), send_msg);
        let error = extract_error_msg(handle_result);
        assert_eq!(
            error,
            "This contract is stopped and this action is not allowed".to_string()
        );
    }

    // Query tests

    #[test]
    fn test_authenticated_queries() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "giannis",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let no_vk_yet_query_msg = QueryMsg::Balance {
            address: Addr::unchecked("giannis".to_string()),
            key: "no_vk_yet".to_string(),
        };
        let query_result = query(&deps, no_vk_yet_query_msg);
        let error = extract_error_msg(query_result);
        assert_eq!(
            error,
            "Wrong viewing key for this address or viewing key not set".to_string()
        );

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "34".to_string(),
            padding: None,
        };
        let handle_response = execute(&mut deps, mock_env("giannis", &[]), create_vk_msg).unwrap();
        let vk = match from_binary(&handle_response.data.unwrap()).unwrap() {
            HandleAnswer::CreateViewingKey { key } => key,
            _ => panic!("Unexpected result from execute"),
        };

        let query_balance_msg = QueryMsg::Balance {
            address: Addr::unchecked("giannis".to_string()),
            key: vk.0,
        };

        let query_response = query(&deps, query_balance_msg).unwrap();
        let balance = match from_binary(&query_response).unwrap() {
            QueryAnswer::Balance { amount } => amount,
            _ => panic!("Unexpected result from query"),
        };
        assert_eq!(balance, Uint128::new(5000));

        let wrong_vk_query_msg = QueryMsg::Balance {
            address: Addr::unchecked("giannis".to_string()),
            key: "wrong_vk".to_string(),
        };
        let query_result = query(&deps, wrong_vk_query_msg);
        let error = extract_error_msg(query_result);
        assert_eq!(
            error,
            "Wrong viewing key for this address or viewing key not set".to_string()
        );
    }

    #[test]
    fn test_query_token_info() {
        let init_name = "sec-sec".to_string();
        let init_admin = Addr::unchecked("admin".to_string());
        let init_symbol = "SECSEC".to_string();
        let init_decimals = 8;
        let init_config: InitConfig = from_binary(&Binary::from(
            r#"{ "public_total_supply": true }"#.as_bytes(),
        ))
        .unwrap();
        let init_supply = Uint128::new(5000);

        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: init_name.clone(),
            admin: Some(init_admin.clone()),
            symbol: init_symbol.clone(),
            decimals: Some(init_decimals.clone()),
            share_decimals: 18,
            prng_seed: Binary::from("lolz fun yay".as_bytes()),
            config: Some(init_config),
            unbond_time: 10,
            staked_token: Contract {
                address: Addr::unchecked("token".to_string()),
                code_hash: "hash".to_string(),
            },
            treasury: Some(Addr::unchecked("treasury".to_string())),
            treasury_code_hash: None,
            limit_transfer: true,
            distributors: None,
        };
        let init_result = instantiate(&mut deps, env, info, init_msg);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        new_staked_account(&mut deps, "giannis", "pwd", init_supply);

        let query_msg = QueryMsg::TokenInfo {};
        let query_result = query(&deps, query_msg);
        assert!(
            query_result.is_ok(),
            "Init failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::TokenInfo {
                name,
                symbol,
                decimals,
                total_supply,
            } => {
                assert_eq!(name, init_name);
                assert_eq!(symbol, "STKD-".to_string() + &init_symbol);
                assert_eq!(decimals, init_decimals);
                assert_eq!(total_supply, Some(Uint128::new(5000)));
            }
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn test_query_token_config() {
        let init_name = "sec-sec".to_string();
        let init_admin = Addr::unchecked("admin".to_string());
        let init_symbol = "SECSEC".to_string();
        let init_decimals = 8;
        let init_config: InitConfig = from_binary(&Binary::from(
            format!(
                "{{\"public_total_supply\":{},
            \"enable_mint\":{},
            \"enable_burn\":{}}}",
                true, true, false
            )
            .as_bytes(),
        ))
        .unwrap();

        let init_supply = Uint128::new(5000);

        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("instantiator", &[]);
        let init_msg = InstantiateMsg {
            name: init_name.clone(),
            admin: Some(init_admin.clone()),
            symbol: init_symbol.clone(),
            decimals: Some(init_decimals.clone()),
            share_decimals: 18,
            prng_seed: Binary::from("lolz fun yay".as_bytes()),
            config: Some(init_config),
            unbond_time: 10,
            staked_token: Contract {
                address: Addr::unchecked("token".to_string()),
                code_hash: "hash".to_string(),
            },
            treasury: Some(Addr::unchecked("treasury".to_string())),
            treasury_code_hash: None,
            limit_transfer: true,
            distributors: None,
        };
        let init_result = instantiate(&mut deps, env, info, init_msg);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        new_staked_account(&mut deps, "giannis", "pwd", init_supply);

        let query_msg = QueryMsg::TokenConfig {};
        let query_result = query(&deps, query_msg);
        assert!(
            query_result.is_ok(),
            "Init failed: {}",
            query_result.err().unwrap()
        );
        let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match query_answer {
            QueryAnswer::TokenConfig {
                public_total_supply,
            } => {
                assert_eq!(public_total_supply, true);
            }
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn test_query_allowance() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "giannis",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::IncreaseAllowance {
            spender: Addr::unchecked("lebron".to_string()),
            amount: Uint128::new(2000),
            padding: None,
            expiration: None,
        };
        let handle_result = execute(&mut deps, mock_env("giannis", &[]), handle_msg);
        assert!(
            handle_result.is_ok(),
            "execute() failed: {}",
            handle_result.err().unwrap()
        );

        let vk1 = ViewingKey("key1".to_string());
        let vk2 = ViewingKey("key2".to_string());

        let query_msg = QueryMsg::Allowance {
            owner: Addr::unchecked("giannis".to_string()),
            spender: Addr::unchecked("lebron".to_string()),
            key: vk1.0.clone(),
        };
        let query_result = query(&deps, query_msg);
        assert!(
            query_result.is_ok(),
            "Query failed: {}",
            query_result.err().unwrap()
        );
        let error = extract_error_msg(query_result);
        assert!(error.contains("Wrong viewing key"));

        let handle_msg = ExecuteMsg::SetViewingKey {
            key: vk1.0.clone(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("lebron", &[]), handle_msg);
        let unwrapped_result: HandleAnswer =
            from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        assert_eq!(
            to_binary(&unwrapped_result).unwrap(),
            to_binary(&HandleAnswer::SetViewingKey {
                status: ResponseStatus::Success
            })
            .unwrap(),
        );

        let handle_msg = ExecuteMsg::SetViewingKey {
            key: vk2.0.clone(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("giannis", &[]), handle_msg);
        let unwrapped_result: HandleAnswer =
            from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        assert_eq!(
            to_binary(&unwrapped_result).unwrap(),
            to_binary(&HandleAnswer::SetViewingKey {
                status: ResponseStatus::Success
            })
            .unwrap(),
        );

        let query_msg = QueryMsg::Allowance {
            owner: Addr::unchecked("giannis".to_string()),
            spender: Addr::unchecked("lebron".to_string()),
            key: vk1.0.clone(),
        };
        let query_result = query(&deps, query_msg);
        let allowance = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::Allowance { allowance, .. } => allowance,
            _ => panic!("Unexpected"),
        };
        assert_eq!(allowance, Uint128::new(2000));

        let query_msg = QueryMsg::Allowance {
            owner: Addr::unchecked("giannis".to_string()),
            spender: Addr::unchecked("lebron".to_string()),
            key: vk2.0.clone(),
        };
        let query_result = query(&deps, query_msg);
        let allowance = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::Allowance { allowance, .. } => allowance,
            _ => panic!("Unexpected"),
        };
        assert_eq!(allowance, Uint128::new(2000));

        let query_msg = QueryMsg::Allowance {
            owner: Addr::unchecked("lebron".to_string()),
            spender: Addr::unchecked("giannis".to_string()),
            key: vk2.0.clone(),
        };
        let query_result = query(&deps, query_msg);
        let allowance = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::Allowance { allowance, .. } => allowance,
            _ => panic!("Unexpected"),
        };
        assert_eq!(allowance, Uint128::new(0));
    }

    #[test]
    fn test_query_balance() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::SetViewingKey {
            key: "key".to_string(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let unwrapped_result: HandleAnswer =
            from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        assert_eq!(
            to_binary(&unwrapped_result).unwrap(),
            to_binary(&HandleAnswer::SetViewingKey {
                status: ResponseStatus::Success
            })
            .unwrap(),
        );

        let query_msg = QueryMsg::Balance {
            address: Addr::unchecked("bob".to_string()),
            key: "wrong_key".to_string(),
        };
        let query_result = query(&deps, query_msg);
        let error = extract_error_msg(query_result);
        assert!(error.contains("Wrong viewing key"));

        let query_msg = QueryMsg::Balance {
            address: Addr::unchecked("bob".to_string()),
            key: "key".to_string(),
        };
        let query_result = query(&deps, query_msg);
        let balance = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::Balance { amount } => amount,
            _ => panic!("Unexpected"),
        };
        assert_eq!(balance, Uint128::new(5000));
    }

    #[test]
    fn test_query_transfer_history() {
        let (init_result, mut deps) = init_helper(vec![InitBalance {
            acc: "bob",
            pwd: "pwd",
            stake: Uint128::new(5000),
        }]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::SetViewingKey {
            key: "key".to_string(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(ensure_success(handle_result.unwrap()));

        let handle_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(1000),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));
        let handle_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("banana".to_string()),
            amount: Uint128::new(500),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));
        let handle_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("mango".to_string()),
            amount: Uint128::new(2500),
            memo: None,
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));

        let query_msg = QueryMsg::TransferHistory {
            address: Addr::unchecked("bob".to_string()),
            key: "key".to_string(),
            page: None,
            page_size: 0,
        };
        let query_result = query(&deps, query_msg);
        // let a: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        // println!("{:?}", a);
        let transfers = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::TransferHistory { txs, .. } => txs,
            _ => panic!("Unexpected"),
        };
        assert!(transfers.is_empty());

        let query_msg = QueryMsg::TransferHistory {
            address: Addr::unchecked("bob".to_string()),
            key: "key".to_string(),
            page: None,
            page_size: 10,
        };
        let query_result = query(&deps, query_msg);
        let transfers = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::TransferHistory { txs, .. } => txs,
            _ => panic!("Unexpected"),
        };
        assert_eq!(transfers.len(), 3);

        let query_msg = QueryMsg::TransferHistory {
            address: Addr::unchecked("bob".to_string()),
            key: "key".to_string(),
            page: None,
            page_size: 2,
        };
        let query_result = query(&deps, query_msg);
        let transfers = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::TransferHistory { txs, .. } => txs,
            _ => panic!("Unexpected"),
        };
        assert_eq!(transfers.len(), 2);

        let query_msg = QueryMsg::TransferHistory {
            address: Addr::unchecked("bob".to_string()),
            key: "key".to_string(),
            page: Some(1),
            page_size: 2,
        };
        let query_result = query(&deps, query_msg);
        let transfers = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::TransferHistory { txs, .. } => txs,
            _ => panic!("Unexpected"),
        };
        assert_eq!(transfers.len(), 1);
    }

    #[test]
    fn test_query_transaction_history() {
        let (init_result, mut deps) = init_helper_with_config(
            vec![InitBalance {
                acc: "bob",
                pwd: "pwd",
                stake: Uint128::new(10000),
            }],
            true,
            true,
            false,
            false,
            0,
        );
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::SetViewingKey {
            key: "key".to_string(),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        assert!(ensure_success(handle_result.unwrap()));

        let handle_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("alice".to_string()),
            amount: Uint128::new(1000),
            memo: Some("my transfer message #1".to_string()),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));

        let handle_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("banana".to_string()),
            amount: Uint128::new(500),
            memo: Some("my transfer message #2".to_string()),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));

        let handle_msg = ExecuteMsg::Transfer {
            recipient: Addr::unchecked("mango".to_string()),
            amount: Uint128::new(2500),
            memo: Some("my transfer message #3".to_string()),
            padding: None,
        };
        let handle_result = execute(&mut deps, mock_env("bob", &[]), handle_msg);
        let result = handle_result.unwrap();
        assert!(ensure_success(result));

        let query_msg = QueryMsg::TransferHistory {
            address: Addr::unchecked("bob".to_string()),
            key: "key".to_string(),
            page: None,
            page_size: 10,
        };
        let query_result = query(&deps, query_msg);
        let transfers = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::TransferHistory { txs, .. } => txs,
            _ => panic!("Unexpected"),
        };
        assert_eq!(transfers.len(), 3);

        let query_msg = QueryMsg::TransactionHistory {
            address: Addr::unchecked("bob".to_string()),
            key: "key".to_string(),
            page: None,
            page_size: 10,
        };
        let query_result = query(&deps, query_msg);
        let transfers = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::TransactionHistory { txs, .. } => txs,
            other => panic!("Unexpected: {:?}", other),
        };

        use crate::transaction_history::{RichTx, TxAction};
        let expected_transfers = [
            RichTx {
                id: 4,
                action: TxAction::Transfer {
                    from: Addr::unchecked("bob".to_string()),
                    sender: Addr::unchecked("bob".to_string()),
                    recipient: Addr::unchecked("mango".to_string()),
                },
                coins: Coin {
                    denom: "STKD-SECSEC".to_string(),
                    amount: Uint128::new(2500).into(),
                },
                memo: Some("my transfer message #3".to_string()),
                block_time: 1571797419,
                block_height: 12345,
            },
            RichTx {
                id: 3,
                action: TxAction::Transfer {
                    from: Addr::unchecked("bob".to_string()),
                    sender: Addr::unchecked("bob".to_string()),
                    recipient: Addr::unchecked("banana".to_string()),
                },
                coins: Coin {
                    denom: "STKD-SECSEC".to_string(),
                    amount: Uint128::new(500).into(),
                },
                memo: Some("my transfer message #2".to_string()),
                block_time: 1571797419,
                block_height: 12345,
            },
            RichTx {
                id: 2,
                action: TxAction::Transfer {
                    from: Addr::unchecked("bob".to_string()),
                    sender: Addr::unchecked("bob".to_string()),
                    recipient: Addr::unchecked("alice".to_string()),
                },
                coins: Coin {
                    denom: "STKD-SECSEC".to_string(),
                    amount: Uint128::new(1000).into(),
                },
                memo: Some("my transfer message #1".to_string()),
                block_time: 1571797419,
                block_height: 12345,
            },
            RichTx {
                id: 1,
                action: TxAction::Stake {
                    staker: Addr::unchecked("bob".to_string()),
                },
                coins: Coin {
                    denom: "STKD-SECSEC".to_string(),
                    amount: Uint128::new(10000).into(),
                },
                memo: None,
                block_time: 1571797419,
                block_height: 12345,
            },
        ];

        assert_eq!(transfers, expected_transfers);
    }
}
