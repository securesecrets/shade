import { SecretNetworkClient } from "secretjs";
import * as LBToken from "./types";

export async function queryBalance(
  client: SecretNetworkClient,
  contractHash: string,
  contractAddress: string,
  token_id: number
): Promise<string> {
  const balanceQuery: LBToken.QueryMsg = {
    balance: {
      key: "viewing_key",
      owner: client.address,
      token_id: `${token_id}`,
      viewer: client.address,
      address: client.address,
    },
  };

  const response = (await client.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractHash,
    query: balanceQuery,
  })) as LBToken.QueryAnswer;

  if ('err"' in response) {
    throw new Error(
      `Query failed with the following err: ${JSON.stringify(response)}`
    );
  }

  // Check if the response has the 'balance' property
  if ("balance" in response) {
    // console.log(JSON.stringify(response));
    // Assuming amount is a string. If it's not, convert it to a string as needed.
    return response.balance.amount;
  } else {
    throw new Error("Balance data not found in the response");
  }
}
