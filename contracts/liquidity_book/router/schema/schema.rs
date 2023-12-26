use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use shade_protocol::swap::router::{ExecuteMsg, InitMsg, InvokeMsg, QueryMsg, QueryMsgResponse};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InvokeMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(QueryMsgResponse), &out_dir);
}
