import { SecretNetworkClient } from "secretjs";
import * as LBFactory from "./types";
import { TokenType } from "./types";

const getLBPairImplementationQuery: LBFactory.QueryMsg = {
  get_lb_pair_implementation: {},
};

const getLBTokenImplementationQuery: LBFactory.QueryMsg = {
  get_lb_token_implementation: {},
};

const getPresetQuery: LBFactory.QueryMsg = {
  get_preset: {
    bin_step: 100,
  },
};

export async function queryLBPairImplementation(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBFactory.LBPairImplementationResponse> {
  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getLBPairImplementationQuery,
  })) as LBFactory.LBPairImplementationResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryLBTokenImplementation(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBFactory.LBTokenImplementationResponse> {
  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getLBTokenImplementationQuery,
  })) as LBFactory.LBTokenImplementationResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryPreset(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBFactory.PresetResponse> {
  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getPresetQuery,
  })) as LBFactory.PresetResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryLBPairInformation(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string,
  tokenX: TokenType,
  tokenY: TokenType,
  bin_step: number
): Promise<LBFactory.LBPairInformationResponse> {
  const getAllLBPairsQuery: LBFactory.QueryMsg = {
    get_lb_pair_information: {
      token_x: tokenX,
      token_y: tokenY,
      bin_step: bin_step,
    },
  };

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getAllLBPairsQuery,
  })) as LBFactory.LBPairInformationResponse;

  console.log(JSON.stringify(response));
  return response;
}
