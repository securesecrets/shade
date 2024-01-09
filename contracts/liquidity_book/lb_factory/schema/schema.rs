use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use shade_protocol::liquidity_book::lb_factory::{
    AllBinStepsResponse,
    AllLBPairsResponse,
    ExecuteMsg,
    FeeRecipientResponse,
    InstantiateMsg,
    IsQuoteAssetResponse,
    LBPairAtIndexResponse,
    LBPairImplementationResponse,
    LBPairInformationResponse,
    LBTokenImplementationResponse,
    MinBinStepResponse,
    NumberOfLBPairsResponse,
    NumberOfQuoteAssetsResponse,
    OpenBinStepsResponse,
    PresetResponse,
    QueryMsg,
    QuoteAssetAtIndexResponse,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);

    // Add export_schema for each response struct
    export_schema(&schema_for!(MinBinStepResponse), &out_dir);
    export_schema(&schema_for!(FeeRecipientResponse), &out_dir);
    export_schema(&schema_for!(LBPairImplementationResponse), &out_dir);
    export_schema(&schema_for!(LBTokenImplementationResponse), &out_dir);
    export_schema(&schema_for!(NumberOfLBPairsResponse), &out_dir);
    export_schema(&schema_for!(LBPairAtIndexResponse), &out_dir);
    export_schema(&schema_for!(NumberOfQuoteAssetsResponse), &out_dir);
    export_schema(&schema_for!(QuoteAssetAtIndexResponse), &out_dir);
    export_schema(&schema_for!(IsQuoteAssetResponse), &out_dir);
    export_schema(&schema_for!(LBPairInformationResponse), &out_dir);
    export_schema(&schema_for!(PresetResponse), &out_dir);
    export_schema(&schema_for!(AllBinStepsResponse), &out_dir);
    export_schema(&schema_for!(OpenBinStepsResponse), &out_dir);
    export_schema(&schema_for!(AllLBPairsResponse), &out_dir);
}
