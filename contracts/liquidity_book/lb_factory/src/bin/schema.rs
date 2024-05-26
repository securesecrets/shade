use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env, fs::create_dir_all, path::PathBuf};

use shade_protocol::liquidity_book::lb_factory::{
    AllBinStepsResponse, AllLbPairsResponse, ExecuteMsg, FeeRecipientResponse, InstantiateMsg,
    IsQuoteAssetResponse, LbPairAtIndexResponse, LbPairImplementationResponse,
    LbPairInformationResponse, LbTokenImplementationResponse, MinBinStepResponse,
    NumberOfLbPairsResponse, NumberOfQuoteAssetsResponse, OpenBinStepsResponse, PresetResponse,
    QueryMsg, QuoteAssetAtIndexResponse,
};

fn main() {
    // Get the directory of the current crate
    let mut out_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
    out_dir.push("schema");

    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);

    // Add export_schema for each response struct
    export_schema(&schema_for!(MinBinStepResponse), &out_dir);
    export_schema(&schema_for!(FeeRecipientResponse), &out_dir);
    export_schema(&schema_for!(LbPairImplementationResponse), &out_dir);
    export_schema(&schema_for!(LbTokenImplementationResponse), &out_dir);
    export_schema(&schema_for!(NumberOfLbPairsResponse), &out_dir);
    export_schema(&schema_for!(LbPairAtIndexResponse), &out_dir);
    export_schema(&schema_for!(NumberOfQuoteAssetsResponse), &out_dir);
    export_schema(&schema_for!(QuoteAssetAtIndexResponse), &out_dir);
    export_schema(&schema_for!(IsQuoteAssetResponse), &out_dir);
    export_schema(&schema_for!(LbPairInformationResponse), &out_dir);
    export_schema(&schema_for!(PresetResponse), &out_dir);
    export_schema(&schema_for!(AllBinStepsResponse), &out_dir);
    export_schema(&schema_for!(OpenBinStepsResponse), &out_dir);
    export_schema(&schema_for!(AllLbPairsResponse), &out_dir);
}
