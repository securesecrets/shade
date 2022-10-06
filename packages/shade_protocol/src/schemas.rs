use cosmwasm_schema::{export_schema, remove_schemas};
use schemars::schema_for;
use std::{env::current_dir, fs::create_dir_all};

#[macro_export]
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

pub fn main() {
    generate_schemas!(
        airdrop,
        bonds,
        governance,
        peg_stability,
        query_auth,
        sky,
        snip20
    );

    //TODO: custom impl for admin, mint, oracles, dex, dao and staking
}
