use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env, fs::create_dir_all, path::PathBuf};

use shade_protocol::liquidity_book::lb_pair::{
    ActiveIdResponse, AllBinsResponse, BinResponse, BinStepResponse, BinUpdatingHeightsResponse,
    BinsResponse, ExecuteMsg, FactoryResponse, GetPairInfoResponse, IdFromPriceResponse,
    InstantiateMsg, InvokeMsg, LbTokenResponse, MintResponse, NextNonEmptyBinResponse,
    OracleParametersResponse, OracleSampleAtResponse, OracleSamplesAfterResponse,
    OracleSamplesAtResponse, PriceFromIdResponse, ProtocolFeesResponse, QueryMsg, ReservesResponse,
    RewardsDistributionResponse, StakingResponse, StaticFeeParametersResponse, SwapInResponse,
    SwapOutResponse, SwapSimulationResponse, TokenXResponse, TokenYResponse, TokensResponse,
    TotalSupplyResponse, UpdatedBinsAfterHeightResponse, UpdatedBinsAtHeightResponse,
    UpdatedBinsAtMultipleHeightResponse, VariableFeeParametersResponse,
};

fn main() {
    // Get the directory of the current crate
    let mut out_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
    out_dir.push("schema");

    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InvokeMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);

    export_schema(&schema_for!(StakingResponse), &out_dir);
    export_schema(&schema_for!(LbTokenResponse), &out_dir);
    export_schema(&schema_for!(GetPairInfoResponse), &out_dir);
    export_schema(&schema_for!(SwapSimulationResponse), &out_dir);
    export_schema(&schema_for!(FactoryResponse), &out_dir);
    export_schema(&schema_for!(TokensResponse), &out_dir);
    export_schema(&schema_for!(TokenXResponse), &out_dir);
    export_schema(&schema_for!(TokenYResponse), &out_dir);
    export_schema(&schema_for!(BinStepResponse), &out_dir);
    export_schema(&schema_for!(ReservesResponse), &out_dir);
    export_schema(&schema_for!(ActiveIdResponse), &out_dir);
    export_schema(&schema_for!(BinResponse), &out_dir);
    export_schema(&schema_for!(BinsResponse), &out_dir);
    export_schema(&schema_for!(AllBinsResponse), &out_dir);
    export_schema(&schema_for!(UpdatedBinsAtHeightResponse), &out_dir);
    export_schema(&schema_for!(UpdatedBinsAtMultipleHeightResponse), &out_dir);
    export_schema(&schema_for!(UpdatedBinsAfterHeightResponse), &out_dir);
    export_schema(&schema_for!(BinUpdatingHeightsResponse), &out_dir);
    export_schema(&schema_for!(NextNonEmptyBinResponse), &out_dir);
    export_schema(&schema_for!(ProtocolFeesResponse), &out_dir);
    export_schema(&schema_for!(StaticFeeParametersResponse), &out_dir);
    export_schema(&schema_for!(VariableFeeParametersResponse), &out_dir);
    export_schema(&schema_for!(OracleParametersResponse), &out_dir);
    export_schema(&schema_for!(OracleSampleAtResponse), &out_dir);
    export_schema(&schema_for!(OracleSamplesAtResponse), &out_dir);
    export_schema(&schema_for!(OracleSamplesAfterResponse), &out_dir);
    export_schema(&schema_for!(PriceFromIdResponse), &out_dir);
    export_schema(&schema_for!(IdFromPriceResponse), &out_dir);
    export_schema(&schema_for!(SwapInResponse), &out_dir);
    export_schema(&schema_for!(SwapOutResponse), &out_dir);
    export_schema(&schema_for!(TotalSupplyResponse), &out_dir);
    export_schema(&schema_for!(RewardsDistributionResponse), &out_dir);
    export_schema(&schema_for!(MintResponse), &out_dir);
}
