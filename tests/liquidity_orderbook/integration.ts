import axios from "axios";
import { Wallet, SecretNetworkClient } from "secretjs";
import fs from "fs";
import assert from "assert";
import {
  initializeFactoryContract,
  executeSetLBPairImplementation,
  executeCreateLBPair,
  executeSetPreset,
  executeAddQuoteAsset,
  queryLBPairImplementation,
  executeSetLBTokenImplementation,
  queryLBTokenImplementation,
  queryLBPairInformation,
  queryPreset,
  TokenType,
  CustomToken,
  NativeToken,
} from "./lb_factory";
import {
  ContractInfo,
  initializePairContract,
  queryReserves,
  queryActiveId,
  queryIdfromPrice,
  queryPriceFromId,
  querySwapIn,
  querySwapOut,
  queryVariableFeeParameters,
  queryStaticFeeParameters,
  queryOracleParameters,
  executeAddLiquidity,
  executeRemoveLiquidity,
  executeSwap,
  queryTotalSupply,
} from "./lb_pair";
import {
  initializeTokenContract,
  queryName,
  querySymbol,
  queryDecimals,
} from "./lb_token";
import {
  initializeRouterContract,
  executeCreateLBPairUsingRouter,
  executeSwapTokensForExact,
} from "./lb_router";
import dotenv from "dotenv";

dotenv.config({ path: ".env" });

const build = "./tests/wasm/";

// This helps when deploying to Pulsar. It can be shortened to test on secretdev.
export const sleep = () => new Promise((resolve) => setTimeout(resolve, 100));

var mnemonic: string;
var endpoint: string = "http://localhost:1317";
var chainId: string = "secretdev-1";

// Uncomment to use .env file to deploy to Pulsar:
// mnemonic = process.env.MNEMONIC!;
// endpoint = process.env.LCD_URL!;
// chainId = process.env.CHAIN_ID!;

// Create a write stream to the desired file
const logStream = fs.createWriteStream("localcontracts.log", { flags: "a" });
const gasStream = fs.createWriteStream("gas.log", { flags: "w" });

// Custom logging functions
export function logToFile(message: string) {
  logStream.write(message + "\n");
}

export function logGasToFile(message: string) {
  gasStream.write(message + "\n");
}

// Returns a client with which we can interact with secret network
const initializeClient = async (endpoint: string, chainId: string) => {
  let wallet: Wallet;
  if (mnemonic) {
    wallet = new Wallet(mnemonic);
  } else {
    wallet = new Wallet();
  }
  const accAddress = wallet.address;
  const client = new SecretNetworkClient({
    // Create a client to interact with the network
    url: endpoint,
    chainId: chainId,
    wallet: wallet,
    walletAddress: accAddress,
  });

  console.log(`Initialized client with wallet address: ${accAddress}`);
  return client;
};

const getFromFaucet = async (address: string) => {
  await axios.get(`http://localhost:5000/faucet?address=${address}`);
};

async function getScrtBalance(userCli: SecretNetworkClient): Promise<string> {
  let balanceResponse = await userCli.query.bank.balance({
    address: userCli.address,
    denom: "uscrt",
  });

  if (balanceResponse?.balance?.amount === undefined) {
    throw new Error(`Failed to get balance for address: ${userCli.address}`);
  }

  return balanceResponse.balance.amount;
}

async function fillUpFromFaucet(
  client: SecretNetworkClient,
  targetBalance: Number,
) {
  let balance = await getScrtBalance(client);
  while (Number(balance) < targetBalance.valueOf()) {
    try {
      await getFromFaucet(client.address);
    } catch (e) {
      console.error(`failed to get tokens from faucet: ${e}`);
    }
    balance = await getScrtBalance(client);
  }
  console.error(`got tokens from faucet: ${balance}`);
}

