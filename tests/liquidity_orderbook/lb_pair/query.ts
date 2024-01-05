import { SecretNetworkClient } from "secretjs";
import * as LBPair from "./types";

const getFactoryQuery: LBPair.QueryMsg = {
  get_factory: {},
};

const getLbStakingQuery: LBPair.QueryMsg = {
  get_staking_contract: {},
};

const getLbTokenQuery: LBPair.QueryMsg = {
  get_lb_token: {},
};

const getTokenXQuery: LBPair.QueryMsg = {
  get_token_x: {},
};

const getTokenYQuery: LBPair.QueryMsg = {
  get_token_y: {},
};

const getBinStepQuery: LBPair.QueryMsg = {
  get_bin_step: {},
};

const getReservesQuery: LBPair.QueryMsg = {
  get_reserves: {},
};

const getActiveIdQuery: LBPair.QueryMsg = {
  get_active_id: {},
};

const getNextNonEmptyBinQuery: LBPair.QueryMsg = {
  get_next_non_empty_bin: {
    swap_for_y: true,
    id: 8388608,
  },
};
const totalSupplyQuery: LBPair.QueryMsg = {
  total_supply: {
    id: 8388608,
  },
};
const getProtocolFeesQuery: LBPair.QueryMsg = {
  get_protocol_fees: {},
};

const getStaticFeeParametersQuery: LBPair.QueryMsg = {
  get_static_fee_parameters: {},
};

const getVariableFeeParametersQuery: LBPair.QueryMsg = {
  get_variable_fee_parameters: {},
};

const getOracleParametersQuery: LBPair.QueryMsg = {
  get_oracle_parameters: {},
};

const getOracleSampleAtQuery: LBPair.QueryMsg = {
  get_oracle_sample_at: {
    look_up_timestamp: 1234567890,
  },
};

const getPriceFromIdQuery: LBPair.QueryMsg = {
  get_price_from_id: {
    id: 8388608,
  },
};

const getIdFromPriceQuery: LBPair.QueryMsg = {
  get_id_from_price: {
    price: "924521306405372907020063908180274956666",
  },
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

  //   console.log(JSON.stringify(response));
  return response;
}

export async function queryLbStaking(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.StakingResponse> {
  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getLbStakingQuery,
  })) as LBPair.StakingResponse;

  return response;
}

export async function queryLbToken(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBPair.LbTokenResponse> {
  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getLbTokenQuery,
  })) as LBPair.LbTokenResponse;

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

  // console.log(JSON.stringify(response));
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

  // console.log(JSON.stringify(response));
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

  //   console.log(JSON.stringify(response));
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

  //   console.log(JSON.stringify(response));
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

  //   console.log(JSON.stringify(response));
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

  //   console.log(JSON.stringify(response));
  return response;
}

export async function querySwapIn(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string,
  amount_out: number,
  swap_for_y: boolean
): Promise<LBPair.SwapInResponse> {
  const getSwapInQuery: LBPair.QueryMsg = {
    get_swap_in: {
      amount_out: `${amount_out.toFixed(0)}`,
      swap_for_y: swap_for_y,
    },
  };

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getSwapInQuery,
  })) as LBPair.SwapInResponse;

  //   console.log(JSON.stringify(response));
  return response;
}

export async function querySwapOut(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string,
  amount_in: number,
  swap_for_y: boolean
): Promise<LBPair.SwapOutResponse> {
  const getSwapOutQuery: LBPair.QueryMsg = {
    get_swap_out: {
      amount_in: `${amount_in.toFixed(0)}`,
      swap_for_y,
    },
  };

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getSwapOutQuery,
  })) as LBPair.SwapOutResponse;

  //   console.log(JSON.stringify(response));
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

  //   console.log(JSON.stringify(response));
  return response;
}
