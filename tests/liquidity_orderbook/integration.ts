import dotenv from "dotenv";

dotenv.config({ path: ".env" });

const build = "./wasm/";

import {
  clientInfo,
  gasStream,
  initializeAndUploadContract,
  logStream,
  logToFile,
  sleep,
  test_configure_factory,
} from "./helper";
import {
  executeAddLiquidity,
  executeRemoveLiquidity,
  executeSwap,
  get_id,
  get_total_bins,
} from "./lb_pair/execute";
import { queryReserves } from "./lb_pair/query";
import { queryBalance } from "./lb_token/query";

(async () => {
  const currentTime = new Date();
  const currentTimeString = currentTime.toTimeString();
  logToFile(`Deploy Time: ${currentTimeString}`);

  //initialize contrats
  let clientInfo = await initializeAndUploadContract();
  //set factory and initialize lb_pair
  //lb_pair initializes lb_token and lb_staking
  clientInfo = await test_configure_factory(clientInfo);

  // await test_liquidity(clientInfo);

  await test_swaps(clientInfo);

  sleep();

  // await runTestFunction(
  //   test_liquidity,
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory,
  //   contractHashRouter,
  //   contractAddressRouter,
  //   codeIdPair,
  //   contractHashPair,
  //   codeIdToken,
  //   contractHashToken,
  //   tokenX,
  //   tokenY
  // );

  logToFile(`\n\n\n`);
  logStream.end();
  gasStream.end();
})();

async function test_liquidity(info: clientInfo) {
  if ("custom_token" in info.tokenX && "custom_token" in info.tokenY) {
    // increase allowance for Token X
    let tx = await info.client.tx.snip20.increaseAllowance(
      {
        sender: info.client.address,
        contract_address: info.tokenX.custom_token.contract_addr,
        code_hash: info.tokenX.custom_token.token_code_hash,
        msg: {
          increase_allowance: {
            spender: info.contractAddressPair,
            amount: "340282366920938463463374607431768211454",
          },
        },
      },
      {
        gasLimit: 200_000,
      }
    );

    if (tx.code !== 0) {
      throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
    }

    console.log(`Increase Token X Allowance TX used ${tx.gasUsed} gas`);

    await sleep();

    // increase allowance for Token Y
    let tx2 = await info.client.tx.snip20.increaseAllowance(
      {
        sender: info.client.address,
        contract_address: info.tokenY.custom_token.contract_addr,
        code_hash: info.tokenY.custom_token.token_code_hash,
        msg: {
          increase_allowance: {
            spender: info.contractAddressPair,
            amount: "340282366920938463463374607431768211454",
          },
        },
      },
      {
        gasLimit: 200_000,
      }
    );

    if (tx2.code !== 0) {
      throw new Error(`Failed with the following error:\n ${tx2.rawLog}`);
    }

    console.log(`Increase Token Y Allowance TX used ${tx2.gasUsed} gas`);

    const bin_step = 100;

    for (let bins = 1; bins <= 100; bins++) {
      await executeAddLiquidity(
        info.client,
        info.contractHashPair,
        info.contractAddressPair,
        bin_step,
        info.tokenX,
        info.tokenY,
        100_000_000,
        100_000_000,
        bins,
        bins
      );

      let total_bins = get_total_bins(bins, bins);
      let ids: number[] = [];
      let balances: string[] = [];

      for (let idx = 0; idx < total_bins; idx++) {
        let id = get_id(2 ** 23, idx, bins);

        ids.push(id);

        let balancetoken = await queryBalance(
          info.client,
          info.contractHashToken,
          info.contractAddressToken,
          id
        );

        balances.push(balancetoken);
      }

      await executeRemoveLiquidity(
        info.client,
        info.contractHashPair,
        info.contractAddressPair,
        bin_step,
        info.tokenX,
        info.tokenY,
        ids,
        balances
      );

      await sleep();
    }
  }
  // await queryTotalSupply(info.client, info.contractHashPair, info.contractAddressPair).catch(
  //   (e) => console.log(e)
  // );
  // await sleep();

  // await executeRemoveLiquidity(
  //   client,
  //   contractHashPair,
  //   contractAddressPair,
  //   bin_step,
  //   tokenX,
  //   tokenY
  // );
  await sleep();
}

