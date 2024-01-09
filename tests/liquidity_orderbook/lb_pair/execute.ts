import { SecretNetworkClient } from "secretjs";
import { Snip20SendOptions } from "secretjs/src/extensions/snip20/types";
import * as LBPair from "./types";
import { LiquidityParameters, TokenType } from "./types";

export async function executeSwap(
  client: SecretNetworkClient,
  contractHashPair: string,
  contractAddressPair: string,
  contractHashToken: string,
  contractAddressToken: string,
  amount: number
) {
  const msg: LBPair.InvokeMsg = {
    swap_tokens: {},
  };

  const send_msg: Snip20SendOptions = {
    send: {
      recipient: contractAddressPair,
      amount: `${amount.toFixed(0)}`,
      msg: Buffer.from(JSON.stringify(msg)).toString("base64"),
    },
  };

  const tx = await client.tx.snip20.send(
    {
      sender: client.address,
      contract_address: contractAddressToken,
      code_hash: contractHashToken,
      msg: send_msg,
      sent_funds: [],
    },
    {
      gasLimit: 6_000_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`Swap TX used ${tx.gasUsed} gas`);
}

const PRECISION = 1_000_000_000_000_000_000; // 1e18

export function get_total_bins(nb_bin_x: number, nb_bin_y: number): number {
  if (nb_bin_x > 0 && nb_bin_y > 0) {
    return nb_bin_x + nb_bin_y - 1;
  }
  return nb_bin_x + nb_bin_y;
}
export function get_id(active_id: number, i: number, nb_bin_y: number): number {
  let id = active_id + i;

  if (nb_bin_y > 0) {
    id = id - nb_bin_y + 1;
  }

  return safe24(id);
}

// Assuming safe24 is a function that ensures the number is within a 24-bit range
function safe24(num: number): number {
  // Adjust the logic here based on what safe24 is supposed to do in your Rust code
  // Example: ensuring the number doesn't exceed 24-bit max value (16777215)
  const MAX_24_BIT = (1 << 24) - 1;
  return Math.min(num, MAX_24_BIT);
}

function safe64Divide(numerator: number, denominator: number): number {
  if (denominator === 0) {
    throw new Error("Division by zero");
  }

  const result = numerator / denominator;
  const maxU64 = BigInt(2 ** 64 - 1);

  if (result > maxU64) {
    throw new Error("Result exceeds u64 range");
  }

  return result;
}

function multiplyRatio(
  value: number,
  numerator: number,
  denominator: number
): number {
  if (denominator === 0) {
    throw new Error("Division by zero in ratio calculation");
  }
  return (value * numerator) / denominator;
}

function liquidityParametersGenerator(
  binStep: number,
  activeId: number,
  tokenX: TokenType,
  tokenY: TokenType,
  amountX: number, // Assuming Uint128 is represented as bigint in TS
  amountY: number,
  nbBinsX: number,
  nbBinsY: number
): LiquidityParameters {
  if (activeId > 2 ** 24 - 1) {
    throw new Error("active_id too big");
  }

  let total = get_total_bins(nbBinsX, nbBinsY); // Implement get_total_bins function

  let distributionX: number[] = [];
  let distributionY: number[] = [];
  let deltaIds: number[] = [];

  for (let i = 0; i < total; i++) {
    if (nbBinsY > 0) {
      deltaIds.push(i - nbBinsY + 1);
    } else {
      deltaIds.push(i);
    }

    let id = get_id(activeId, i, nbBinsY); // Implement get_id function
    let distribX =
      id >= activeId && nbBinsX > 0 ? safe64Divide(PRECISION, nbBinsX) : 0;
    let distribY =
      id <= activeId && nbBinsY > 0 ? safe64Divide(PRECISION, nbBinsY) : 0;

    distributionX.push(distribX);
    distributionY.push(distribY);
  }

  let liquidityParameters: LiquidityParameters = {
    active_id_desired: activeId,
    amount_x: `${amountX}`,
    amount_x_min: multiplyRatio(amountX, 999, 1000).toFixed(0),
    amount_y: `${amountY}`,
    amount_y_min: multiplyRatio(amountY, 999, 1000).toFixed(0),
    bin_step: binStep,
    deadline: 999999999999,
    delta_ids: deltaIds,
    distribution_x: distributionX,
    distribution_y: distributionY,
    id_slippage: 15,
    token_x: tokenX,
    token_y: tokenY,
  };

  return liquidityParameters;
}

// Implement safe64Divide, get_total_bins, get_id, multiplyRatio and any other required functions or logic

export async function executeAddLiquidity(
  client: SecretNetworkClient,
  contractHashPair: string,
  contractAddressPair: string,
  bin_step: number,
  tokenX: LBPair.TokenType,
  tokenY: LBPair.TokenType,
  amount_x: number,
  amount_y: number,
  no_of_bins_x: number,
  no_of_bins_y: number
) {
  console.log(amount_x);
  console.log(amount_y);
  let liquidity_params = liquidityParametersGenerator(
    bin_step,
    2 ** 23,
    tokenX,
    tokenY,
    amount_x,
    amount_y,
    no_of_bins_x,
    no_of_bins_y
  );

  const msg: LBPair.ExecuteMsg = {
    add_liquidity: {
      liquidity_parameters: liquidity_params,
    },
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
      gasLimit: 16000000,
    }
  );
  if ("custom_token" in tokenX && "custom_token" in tokenY) {
    let contractX = {
      address: tokenX.custom_token.contract_addr,
      code_hash: tokenX.custom_token.token_code_hash,
    };
    let balanceX = await client.query.snip20.getBalance({
      contract: contractX,
      address: client.address,
      auth: {
        key: "viewing_key",
      },
    });
  }
  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  let total_bins = get_total_bins(no_of_bins_x, no_of_bins_y);

  console.log(`total_bins ${total_bins}`);
  // console.log(
  //   `Add Liquidity TX used ${tx.gasUsed} gas, total_bins ${total_bins}`
  // );
}

export async function executeRemoveLiquidity(
  client: SecretNetworkClient,
  contractHashPair: string,
  contractAddressPair: string,
  bin_step: number,
  tokenX: LBPair.TokenType,
  tokenY: LBPair.TokenType,
  ids: number[],
  amounts: string[]
) {
  const removeLiquidity: LBPair.RemoveLiquidity = {
    token_x: tokenX,
    token_y: tokenY,
    bin_step: bin_step,
    amount_x_min: "950000",
    amount_y_min: "950000",
    amounts,
    ids,
    deadline: 999999999999999,
  };

  const msg: LBPair.ExecuteMsg = {
    remove_liquidity: {
      remove_liquidity_params: removeLiquidity,
    },
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
      gasLimit: 6_000_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(
    `Remove Liquidity TX used ${tx.gasUsed} gas, total_bins ${ids.length}`
  );
}
