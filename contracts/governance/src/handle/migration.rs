use shade_protocol::{
    c_std::{to_binary, wasm_instantiate, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg},
    governance::{
        stored_id::ID,
        Config,
        HandleAnswer,
        InstantiateMsg,
        MigrationData,
        MigrationDataAsk,
        MigrationInit,
        RuntimeState,
    },
    utils::{generic_response::ResponseStatus, storage::plus::ItemStorage, InstantiateCallback},
    Contract,
};

pub fn try_migrate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    label: String,
    code_hash: String,
) -> StdResult<Response> {
    ID::init_migration(deps.storage)?;

    let config = Config::load(deps.storage)?;

    RuntimeState::Migrated {}.save(deps.storage)?;

    let res = Response::new();
    // TODO: handle reply on success : 0 will always be for migration
    // TODO: the callback will be used to define the runtime state
    // TODO: make init response return address and code hash
    res.add_submessage(SubMsg::reply_on_success(
        InstantiateMsg {
            treasury: config.treasury,
            query_auth: config.query,
            assemblies: None,
            funding_token: config.funding_token,
            vote_token: config.vote_token,
            migrator: Some(MigrationInit {
                source: Contract {
                    address: env.contract.address,
                    code_hash: env.contract.code_hash,
                },
                assembly: ID::assembly(deps.storage)?,
                assemblyMsg: ID::assembly_msg(deps.storage)?,
                profile: ID::profile(deps.storage)?,
                contract: ID::contract(deps.storage)?,
            }),
        }
        .to_cosmos_msg(label, id, code_hash, vec![]),
        0,
    ))?;

    res.set_data(to_binary(&HandleAnswer::Migrate {
        status: ResponseStatus::Success,
    })?);
    Ok(res)
}

pub fn try_migrate_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: MigrationDataAsk,
) -> StdResult<Response> {
    let res = Response::new();

    if let RuntimeState::Migrated { .. } = RuntimeState::load(deps.storage)? {}

    res.set_data(to_binary(&HandleAnswer::MigrateData {
        status: ResponseStatus::Success,
    })?);
    Ok(res)
}

pub fn try_receive_migration_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: MigrationData,
) -> StdResult<Response> {
    let res = Response::new();

    res.set_data(to_binary(&HandleAnswer::ReceiveMigrationData {
        status: ResponseStatus::Success,
    })?);
    Ok(res)
}
