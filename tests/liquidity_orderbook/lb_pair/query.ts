import { SecretNetworkClient } from "secretjs";
import * as LBPair from "./types"

const getFactoryQuery: LBPair.GetFactoryQuery = {
  get_factory: {}
};

const getTokenXQuery: LBPair.GetTokenXQuery = {
  get_token_x: {}
};

const getTokenYQuery: LBPair.GetTokenYQuery = {
  get_token_y: {}
};

const getBinStepQuery: LBPair.GetBinStepQuery = {
  get_bin_step: {}
};

const getReservesQuery: LBPair.GetReservesQuery = {
  get_reserves: {}
};

const getActiveIdQuery: LBPair.GetActiveIdQuery = {
  get_active_id: {}
};

const getBinQuery: LBPair.GetBinQuery = {
  get_bin: {
    id: 1234
  }
};

const getNextNonEmptyBinQuery: LBPair.GetNextNonEmptyBinQuery = {
  get_next_non_empty_bin: {
    swap_for_y: true,
    id: 1234
  }
};
const totalSupplyQuery: LBPair.GetTotalSupplyQuery = {
  "total_supply": {
    "id": 8388608
  }
};
const getProtocolFeesQuery: LBPair.GetProtocolFeesQuery = {
  get_protocol_fees: {}
};

const getStaticFeeParametersQuery: LBPair.GetStaticFeeParametersQuery = {
  get_static_fee_parameters: {}
};

const getVariableFeeParametersQuery: LBPair.GetVariableFeeParametersQuery = {
  get_variable_fee_parameters: {}
};

const getOracleParametersQuery: LBPair.GetOracleParametersQuery = {
  get_oracle_parameters: {}
};

const getOracleSampleAtQuery: LBPair.GetOracleSampleAtQuery = {
  get_oracle_sample_at: {
    look_up_timestamp: 1234567890
  }
};

const getPriceFromIdQuery: LBPair.GetPriceFromIdQuery = {
  get_price_from_id: {
    id: 8388608
  }
};

const getIdFromPriceQuery: LBPair.GetIdFromPriceQuery = {
  get_id_from_price: {
    price: "924521306405372907020063908180274956666"
  }
};

const getSwapInQuery: LBPair.GetSwapInQuery = {
  get_swap_in: {
    amount_out: "1000000",
    swap_for_y: true
  }
};

const getSwapOutQuery: LBPair.GetSwapOutQuery = {
  get_swap_out: {
    amount_in: "100000",
    swap_for_y: false
  }
};

export async function queryFactory(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.FactoryResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getFactoryQuery,
  })) as LBPair.FactoryResponse;

  console.log(JSON.stringify(response));
  return response;
}


export async function queryReserves(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.ReservesResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getReservesQuery,
  })) as LBPair.ReservesResponse;

  console.log(JSON.stringify(response))
  return response;
}

export async function queryActiveId(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.ActiveIdResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getActiveIdQuery,
  })) as LBPair.ActiveIdResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryStaticFeeParameters(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.StaticFeeParametersResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getStaticFeeParametersQuery,
  })) as LBPair.StaticFeeParametersResponse;

  console.log(JSON.stringify(response));
  return response;
}


export async function queryVariableFeeParameters(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.VariableFeeParametersResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getVariableFeeParametersQuery,
  })) as LBPair.VariableFeeParametersResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryOracleParameters(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.OracleParametersResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getOracleParametersQuery,
  })) as LBPair.OracleParametersResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryPriceFromId(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.PriceFromIdResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getPriceFromIdQuery,
  })) as LBPair.PriceFromIdResponse;

  console.log(JSON.stringify(response))
  return response;
}

export async function queryIdfromPrice(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.IdFromPriceResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getIdFromPriceQuery,
  })) as LBPair.IdFromPriceResponse;

  console.log(JSON.stringify(response))
  return response;
}

export async function querySwapIn(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.SwapInResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getSwapInQuery,
  })) as LBPair.SwapInResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function querySwapOut(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.SwapOutResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getSwapOutQuery,
  })) as LBPair.SwapOutResponse;

  console.log(JSON.stringify(response));
  return response;
}
export async function queryTotalSupply(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.SwapOutResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: totalSupplyQuery,
  })) as LBPair.SwapOutResponse;

  console.log(JSON.stringify(response));
  return response;
}