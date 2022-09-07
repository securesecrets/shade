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
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::Profile,
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
                    assemblyMsg: ID::assembly_msg(deps.storage)?,
                    profile: ID::profile(deps.storage)?,
                    contract: ID::contract(deps.storage)?,
                }),
            }
            .to_cosmos_msg(label, id, code_hash, vec![])?,
            0,
        ))
        .set_data(to_binary(&HandleAnswer::Migrate {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_migrate_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: MigrationDataAsk,
    total: u128,
) -> StdResult<Response> {
    let res = Response::new();

    let config = Config::load(deps.storage)?;

    match RuntimeState::load(deps.storage)? {
        // Fail if not migrating
        RuntimeState::Normal | RuntimeState::SpecificAssemblies { .. } => {
            return Err(StdError::generic_err("No migration has started"));
        }
        RuntimeState::Migrated => {
            if let Some(target) = config.migrated_to {
                let res_data: MigrationData;

                // Go over the migration data asks, there might be a cleaner way to do this
                match data {
                    MigrationDataAsk::Assembly => {
                        let mut assemblies = vec![];

                        // Get the next id to pick
                        let current_id = ID::assembly_migration(deps.storage)?.u128();
                        let last_id = min(current_id + total, ID::assembly(deps.storage)?.u128());

                        // iterate from next over to last
                        for i in current_id..=last_id {
                            let id = Uint128::new(i);
                            assemblies.push((id.clone(), Assembly::load(deps.storage, &id)?));
                        }

                        ID::set_assembly_migration(deps.storage, Uint128::new(last_id))?;

                        res_data = MigrationData::Assembly { data: assemblies };
                    }
                    MigrationDataAsk::AssemblyMsg => {
                        let mut assembly_msgs = vec![];

                        let current_id = ID::assembly_msg_migration(deps.storage)?.u128();
                        let last_id =
                            min(current_id + total, ID::assembly_msg(deps.storage)?.u128());

                        for i in current_id..=last_id {
                            let id = Uint128::new(i);
                            assembly_msgs.push((id.clone(), AssemblyMsg::load(deps.storage, &id)?));
                        }

                        ID::set_assembly_msg_migration(deps.storage, Uint128::new(last_id))?;

                        res_data = MigrationData::AssemblyMsg {
                            data: assembly_msgs,
                        }
                    }
                    MigrationDataAsk::Profile => {
                        let mut profiles = vec![];

                        let current_id = ID::profile_migration(deps.storage)?.u128();
                        let last_id = min(current_id + total, ID::profile(deps.storage)?.u128());

                        for i in current_id..=last_id {
                            let id = Uint128::new(i);
                            profiles.push((id.clone(), Profile::load(deps.storage, &id)?));
                        }

                        ID::set_profile_migration(deps.storage, Uint128::new(last_id))?;

                        res_data = MigrationData::Profile { data: profiles };
                    }
                    MigrationDataAsk::Contract => {
                        let mut contracts = vec![];

                        let current_id = ID::contract_migration(deps.storage)?.u128();
                        let last_id = min(current_id + total, ID::assembly(deps.storage)?.u128());

                        for i in current_id..=last_id {
                            let id = Uint128::new(i);
                            contracts.push((id.clone(), AllowedContract::load(deps.storage, &id)?));
                        }

                        ID::set_contract_migration(deps.storage, Uint128::new(last_id))?;

                        res_data = MigrationData::Contract { data: contracts };
                    }
                };

                return Ok(res
                    .add_message(
                        ReceiveMigrationData { data: res_data }.to_cosmos_msg(&target, vec![])?,
                    )
                    .set_data(to_binary(&HandleAnswer::MigrateData {
                        status: ResponseStatus::Success,
                    })?));
            } else {
                return Err(StdError::generic_err("No migration target found"));
            }
        }
    };
}

pub fn try_receive_migration_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: MigrationData,
) -> StdResult<Response> {
    let res = Response::new();

    let config = Config::load(deps.storage)?;

    if let Some(from) = config.migrated_from {
        if from.address != info.sender {
            return Err(StdError::generic_err("Unauthorized"));
        }

        match data {
            MigrationData::Assembly { data } => {
                for item in data {
                    item.1.save(deps.storage, &item.0)?;
                }
            }
            MigrationData::AssemblyMsg { data } => {
                for item in data {
                    item.1.save(deps.storage, &item.0)?;
                }
            }
            MigrationData::Profile { data } => {
                for item in data {
                    item.1.save(deps.storage, &item.0)?;
                }
            }
            MigrationData::Contract { data } => {
                for item in data {
                    item.1.save(deps.storage, &item.0)?;
                }
            }
        }
    } else {
        return Err(StdError::generic_err("No target found"));
    }

    Ok(res.set_data(to_binary(&HandleAnswer::ReceiveMigrationData {
        status: ResponseStatus::Success,
    })?))
}