const initializeSnip20Contract = async (
  client: SecretNetworkClient,
  contractPath: string,
) => {
  const wasmCode = fs.readFileSync(contractPath);
  console.log("\nUploading contract");

  const uploadReceipt = await client.tx.compute.storeCode(
    {
      wasm_byte_code: wasmCode,
      sender: client.address,
      source: "",
      builder: "enigmampc/secret-contract-optimizer:1.0.9",
    },
    {
      gasLimit: 4_000_000,
    },
  );

  if (uploadReceipt.code !== 0) {
    console.log(
      `Failed to get code id: ${JSON.stringify(uploadReceipt.rawLog)}`,
    );
    throw new Error(`Failed to upload contract`);
  }

  const codeIdKv = uploadReceipt.jsonLog![0].events[0].attributes.find(
    (a: any) => {
      return a.key === "code_id";
    },
  );

  console.log(`Upload used ${uploadReceipt.gasUsed} gas`);

  const codeId = Number(codeIdKv!.value);
  console.log("snip20 Contract codeId: ", codeId);
  logToFile(`SNIP20_CODE_ID="${codeId}"`);

  const contractCodeHash = (
    await client.query.compute.codeHashByCodeId({ code_id: String(codeId) })
  ).code_hash;

  if (contractCodeHash === undefined) {
    throw new Error(`Failed to get code hash`);
  }

  console.log(`snip20 Contract hash: ${contractCodeHash}`);
  logToFile(`SNIP20_CODE_HASH="${contractCodeHash}"`);

  // first token contract

  const init_msg_x = {
    name: "token x",
    admin: client.address,
    symbol: "TOKENX",
    decimals: 6,
    initial_balances: [
      { address: client.address, amount: "1000000000000000000000" },
    ],
    prng_seed: Buffer.from("kent rocks").toString("base64"),
    config: { public_total_supply: true },
  };

  const contractX = await client.tx.compute.instantiateContract(
    {
      sender: client.address,
      code_id: codeId,
      init_msg: init_msg_x,
      code_hash: contractCodeHash,
      label: "token_x SNIP20" + Math.ceil(Math.random() * 50000), // The label should be unique for every contract, add random string in order to maintain uniqueness
    },
    {
      gasLimit: 5000000,
    },
  );

  if (contractX.code !== 0) {
    throw new Error(
      `Failed to instantiate the contract with the following error ${contractX.rawLog}`,
    );
  }

  const contractAddressX = contractX.arrayLog!.find(
    (log) => log.type === "message" && log.key === "contract_address",
  )!.value;

  console.log(`tokenX Contract address: ${contractAddressX}`);
  logToFile(`TOKENX_SYMBOL="${init_msg_x.symbol}"`);
  logToFile(`TOKENX_CONTRACT_ADDRESS="${contractAddressX}"`);
  logToFile(`TOKENX_CODE_HASH="${contractCodeHash}"`);

  await sleep();

  // second token contract

  const init_msg_y = {
    name: "token y",
    admin: client.address,
    symbol: "TOKENY",
    decimals: 6,
    initial_balances: [
      { address: client.address, amount: "1000000000000000000000" },
    ],
    prng_seed: Buffer.from("haseeb rocks").toString("base64"),
    config: { public_total_supply: true },
  };

  const contractY = await client.tx.compute.instantiateContract(
    {
      sender: client.address,
      code_id: codeId,
      init_msg: init_msg_y,
      code_hash: contractCodeHash,
      label: "token_y SNIP20" + Math.ceil(Math.random() * 50000), // The label should be unique for every contract, add random string in order to maintain uniqueness
    },
    {
      gasLimit: 5000000,
    },
  );

  if (contractY.code !== 0) {
    throw new Error(
      `Failed to instantiate the contract with the following error ${contractY.rawLog}`,
    );
  }

  const contractAddressY = contractY.arrayLog!.find(
    (log) => log.type === "message" && log.key === "contract_address",
  )!.value;

  console.log(`tokenY Contract address: ${contractAddressY}`);
  logToFile(`TOKENY_SYMBOL="${init_msg_x.symbol}"`);
  logToFile(`TOKENY_CONTRACT_ADDRESS="${contractAddressY}"`);
  logToFile(`TOKENY_CODE_HASH="${contractCodeHash}"`);

  await sleep();

  var contractInfo: [CustomToken, CustomToken] = [
    {
      custom_token: {
        contract_addr: contractAddressX,
        token_code_hash: contractCodeHash,
      },
    },
    {
      custom_token: {
        contract_addr: contractAddressY,
        token_code_hash: contractCodeHash,
      },
    },
  ];
  return contractInfo;
};

