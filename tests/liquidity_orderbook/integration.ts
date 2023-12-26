import dotenv from "dotenv";

dotenv.config({ path: ".env" });

const build = "./wasm/";

import { SecretNetworkClient } from "secretjs";
import {
  gasStream,
  initializeAndUploadContract,
  logStream,
  logToFile,
  runTestFunction,
  sleep,
} from "./helper";
import {
  TokenType,
  executeSetLBPairImplementation,
  executeSetLBTokenImplementation,
} from "./lb_factory";

async function test_configure_factory(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  contractHashRouter: string,
  contractAddressRouter: string,
  codeIdPair: number,
  contractHashPair: string,
  codeIdToken: number,
  contractHashToken: string,
  tokenX: TokenType,
  tokenY: TokenType
) {
  await executeSetLBPairImplementation(
    client,
    contractHashFactory,
    contractAddressFactory,
    codeIdPair,
    contractHashPair
  );
  await sleep();

  // await queryLBPairImplementation(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory
  // );

  await executeSetLBTokenImplementation(
    client,
    contractHashFactory,
    contractAddressFactory,
    codeIdToken,
    contractHashToken
  );
  await sleep();

  // await queryLBTokenImplementation(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory
  // );

  // const bin_step: number = 100;
  // const base_factor: number = 5000;
  // const filter_period = 30;
  // const decay_period = 600;
  // const reduction_factor = 5000;
  // const variable_fee_control = 40000;
  // const protocol_share = 1000;
  // const max_volatility_accumulator = 350000;
  // const is_open = true;

  // await executeSetPreset(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory,
  //   bin_step,
  //   base_factor,
  //   filter_period,
  //   decay_period,
  //   reduction_factor,
  //   variable_fee_control,
  //   protocol_share,
  //   max_volatility_accumulator,
  //   is_open
  // );
  // await sleep();

  // await executeSetPreset(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory,
  //   50,
  //   base_factor,
  //   filter_period,
  //   decay_period,
  //   reduction_factor,
  //   variable_fee_control,
  //   protocol_share,
  //   max_volatility_accumulator,
  //   is_open
  // );
  // await sleep();

  // await executeSetPreset(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory,
  //   25,
  //   base_factor,
  //   filter_period,
  //   decay_period,
  //   reduction_factor,
  //   variable_fee_control,
  //   protocol_share,
  //   max_volatility_accumulator,
  //   is_open
  // );
  // await sleep();

  // // TOKENY
  // await executeAddQuoteAsset(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory,
  //   tokenY.custom_token.token_code_hash,
  //   tokenY.custom_token.contract_addr
  // );
  // await sleep();

  // await queryPreset(client, contractHashFactory, contractAddressFactory);

  // const active_id = 8388608;

  // await executeCreateLBPairUsingRouter(
  //   client,
  //   contractHashRouter,
  //   contractAddressRouter,
  //   tokenX.custom_token.token_code_hash,
  //   tokenX.custom_token.contract_addr,
  //   tokenY.custom_token.token_code_hash,
  //   tokenY.custom_token.contract_addr,
  //   active_id,
  //   bin_step
  // );
  // await sleep();
}

// async function test_liquidity(
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
//   // const sSCRT: CustomToken = {
//   //   custom_token: {
//   //     contract_addr: "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg",
//   //     token_code_hash: "9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813"
//   //   }
//   // }

//   // const SILK: CustomToken = {
//   //   custom_token: {
//   //     contract_addr: "secret16xz08fdtkp5m8m6arpfgnehlfl4t86l0p33xg0",
//   //     token_code_hash: "b6c896d21e46e037a2a1bca1d55af262d7ae4a5a175af055f3939722626b30c3"
//   //   }
//   // }

//   // TODO: a better way to get and keep the lb_pair contract address
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

//   logToFile(`LB_PAIR_ADDRESS="${contractAddressPair}"`);

//   // increase allowance for Token X
//   let tx = await client.tx.snip20.increaseAllowance(
//     {
//       sender: client.address,
//       contract_address: tokenX.custom_token.contract_addr,
//       code_hash: tokenX.custom_token.token_code_hash,
//       msg: {
//         increase_allowance: {
//           spender: contractAddressPair,
//           amount: "340282366920938463463374607431768211454",
//         },
//       },
//     },
//     {
//       gasLimit: 200_000,
//     }
//   );

//   if (tx.code !== 0) {
//     throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
//   }

//   console.log(`Increase Token X Allowance TX used ${tx.gasUsed} gas`);

//   await sleep();

//   // increase allowance for Token Y
//   let tx2 = await client.tx.snip20.increaseAllowance(
//     {
//       sender: client.address,
//       contract_address: tokenY.custom_token.contract_addr,
//       code_hash: tokenY.custom_token.token_code_hash,
//       msg: {
//         increase_allowance: {
//           spender: contractAddressPair,
//           amount: "340282366920938463463374607431768211454",
//         },
//       },
//     },
//     {
//       gasLimit: 200_000,
//     }
//   );

//   if (tx2.code !== 0) {
//     throw new Error(`Failed with the following error:\n ${tx2.rawLog}`);
//   }

//   console.log(`Increase Token Y Allowance TX used ${tx2.gasUsed} gas`);

//   const bin_step = 100;
//   await executeAddLiquidity(
//     client,
//     contractHashPair,
//     contractAddressPair,
//     bin_step,
//     tokenX,
//     tokenY
//   );
//   await sleep();
//   await queryTotalSupply(client, contractHashPair, contractAddressPair).catch(
//     (e) => console.log(e)
//   );
//   await sleep();

//   await executeRemoveLiquidity(
//     client,
//     contractHashPair,
//     contractAddressPair,
//     bin_step,
//     tokenX,
//     tokenY
//   );
//   await sleep();
// }

// async function test_swaps(
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
//   // TODO: a better way to get and keep the lb_pair contract address
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

//   const swapAmount = "10000000000";

//   await executeSwap(client, contractHashPair, contractAddressPair, swapAmount);
//   await sleep();
// }

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

(async () => {
  const currentTime = new Date();
  const currentTimeString = currentTime.toTimeString();
  logToFile(`Deploy Time: ${currentTimeString}`);

  const [
    client,
    contractHashFactory,
    contractAddressFactory,
    contractHashRouter,
    contractAddressRouter,
    codeIdPair,
    contractHashPair,
    codeIdToken,
    contractHashToken,
    tokenX,
    tokenY,
  ] = await initializeAndUploadContract();

  sleep();

  await runTestFunction(
    test_configure_factory,
    client,
    contractHashFactory,
    contractAddressFactory,
    contractHashRouter,
    contractAddressRouter,
    codeIdPair,
    contractHashPair,
    codeIdToken,
    contractHashToken,
    tokenX,
    tokenY
  );

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

  // await runTestFunction(
  //   test_pair_queries,
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

  // await runTestFunction(
  //   test_swaps,
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

  // await runTestFunction(
  //   test_token_queries,
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
  //   tokenY,
  // );

  logToFile(`\n\n\n`);
  logStream.end();
  gasStream.end();
})();
