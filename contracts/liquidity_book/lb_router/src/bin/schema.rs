use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env, fs::create_dir_all, path::PathBuf};

use shade_protocol::liquidity_book::lb_router::{
    ExecuteMsg, InitMsg, InvokeMsg, QueryMsg, QueryMsgResponse,
};

fn main() {
    // Get the directory of the current crate
    let mut out_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
    out_dir.push("schema");

    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InvokeMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(QueryMsgResponse), &out_dir);
}
