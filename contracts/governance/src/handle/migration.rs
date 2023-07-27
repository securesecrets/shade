use shade_protocol::{
    c_std::{to_binary, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg},
    governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        errors::Error,
        profile::Profile,
        stored_id::ID,
        Config,
        ExecuteAnswer,
        ExecuteMsg::ReceiveMigrationData,
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
    _info: MessageInfo,
    id: u64,
    mut label: String,
    code_hash: String,
) -> StdResult<Response> {
    // TODO: maybe randomly generate migration label
    ID::init_migration(deps.storage)?;

    let config = Config::load(deps.storage)?;

    RuntimeState::Migrated {}.save(deps.storage)?;

    label.push_str(&env.block.time.nanos().to_string());

    let res = Response::new();
    Ok(res
        .add_submessage(SubMsg::reply_on_success(
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
                    assembly_msg: ID::assembly_msg(deps.storage)?,
                    profile: ID::profile(deps.storage)?,
                    contract: ID::contract(deps.storage)?,
                }),
            }
            .to_cosmos_msg(label, id, code_hash, vec![])?,
            0,
        ))
        .set_data(to_binary(&ExecuteAnswer::Migrate {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_migrate_data(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    data: MigrationDataAsk,
    total: u16,
) -> StdResult<Response> {
    let res = Response::new();

    let config = Config::load(deps.storage)?;

    match RuntimeState::load(deps.storage)? {
        // Fail if not migrating
        RuntimeState::Normal | RuntimeState::SpecificAssemblies { .. } => {
            return Err(Error::migration_not_started(vec![]));
        }
        RuntimeState::Migrated => {
            if let Some(target) = config.migrated_to {
                let res_data: MigrationData;

                // Go over the migration data asks, there might be a cleaner way to do this
                match data {
                    MigrationDataAsk::Assembly => {
                        let mut assemblies = vec![];

                        // Get the next id to pick
                        let current_id = ID::assembly_migration(deps.storage)?;
                        let last_id = min(current_id + total, ID::assembly(deps.storage)?);

                        // iterate from next over to last
                        for i in current_id..=last_id {
                            assemblies.push((i, Assembly::load(deps.storage, i)?));
                        }

                        ID::set_assembly_migration(deps.storage, last_id)?;

                        res_data = MigrationData::Assembly { data: assemblies };
                    }
                    MigrationDataAsk::AssemblyMsg => {
                        let mut assembly_msgs = vec![];

                        let current_id = ID::assembly_msg_migration(deps.storage)?;
                        let last_id = min(current_id + total, ID::assembly_msg(deps.storage)?);

                        for i in current_id..=last_id {
                            assembly_msgs.push((i, AssemblyMsg::load(deps.storage, i)?));
                        }

                        ID::set_assembly_msg_migration(deps.storage, last_id)?;

                        res_data = MigrationData::AssemblyMsg {
                            data: assembly_msgs,
                        }
                    }
                    MigrationDataAsk::Profile => {
                        let mut profiles = vec![];

                        let current_id = ID::profile_migration(deps.storage)?;
                        let last_id = min(current_id + total, ID::profile(deps.storage)?);

                        for i in current_id..=last_id {
                            profiles.push((i, Profile::load(deps.storage, i)?));
                        }

                        ID::set_profile_migration(deps.storage, last_id)?;

                        res_data = MigrationData::Profile { data: profiles };
                    }
                    MigrationDataAsk::Contract => {
                        let mut contracts = vec![];

                        let current_id = ID::contract_migration(deps.storage)?;
                        let last_id = min(current_id + total, ID::contract(deps.storage)?);

                        for i in current_id..=last_id {
                            contracts.push((i, AllowedContract::load(deps.storage, i)?));
                        }

                        ID::set_contract_migration(deps.storage, last_id)?;

                        res_data = MigrationData::Contract { data: contracts };
                    }
                };

                return Ok(res
                    .add_message(
                        ReceiveMigrationData { data: res_data }.to_cosmos_msg(&target, vec![])?,
                    )
                    .set_data(to_binary(&ExecuteAnswer::MigrateData {
                        status: ResponseStatus::Success,
                    })?));
            } else {
                return Err(Error::migration_tartet(vec![]));
            }
        }
    };
}

pub fn try_receive_migration_data(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: MigrationData,
) -> StdResult<Response> {
    let res = Response::new();

    let config = Config::load(deps.storage)?;

    if let Some(from) = config.migrated_from {
        if from.address != info.sender {
            return Err(Error::no_migrator(vec![]));
        }

        match data {
            MigrationData::Assembly { data } => {
                for item in data {
                    item.1.save(deps.storage, item.0)?;
                }
            }
            MigrationData::AssemblyMsg { data } => {
                for item in data {
                    item.1.save(deps.storage, item.0)?;
                }
            }
            MigrationData::Profile { data } => {
                for item in data {
                    item.1.save(deps.storage, item.0)?;
                }
            }
            MigrationData::Contract { data } => {
                for item in data {
                    item.1.save(deps.storage, item.0)?;
                }
            }
        }
    } else {
        return Err(Error::migration_tartet(vec![]));
    }

    Ok(
        res.set_data(to_binary(&ExecuteAnswer::ReceiveMigrationData {
            status: ResponseStatus::Success,
        })?),
    )
}
