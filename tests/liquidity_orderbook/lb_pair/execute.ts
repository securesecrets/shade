import { SecretNetworkClient } from "secretjs";
import * as LBPair from "./types"
import { CustomToken, TokenType } from "../lb_factory/types";

export async function executeSwap(
  client: SecretNetworkClient,
  contractHashPair: string,
  contractAddressPair: string,
  amount: string,
) {
  const msg: LBPair.SwapMsg = {
    swap: {
      swap_for_y: true,
      to: client.address,
      amount_received: amount,
      }
  }
  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressPair,
      code_hash: contractHashPair,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 2_000_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(
      `Failed with the following error:\n ${tx.rawLog}`
    );
  };

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`Swap TX used ${tx.gasUsed} gas`);
}

export async function executeAddLiquidity(
  client: SecretNetworkClient,
  contractHashPair: string,
  contractAddressPair: string,
  bin_step: number,
  tokenX: CustomToken,
  tokenY: CustomToken,
) {
  const liquidityParameters: LBPair.LiquidityParameters = {
    token_x: tokenX,
    token_y: tokenY,
    bin_step: bin_step,
    amount_x: (100 * 10e6).toFixed(0),
    amount_y: (100 * 10e6).toFixed(0),
    amount_x_min: (95 * 10e6).toFixed(0),
    amount_y_min: (95 * 10e6).toFixed(0),
    active_id_desired: 2**23,
    id_slippage: 10,
    delta_ids: [-5,-4,-3,-2,-1,0,1,2,3,4,5],
    distribution_x: [
      0, 0, 0, 0, 0, 0.090909, 0.181818, 0.181818, 0.181818, 0.181818, 0.181818
    ].map((el) => el * 1e18),
    distribution_y: [
      0.181818, 0.181818, 0.181818, 0.181818, 0.181818, 0.090909, 0, 0, 0, 0, 0
    ].map((el) => el * 1e18),
    // to: client.address,
    deadline: 999999999999999
  };

  const msg: LBPair.AddLiquidityMsg = {
    add_liquidity: {
      liquidity_parameters: liquidityParameters
    }
  };

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressPair,
      code_hash: contractHashPair,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 1400000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(
      `Failed with the following error:\n ${tx.rawLog}`
    );
  };

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`Add Liquidity TX used ${tx.gasUsed} gas`);
}

export async function executeRemoveLiquidity(
  client: SecretNetworkClient,
  contractHashPair: string,
  contractAddressPair: string,
  bin_step: number,
  tokenX: CustomToken,
  tokenY: CustomToken,
) {
  const removeLiquidity: LBPair.RemoveLiquidity = {
    token_x: tokenX,
    token_y: tokenY,
    bin_step: bin_step,
    amount_x_min: "950000",
    amount_y_min: "950000",
    amounts: ["31869459388831189549983844374029232670507008000"],
    deadline: 999999999999999,
    ids: [8388608]
  };

  const msg: LBPair.RemoveLiquidityMsg = {
    remove_liquidity: {
      remove_liquidity_params: removeLiquidity
    }
  };

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressPair,
      code_hash: contractHashPair,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 1_400_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(
      `Failed with the following error:\n ${tx.rawLog}`
    );
  };

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`Remove Liquidity TX used ${tx.gasUsed} gas`);
}