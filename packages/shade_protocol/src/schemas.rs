use cosmwasm_schema::{write_api, export_schema, remove_schemas};
use schemars::schema_for;
use std::{env::current_dir, fs::create_dir_all};

macro_rules! generate_schemas {
    ($($Contract:ident),+) => {
        $(
            use shade_protocol::contract_interfaces::$Contract;

            let mut out_dir = current_dir().unwrap();
            out_dir.push("schema");
            out_dir.push(stringify!($Contract));
            create_dir_all(&out_dir).unwrap();
            remove_schemas(&out_dir).unwrap();

            export_schema(&schema_for!($Contract::InstantiateMsg), &out_dir);
            export_schema(&schema_for!($Contract::ExecuteMsg), &out_dir);
            export_schema(&schema_for!($Contract::ExecuteAnswer), &out_dir);
            export_schema(&schema_for!($Contract::QueryMsg), &out_dir);
            export_schema(&schema_for!($Contract::QueryAnswer), &out_dir);
        )+
    };
}

macro_rules! generate_nested_schemas {
    ($Folder:ident, $($Contract:ident),+) => {
        $(
            use shade_protocol::contract_interfaces::$Folder::$Contract;

            let mut out_dir = current_dir().unwrap();
            out_dir.push("schema");
            out_dir.push(stringify!($Folder));
            out_dir.push(stringify!($Contract));
            create_dir_all(&out_dir).unwrap();
            remove_schemas(&out_dir).unwrap();

            export_schema(&schema_for!($Contract::InstantiateMsg), &out_dir);
            export_schema(&schema_for!($Contract::ExecuteMsg), &out_dir);
            export_schema(&schema_for!($Contract::ExecuteAnswer), &out_dir);
            export_schema(&schema_for!($Contract::QueryMsg), &out_dir);
            export_schema(&schema_for!($Contract::QueryAnswer), &out_dir);
        )+
    };
}

macro_rules! generate_nested_schemas_2 {
    ($Folder:ident, $($Contract:ident),+) => {
        $(
            use shade_protocol::contract_interfaces::$Folder::$Contract;

            let mut out_dir = current_dir().unwrap();
            out_dir.push("schema");
            out_dir.push(stringify!($Folder));
            out_dir.push(stringify!($Contract));
            create_dir_all(&out_dir).unwrap();
            remove_schemas(&out_dir).unwrap();

            export_schema(&schema_for!($Contract::InstantiateMsg), &out_dir);
            export_schema(&schema_for!($Contract::ExecuteMsg), &out_dir);
            export_schema(&schema_for!($Contract::QueryMsg), &out_dir);
        )+
    };
}




pub fn main() {
    generate_schemas!(
        airdrop,
        basic_staking,
        bonds,
        governance,
        peg_stability,
        query_auth,
        sky,
        snip20
    );

    // generate_nested_schemas!(mint, liability_mint, mint, mint_router);
    // generate_nested_schemas!(oracles, oracle);
    generate_nested_schemas!(dao, treasury_manager, treasury, scrt_staking, stkd_scrt);

    // TODO: make lb schema generation better. We can't use the write_api! macro in a workspace
    // because it will always write to the same location. So I don't know how to generate the
    // QueryResponse in the was cosmwasm_schema suggests.
    generate_nested_schemas!(liquidity_book, lb_token);
    generate_nested_schemas_2!(liquidity_book, lb_factory, lb_pair);

    // TODO: make admin interface up to standard
    use shade_protocol::contract_interfaces::admin;

    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    out_dir.push("admin");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(admin::InstantiateMsg), &out_dir);
    export_schema(&schema_for!(admin::ExecuteMsg), &out_dir);
    export_schema(&schema_for!(admin::QueryMsg), &out_dir);
}
