import { SecretNetworkClient } from "secretjs";
import * as LBRouter from "./types"

// These queries are identical to the LBPair queries

const getFactoryQuery: LBRouter.GetFactoryQuery = {
  get_factory: {}
};

const getPriceFromIdQuery: LBRouter.GetPriceFromIdQuery = {
  get_price_from_id: {
    id: 12345
  }
};

const getIdFromPriceQuery: LBRouter.GetIdFromPriceQuery = {
  get_id_from_price: {
    price: "1000000"
  }
};

const getSwapInQuery: LBRouter.GetSwapInQuery = {
  get_swap_in: {
    amount_out: "1000000000000000000",
    swap_for_y: true
  }
};

const getSwapOutQuery: LBRouter.GetSwapOutQuery = {
  get_swap_out: {
    amount_in: "1000000000000000000",
    swap_for_y: false
  }
};

export async function queryFactory(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBRouter.FactoryResponse> {

  const response = (await client.query.compute.queryContract({
      contract_address: contractAddress,
      code_hash: contractHash,
      query: getFactoryQuery,
  })) as LBRouter.FactoryResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function queryPriceFromId(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBRouter.PriceFromIdResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getPriceFromIdQuery,
  })) as LBRouter.PriceFromIdResponse;

  console.log(JSON.stringify(response))
  return response;
}

export async function queryIdfromPrice(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBRouter.IdFromPriceResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getIdFromPriceQuery,
  })) as LBRouter.IdFromPriceResponse;

  console.log(JSON.stringify(response))
  return response;
}

export async function querySwapIn(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBRouter.SwapInResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getSwapInQuery,
  })) as LBRouter.SwapInResponse;

  console.log(JSON.stringify(response));
  return response;
}

export async function querySwapOut(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<LBRouter.SwapOutResponse> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: getSwapOutQuery,
  })) as LBRouter.SwapOutResponse;

  console.log(JSON.stringify(response));
  return response;
}