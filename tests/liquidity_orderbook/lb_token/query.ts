import { SecretNetworkClient } from "secretjs";
import * as LBToken from './types'

const nameQuery: LBToken.NameQuery = {
  "name": {}
};

const symbolQuery: LBToken.SymbolQuery = {
  "symbol": {}
};

const decimalsQuery: LBToken.DecimalsQuery = {
  "decimals": {}
};

const totalSupplyQuery: LBToken.TotalSupplyQuery = {
  "totalSupply": {
    "id": 8388608
  }
};

const balanceOfQuery: LBToken.BalanceOfQuery = {
  "balanceOf": {
    "owner": "secret1mz0cdjxk72mnqfuy4v6y9c6",
    "id": 123
  }
};

const balanceOfBatchQuery: LBToken.BalanceOfBatchQuery = {
  "balanceOfBatch": {
    "owners": ["secret1mz0cdjxk72mnqfuy4v6y9c6", "secret1mf7tzqxzvqhpv7m62ccq3gq"],
    "ids": [1, 2, 3]
  }
};

const isApprovedForAllQuery: LBToken.IsApprovedForAllQuery = {
  "isApprovedForAll": {
    "owner": "secret1mz0cdjxk72mnqfuy4v6y9c6",
    "spender": "secret1mf7tzqxzvqhpv7m62ccq3gq"
  }
};


export async function queryName(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<string> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: nameQuery,
  })) as LBToken.NameResponse;

  if ('err"' in response) {
    throw new Error(
      `Query failed with the following err: ${JSON.stringify(response)}`
    );
  }

  console.log(JSON.stringify(response));
  return response.name;
}

export async function querySymbol(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<string> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: symbolQuery,
  })) as LBToken.SymbolResponse;

  if ('err"' in response) {
    throw new Error(
      `Query failed with the following err: ${JSON.stringify(response)}`
    );
  }

  console.log(JSON.stringify(response));
  return response.symbol;
}

export async function queryDecimals(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<number> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: decimalsQuery,
  })) as LBToken.DecimalsResponse;

  if ('err"' in response) {
    throw new Error(
      `Query failed with the following err: ${JSON.stringify(response)}`
    );
  }

  console.log(JSON.stringify(response));
  return response.decimals;
}

export async function queryTotalSupply(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string
): Promise<string> {

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: totalSupplyQuery,
  })) as LBToken.TotalSupplyResponse;

  if ('err"' in response) {
    throw new Error(
      `Query failed with the following err: ${JSON.stringify(response)}`
    );
  }

  console.log(JSON.stringify(response));
  return response.total_supply;
}