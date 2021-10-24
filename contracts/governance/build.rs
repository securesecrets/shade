use std::path::PathBuf;

use cosmwasm_schema::{export_schema, schema_for};
use shade_protocol::governance::{HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg};

fn main() {
    // only generate schemas when building in release mode
    if cfg!(debug_assertions) {
        return;
    }

    let package_dir = env!("CARGO_MANIFEST_DIR");
    let schema_dir = format!("{}/schema", package_dir);

    if std::fs::metadata(&schema_dir).is_ok() {
        // if the schemas already exist, only regenerate if the package has changed
        println!("cargo:rerun-if-changed=../../packages/shade_protocol/src");
        std::fs::remove_dir_all(&schema_dir).expect("remove stale schemas");
    }

    std::fs::create_dir(&schema_dir).expect("create schema directory");

    let schema_dir = PathBuf::from(schema_dir);

    export_schema(&schema_for!(InitMsg), &schema_dir);
    export_schema(&schema_for!(HandleMsg), &schema_dir);
    export_schema(&schema_for!(HandleAnswer), &schema_dir);
    export_schema(&schema_for!(QueryMsg), &schema_dir);
    export_schema(&schema_for!(QueryAnswer), &schema_dir);
}
