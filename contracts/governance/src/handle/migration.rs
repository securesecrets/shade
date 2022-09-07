use shade_protocol::{
    c_std::{
        to_binary,
        wasm_instantiate,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        SubMsg,
        Uint128,
    },
    governance::{
        assembly::Assembly,
        stored_id::ID,
        Config,
        ExecuteMsg::ReceiveMigrationData,
        HandleAnswer,
        InstantiateMsg,
        MigrationData,
        MigrationDataAsk,
        MigrationInit,
        RuntimeState,
    },
    utils::{
        generic_response::ResponseStatus,
        storage::plus::ItemStorage,
        ExecuteCallback,
        InstantiateCallback,
    },
    Contract,
};
use std::cmp::min;

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

    Ok(res.set_data(to_binary(&HandleAnswer::Migrate {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_migrate_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: MigrationDataAsk,
    total: u64,
) -> StdResult<Response> {
    let res = Response::new();

    let config = Config::load(deps.storage)?;

    match RuntimeState::load(deps.storage)? {
        RuntimeState::Normal | RuntimeState::SpecificAssemblies { .. } => {
            return Err(StdError::generic_err("No migration has started"));
        }
        _ => {
            if let Some(target) = config.migrated_to {
                let res_msg: ReceiveMigrationData;

                match data {
                    MigrationDataAsk::Assembly => {
                        let mut assemblies = vec![];

                        let current_id = ID::assembly_migration(deps.storage)?.u128();
                        for i in
                            current_id..min(current_id + total, ID::assembly(deps.storage)?.u128())
                        {
                            let id = Uint128::new(i);
                            assemblies.push((id.clone(), Assembly::load(deps.storage, &id)?));
                        }

                        res_msg = ReceiveMigrationData {
                            data: MigrationData::Assembly { data: assemblies },
                        };
                    }
                    MigrationDataAsk::AssemblyMsg => {}
                    MigrationDataAsk::Profile => {}
                    MigrationDataAsk::Contract => {}
                };

                res.add_message(res_msg.to_cosmos_msg(&target, vec![])?)
            } else {
                return Err(StdError::generic_err("No migration target found"));
            }
        }
    };

    Ok(res.set_data(to_binary(&HandleAnswer::MigrateData {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_receive_migration_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: MigrationData,
) -> StdResult<Response> {
    let res = Response::new();

    let config = Config::load(deps.storage)?;

    match RuntimeState::load(deps.storage)? {
        RuntimeState::Normal | RuntimeState::SpecificAssemblies { .. } => {
            return Err(StdError::generic_err("No migration has started"));
        }
        _ => {
            if config.migrated_from.is_none() {
                return Err(StdError::generic_err("No target found"));
            }
        }
    };

    Ok(res.set_data(to_binary(&HandleAnswer::ReceiveMigrationData {
        status: ResponseStatus::Success,
    })?))
}