// Initialization procedure
async function initializeAndUploadContract() {
  const client = await initializeClient(endpoint, chainId);

  if (chainId == "secretdev-1") {
    await fillUpFromFaucet(client, 100_000_000);
  }

  const [tokenX, tokenY] = await initializeSnip20Contract(
    client,
    build + "snip25.wasm",
  );
  await sleep();

  const [contractHashFactory, contractAddressFactory] =
    await initializeFactoryContract(client, build + "lb_factory.wasm");
  await sleep();

  const [contractHashRouter, contractAddressRouter] =
    await initializeRouterContract(
      client,
      build + "lb_router.wasm",
      contractHashFactory,
      contractAddressFactory,
    );
  await sleep();

  const [codeIdPair, contractHashPair] = await initializePairContract(
    client,
    build + "lb_pair.wasm",
  );
  await sleep();

  const [codeIdToken, contractHashToken] = await initializeTokenContract(
    client,
    build + "lb_token.wasm",
  );
  await sleep();

  var clientInfo: [
    SecretNetworkClient,
    string,
    string,
    string,
    string,
    number,
    string,
    number,
    string,
    CustomToken,
    CustomToken,
  ] = [
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
    ];
  return clientInfo;
}

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
  tokenX: CustomToken,
  tokenY: CustomToken,
) {
  await executeSetLBPairImplementation(
    client,
    contractHashFactory,
    contractAddressFactory,
    codeIdPair,
    contractHashPair,
  );
  await sleep();

  await queryLBPairImplementation(
    client,
    contractHashFactory,
    contractAddressFactory,
  );

  await executeSetLBTokenImplementation(
    client,
    contractHashFactory,
    contractAddressFactory,
    codeIdToken,
    contractHashToken,
  );
  await sleep();

  await queryLBTokenImplementation(
    client,
    contractHashFactory,
    contractAddressFactory,
  );

  const bin_step: number = 100;
  const base_factor: number = 5000;
  const filter_period = 30;
  const decay_period = 600;
  const reduction_factor = 5000;
  const variable_fee_control = 40000;
  const protocol_share = 1000;
  const max_volatility_accumulator = 350000;
  const is_open = true;

  await executeSetPreset(
    client,
    contractHashFactory,
    contractAddressFactory,
    bin_step,
    base_factor,
    filter_period,
    decay_period,
    reduction_factor,
    variable_fee_control,
    protocol_share,
    max_volatility_accumulator,
    is_open,
  );
  await sleep();

  await executeSetPreset(
    client,
    contractHashFactory,
    contractAddressFactory,
    50,
    base_factor,
    filter_period,
    decay_period,
    reduction_factor,
    variable_fee_control,
    protocol_share,
    max_volatility_accumulator,
    is_open,
  );
  await sleep();

  await executeSetPreset(
    client,
    contractHashFactory,
    contractAddressFactory,
    25,
    base_factor,
    filter_period,
    decay_period,
    reduction_factor,
    variable_fee_control,
    protocol_share,
    max_volatility_accumulator,
    is_open,
  );
  await sleep();

  // TOKENY
  await executeAddQuoteAsset(
    client,
    contractHashFactory,
    contractAddressFactory,
    tokenY.custom_token.token_code_hash,
    tokenY.custom_token.contract_addr,
  );
  await sleep();

  // testnet sSCRT
  // await executeAddQuoteAsset(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory,
  //   "9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813",
  //   "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg",
  // );
  // await sleep();

  // testnet stkd-SCRT
  // await executeAddQuoteAsset(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory,
  //   "680fbb3c8f8eb1c920da13d857daaedaa46ab8f9a8e26e892bb18a16985ec29e",
  //   "secret10u3rwj0cc2r04lryaxtkucjhvqw63kqzm5jlxw",
  // );
  // await sleep();

  // testnet SILK
  // await executeAddQuoteAsset(
  //   client,
  //   contractHashFactory,
  //   contractAddressFactory,
  //   "b6c896d21e46e037a2a1bca1d55af262d7ae4a5a175af055f3939722626b30c3",
  //   "secret16xz08fdtkp5m8m6arpfgnehlfl4t86l0p33xg0",
  // );
  // await sleep();

  await queryPreset(client, contractHashFactory, contractAddressFactory);

  const active_id = 8388608;

  await executeCreateLBPairUsingRouter(
    client,
    contractHashRouter,
    contractAddressRouter,
    tokenX.custom_token.token_code_hash,
    tokenX.custom_token.contract_addr,
    tokenY.custom_token.token_code_hash,
    tokenY.custom_token.contract_addr,
    active_id,
    bin_step,
  );
  await sleep();
}