async function test_swaps(info: clientInfo) {
  if ("custom_token" in info.tokenX && "custom_token" in info.tokenY) {
    // increase allowance for Token X
    let tx = await info.client.tx.snip20.increaseAllowance(
      {
        sender: info.client.address,
        contract_address: info.tokenX.custom_token.contract_addr,
        code_hash: info.tokenX.custom_token.token_code_hash,
        msg: {
          increase_allowance: {
            spender: info.contractAddressPair,
            amount: "340282366920938463463374607431768211454",
          },
        },
      },
      {
        gasLimit: 200_000,
      }
    );

    if (tx.code !== 0) {
      throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
    }

    console.log(`Increase Token X Allowance TX used ${tx.gasUsed} gas`);

    await sleep();

    // increase allowance for Token Y
    let tx2 = await info.client.tx.snip20.increaseAllowance(
      {
        sender: info.client.address,
        contract_address: info.tokenY.custom_token.contract_addr,
        code_hash: info.tokenY.custom_token.token_code_hash,
        msg: {
          increase_allowance: {
            spender: info.contractAddressPair,
            amount: "340282366920938463463374607431768211454",
          },
        },
      },
      {
        gasLimit: 200_000,
      }
    );

    if (tx2.code !== 0) {
      throw new Error(`Failed with the following error:\n ${tx2.rawLog}`);
    }

    console.log(`Increase Token Y Allowance TX used ${tx2.gasUsed} gas`);

    const bin_step = 100;
    const amount = 100_000_000;
    let sum = 0;

    sum += amount;
    await executeAddLiquidity(
      info.client,
      info.contractHashPair,
      info.contractAddressPair,
      bin_step,
      info.tokenX,
      info.tokenY,
      amount,
      amount,
      100,
      100
    );

    await executeSwap(
      info.client,
      info.contractHashPair,
      info.contractAddressPair,
      info.tokenX.custom_token.token_code_hash,
      info.tokenX.custom_token.contract_addr,
      sum
    );

    await sleep();

    await executeSwap(
      info.client,
      info.contractHashPair,
      info.contractAddressPair,
      info.tokenY.custom_token.token_code_hash,
      info.tokenY.custom_token.contract_addr,
      sum
    );
    await sleep();
    // await executeSwap(
    //   info.client,
    //   info.contractHashPair,
    //   info.contractAddressPair,
    //   info.tokenY.custom_token.token_code_hash,
    //   info.tokenY.custom_token.contract_addr,
    //   sum
    // );

    // await sleep();

    // await executeSwap(
    //   info.client,
    //   info.contractHashPair,
    //   info.contractAddressPair,
    //   info.tokenX.custom_token.token_code_hash,
    //   info.tokenX.custom_token.contract_addr,
    //   sum
    // );
    // await sleep();

    let reserves_3 = await queryReserves(
      info.client,
      info.contractHashPair,
      info.contractAddressPair
    );
    console.log(`Final Reserves_x: ${reserves_3.reserve_x}`);
    console.log(`Final Reserves_y: ${reserves_3.reserve_y}`);

    await sleep();
    // }

    await sleep();
  }
}
// async function test_pair_queries(
//   client: SecretNetworkClient,
//   contractHashFactory: string,
//   contractAddressFactory: string,
//   contractHashRouter: string,
//   contractAddressRouter: string,
//   codeIdPair: number,
//   contractHashPair: string,
//   codeIdToken: number,
//   contractHashToken: string,
//   tokenX: CustomToken,
//   tokenY: CustomToken
// ) {
//   // TODO: query factory for a pair address after it's created
//   const {
//     lb_pair_information: {
//       lb_pair: {
//         contract: { address: contractAddressPair },
//       },
//     },
//   } = await queryLBPairInformation(
//     client,
//     contractHashFactory,
//     contractAddressFactory,
//     tokenX,
//     tokenY,
//     100
//   );

//   const { reserve_x, reserve_y } = await queryReserves(
//     client,
//     contractHashPair,
//     contractAddressPair
//   );
//   await queryActiveId(client, contractHashPair, contractAddressPair).catch(
//     (e) => console.log(e)
//   );

//   await queryIdfromPrice(client, contractHashPair, contractAddressPair).catch(
//     (e) => console.log(e)
//   );

//   await queryPriceFromId(client, contractHashPair, contractAddressPair).catch(
//     (e) => console.log(e)
//   );

//   await queryStaticFeeParameters(
//     client,
//     contractHashPair,
//     contractAddressPair
//   ).catch((e) => console.log(e));

//   await queryVariableFeeParameters(
//     client,
//     contractHashPair,
//     contractAddressPair
//   ).catch((e) => console.log(e));

//   await queryOracleParameters(
//     client,
//     contractHashPair,
//     contractAddressPair
//   ).catch((e) => console.log(e));

//   await querySwapIn(client, contractHashPair, contractAddressPair).catch((e) =>
//     console.log(e)
//   );

//   await querySwapOut(client, contractHashPair, contractAddressPair).catch((e) =>
//     console.log(e)
//   );
// }

// async function test_gas_limits() {
//   // There is no accurate way to measue gas limits but it is actually very recommended to make sure that the gas that is used by a specific tx makes sense
// }
