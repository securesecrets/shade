import { SecretNetworkClient } from "secretjs";
import { logGasToFile } from "../helper";
import { TokenType } from "../lb_factory";
import { ExecuteMsg, Hop, TokenAmount } from "./types";

export async function executeSwapTokensForExact(
  client: SecretNetworkClient,
  contractHashRouter: string,
  contractAddressRouter: string,
  contractHashPair: string,
  contractAddressPair: string,
  tokenX: TokenType,
  amount: string
) {
  const tokenAmount: TokenAmount = {
    token: tokenX,
    amount: amount,
  };

  const hop: Hop = {
    addr: contractAddressPair,
    code_hash: contractHashPair,
  };

  // const msg: SwapTokensForExactMsg = {
  //   swap_tokens_for_exact: {
  //     offer: tokenAmount,
  //     path: [hop],
  //   },
  // };

  const msg: ExecuteMsg = {
    swap_tokens_for_exact: {
      expected_return: null,
      offer: tokenAmount,
      padding: null,
      path: [hop],
      recipient: null,
    },
  };

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressRouter,
      code_hash: contractHashRouter,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 3_000_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`SwapTokensforExact TX used ${tx.gasUsed} gas`);
  logGasToFile(`SwapTokensforExact TX used ${tx.gasUsed} gas`);
}