async function test_liquidity(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  contractHashRouter: string,
  contractAddressRouter: string,
  codeIdPair: number,
  contractHashPair: string,
  codeIdToken: number,
  contractHashToken: string,
  tokenX: CustomToken,
  tokenY: CustomToken,
) {
  // const sSCRT: CustomToken = {
  //   custom_token: {
  //     contract_addr: "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg",
  //     token_code_hash: "9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813"
  //   }
  // }

  // const SILK: CustomToken = {
  //   custom_token: {
  //     contract_addr: "secret16xz08fdtkp5m8m6arpfgnehlfl4t86l0p33xg0",
  //     token_code_hash: "b6c896d21e46e037a2a1bca1d55af262d7ae4a5a175af055f3939722626b30c3"
  //   }
  // }

  // TODO: a better way to get and keep the lb_pair contract address
  const {
    lb_pair_information: {
      lb_pair: {
        contract: { address: contractAddressPair },
      },
    },
  } = await queryLBPairInformation(
    client,
    contractHashFactory,
    contractAddressFactory,
    tokenX,
    tokenY,
    100,
  );

  logToFile(`LB_PAIR_ADDRESS="${contractAddressPair}"`);

  // increase allowance for Token X
  let tx = await client.tx.snip20.increaseAllowance(
    {
      sender: client.address,
      contract_address: tokenX.custom_token.contract_addr,
      code_hash: tokenX.custom_token.token_code_hash,
      msg: {
        increase_allowance: {
          spender: contractAddressPair,
          amount: "340282366920938463463374607431768211454",
        },
      },
    },
    {
      gasLimit: 200_000,
    },
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  console.log(`Increase Token X Allowance TX used ${tx.gasUsed} gas`);

  await sleep();

  // increase allowance for Token Y
  let tx2 = await client.tx.snip20.increaseAllowance(
    {
      sender: client.address,
      contract_address: tokenY.custom_token.contract_addr,
      code_hash: tokenY.custom_token.token_code_hash,
      msg: {
        increase_allowance: {
          spender: contractAddressPair,
          amount: "340282366920938463463374607431768211454",
        },
      },
    },
    {
      gasLimit: 200_000,
    },
  );

  if (tx2.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx2.rawLog}`);
  }

  console.log(`Increase Token Y Allowance TX used ${tx2.gasUsed} gas`);

  const bin_step = 100;
  await executeAddLiquidity(
    client,
    contractHashPair,
    contractAddressPair,
    bin_step,
    tokenX,
    tokenY,
  );
  await sleep();
  await queryTotalSupply(client, contractHashPair, contractAddressPair).catch(
    (e) => console.log(e),
  );
  await sleep();

  await executeRemoveLiquidity(
    client,
    contractHashPair,
    contractAddressPair,
    bin_step,
    tokenX,
    tokenY,
  );
  await sleep();
}

async function test_swaps(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  contractHashRouter: string,
  contractAddressRouter: string,
  codeIdPair: number,
  contractHashPair: string,
  codeIdToken: number,
  contractHashToken: string,
  tokenX: CustomToken,
  tokenY: CustomToken,
) {
  // TODO: a better way to get and keep the lb_pair contract address
  const {
    lb_pair_information: {
      lb_pair: {
        contract: { address: contractAddressPair },
      },
    },
  } = await queryLBPairInformation(
    client,
    contractHashFactory,
    contractAddressFactory,
    tokenX,
    tokenY,
    100,
  );

  const swapAmount = "10000000000";

  await executeSwap(client, contractHashPair, contractAddressPair, swapAmount);
  await sleep();
}

async function test_pair_queries(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  contractHashRouter: string,
  contractAddressRouter: string,
  codeIdPair: number,
  contractHashPair: string,
  codeIdToken: number,
  contractHashToken: string,
  tokenX: CustomToken,
  tokenY: CustomToken,
) {
  // TODO: query factory for a pair address after it's created
  const {
    lb_pair_information: {
      lb_pair: {
        contract: { address: contractAddressPair },
      },
    },
  } = await queryLBPairInformation(
    client,
    contractHashFactory,
    contractAddressFactory,
    tokenX,
    tokenY,
    100,
  );

  const { reserve_x, reserve_y } = await queryReserves(
    client,
    contractHashPair,
    contractAddressPair,
  );
  await queryActiveId(client, contractHashPair, contractAddressPair).catch(
    (e) => console.log(e),
  );

  await queryIdfromPrice(client, contractHashPair, contractAddressPair).catch(
    (e) => console.log(e),
  );

  await queryPriceFromId(client, contractHashPair, contractAddressPair).catch(
    (e) => console.log(e),
  );

  await queryStaticFeeParameters(
    client,
    contractHashPair,
    contractAddressPair,
  ).catch((e) => console.log(e));

  await queryVariableFeeParameters(
    client,
    contractHashPair,
    contractAddressPair,
  ).catch((e) => console.log(e));

  await queryOracleParameters(
    client,
    contractHashPair,
    contractAddressPair,
  ).catch((e) => console.log(e));

  await querySwapIn(client, contractHashPair, contractAddressPair).catch((e) =>
    console.log(e),
  );

  await querySwapOut(client, contractHashPair, contractAddressPair).catch((e) =>
    console.log(e),
  );
}

async function test_gas_limits() {
  // There is no accurate way to measue gas limits but it is actually very recommended to make sure that the gas that is used by a specific tx makes sense
}

async function runTestFunction(
  tester: (
    // TODO: combine all these into a single object or something
    client: SecretNetworkClient,
    contractHashFactory: string,
    contractAddressFactory: string,
    contractHashRouter: string,
    contractAddressRouter: string,
    codeIdPair: number,
    contractHashPair: string,
    codeIdToken: number,
    contractHashToken: string,
    tokenX: CustomToken,
    tokenY: CustomToken,
  ) => void,
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  contractHashRouter: string,
  contractAddressRouter: string,
  codeIdPair: number,
  contractHashPair: string,
  codeIdToken: number,
  contractHashToken: string,
  tokenX: CustomToken,
  tokenY: CustomToken,
) {
  console.log(`Testing ${tester.name}`);
  await tester(
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
  );
  console.log(`[\x1b[32m SUCCESS \x1b[0m] ${tester.name}`);
}

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
    tokenY,
  );

  await runTestFunction(
    test_liquidity,
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
  );

  await runTestFunction(
    test_pair_queries,
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
  );

  await runTestFunction(
    test_swaps,
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
  );

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
