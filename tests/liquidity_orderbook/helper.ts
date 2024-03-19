import axios from "axios";
import dotenv from "dotenv";
import fs from "fs";
import path from "path";
import { SecretNetworkClient, Wallet } from "secretjs";
import { SetViewingKeyOptions } from "secretjs/dist/extensions/access_control/viewing_key/msgs";
import { initializeAdminAuth } from "./admin_auth/instantiate";
import {
  TokenType,
  executeAddQuoteAsset,
  executeCreateLBPair,
  executeSetLBPairImplementation,
  executeSetLBStakingImplementation,
  executeSetLBTokenImplementation,
  executeSetPreset,
  initializeFactoryContract,
} from "./lb_factory";
import {
  queryLBPairImplementation,
  queryLBPairInformation,
  queryLBTokenImplementation,
  queryPreset,
} from "./lb_factory/query";
import { initializePairContract as uploadPairContract } from "./lb_pair";
import { queryLbStaking, queryLbToken } from "./lb_pair/query";
import { initializeRouterContract } from "./lb_router/instantiate";
import { uploadStakingContract } from "./lb_staking/instantiate";
import { setViewingKey } from "./lb_token/execute";
import { uploadTokenContract } from "./lb_token/instantiate";

dotenv.config({ path: ".env" });

const build = "./wasm/";
// const build_direct_to_target = "../../target/wasm32-unknown-unknown/release/";
const build_direct_to_target = "./wasm/";

// This helps when deploying to Pulsar. It can be shortened to test on secretdev.
export const sleep = () => new Promise((resolve) => setTimeout(resolve, 10));
export const sleeplonger = () =>
  new Promise((resolve) => setTimeout(resolve, 10000));

var mnemonic: string;
var endpoint: string = "http://localhost:1317";
var chainId: string = "secretdev-1";

// Uncomment to use .env file to deploy to Pulsar:
// mnemonic = process.env.MNEMONIC!;
// endpoint = process.env.LCD_URL!;
// chainId = process.env.CHAIN_ID!;

// Create a write stream to the desired file
export const logStream = fs.createWriteStream("localcontracts.log", {
  flags: "a",
});

export const gasStream = fs.createWriteStream("gas.log", { flags: "w" });

// Custom logging functions
export function logToFile(message: string) {
  logStream.write(message + "\n");
}

export function logGasToFile(message: string) {
  gasStream.write(message + "\n");
}

export const initializeClient = async (endpoint: string, chainId: string) => {
  let wallet: Wallet;

  const contractFilePath = path.join(__dirname, "contract_address_log.json");
  const contractsData = readContractAddresses(contractFilePath);

  if (mnemonic) {
    wallet = new Wallet(mnemonic);
  } else if (contractsData["Mnemonic"]) {
    let m = contractsData["Mnemonic"];
    wallet = new Wallet(m);
  } else {
    wallet = new Wallet();
    contractsData["Mnemonic"] = wallet.mnemonic;
    writeContractAddresses(contractFilePath, contractsData);
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

export const getFromFaucet = async (address: string) => {
  await axios.get(`http://localhost:5000/faucet?address=${address}`);
};

export async function getScrtBalance(
  userCli: SecretNetworkClient
): Promise<string> {
  let balanceResponse = await userCli.query.bank.balance({
    address: userCli.address,
    denom: "uscrt",
  });

  if (balanceResponse?.balance?.amount === undefined) {
    throw new Error(`Failed to get balance for address: ${userCli.address}`);
  }

  return balanceResponse.balance.amount;
}

export async function fillUpFromFaucet(
  client: SecretNetworkClient,
  targetBalance: Number
) {
  let balance = await getScrtBalance(client);
  console.log(`User Balance: ${balance}`);
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

export const initializeSnip20Contract = async (
  client: SecretNetworkClient,
  contractPath: string
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
    }
  );

  if (uploadReceipt.code !== 0) {
    console.log(
      `Failed to get code id: ${JSON.stringify(uploadReceipt.rawLog)}`
    );
    throw new Error(`Failed to upload contract`);
  }

  const codeIdKv = uploadReceipt.jsonLog![0].events[0].attributes.find(
    (a: any) => {
      return a.key === "code_id";
    }
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
      {
        address: client.address,
        amount: "340282366920938463463374607431768211454",
      },
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
    }
  );

  if (contractX.code !== 0) {
    throw new Error(
      `Failed to instantiate the contract with the following error ${contractX.rawLog}`
    );
  }

  const contractAddressX = contractX.arrayLog!.find(
    (log) => log.type === "message" && log.key === "contract_address"
  )!.value;

  console.log(`tokenX Contract address: ${contractAddressX}`);
  logToFile(`TOKENX_SYMBOL="${init_msg_x.symbol}"`);
  logToFile(`TOKENX_CONTRACT_ADDRESS="${contractAddressX}"`);
  logToFile(`TOKENX_CODE_HASH="${contractCodeHash}"`);

  await sleep();

  let set_vk_msg: SetViewingKeyOptions = {
    set_viewing_key: {
      key: "viewing_key",
    },
  };

  let res1 = await client.tx.snip20.setViewingKey(
    {
      sender: client.address,
      contract_address: contractAddressX,
      code_hash: contractCodeHash,
      msg: set_vk_msg,
    },
    {
      gasLimit: 5000000,
    }
  );

  if (res1.code !== 0) {
    throw new Error(
      `Failed to instantiate the contract with the following error ${res1.rawLog}`
    );
  }

  // second token contract

  const init_msg_y = {
    name: "token y",
    admin: client.address,
    symbol: "TOKENY",
    decimals: 6,
    initial_balances: [
      {
        address: client.address,
        amount: "340282366920938463463374607431768211454",
      },
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
    }
  );

  if (contractY.code !== 0) {
    throw new Error(
      `Failed to instantiate the contract with the following error ${contractY.rawLog}`
    );
  }

  const contractAddressY = contractY.arrayLog!.find(
    (log) => log.type === "message" && log.key === "contract_address"
  )!.value;

  console.log(`tokenY Contract address: ${contractAddressY}`);
  logToFile(`TOKENY_SYMBOL="${init_msg_x.symbol}"`);
  logToFile(`TOKENY_CONTRACT_ADDRESS="${contractAddressY}"`);
  logToFile(`TOKENY_CODE_HASH="${contractCodeHash}"`);

  await sleep();
  let res = await client.tx.snip20.setViewingKey(
    {
      sender: client.address,
      contract_address: contractAddressY,
      code_hash: contractCodeHash,
      msg: set_vk_msg,
    },
    {
      gasLimit: 5000000,
    }
  );
  if (res.code !== 0) {
    throw new Error(
      `Failed to instantiate the contract with the following error ${res.rawLog}`
    );
  }
  await sleep();

  var contractInfo: TokenType[] = [
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

// export interface SetViewingKeyOptions {
//   set_viewing_key: {
//     key: string;
//     padding?: string;
//   };
// }

// // Initialization procedure
// export async function initializeAndUploadContract() {
//   const client = await initializeClient(endpoint, chainId);

//   if (chainId == "secretdev-1") {
//     await fillUpFromFaucet(client, 100_000_000);
//   }

//   // const [tokenX, tokenY] = await initializeSnip20Contract(
//   //   client,
//   //   build + "snip25.wasm"
//   // );
//   // await sleep();

//   // const [contractHashFactory, contractAddressFactory] =
//   //   await initializeFactoryContract(
//   //     client,
//   //     build + "lb_factory.wasm",
//   //     client.address,
//   //     ""
//   //   );
//   // await sleep();

//   // const [contractHashRouter, contractAddressRouter] =
//   //   await initializeRouterContract(
//   //     client,
//   //     build + "router.wasm",
//   //     contractHashFactory,
//   //     contractAddressFactory
//   //   );
//   // await sleep();

//   const [codeIdPair, contractHashPair] = await initializePairContract(
//     client,
//     build + "lb_pair.wasm",
//     "",
//     ""
//   );
//   await sleep();

//   // const [codeIdToken, contractHashToken] = await initializeTokenContract(
//   //   client,
//   //   build + "lb_token.wasm"
//   // );
//   // await sleep();
// }

// Function to read contract addresses from the file
function readContractAddresses(filePath: string) {
  if (fs.existsSync(filePath)) {
    const fileContent = fs.readFileSync(filePath, "utf8");
    return JSON.parse(fileContent);
  }
  return {};
}

// Function to write contract addresses to the file
function writeContractAddresses(filePath: fs.PathOrFileDescriptor, data: any) {
  fs.writeFileSync(filePath, JSON.stringify(data, null, 2), "utf8");
}

// Function to check if a contract is already initialized
function isContractInitialized(
  data: { [x: string]: { hash: any } },
  contractName: string
) {
  return data[contractName];
}

// Your initialization procedure, modified to use the above functions
export async function initializeAndUploadContract() {
  const client = await initializeClient(endpoint, chainId);
  const contractFileReadPath = path.join(
    __dirname,
    "contract_address_temp_log.json"
  );
  const contractFilePath = path.join(__dirname, "contract_address_log.json");

  const contractsData = readContractAddresses(contractFileReadPath);

  if (chainId == "secretdev-1") {
    await fillUpFromFaucet(client, 100_000_000);
  }

  let tokenX: any,
    tokenY: any,
    contractHashAdminAuth: any,
    contractAddressAdminAuth: any,
    contractHashFactory: any,
    contractAddressFactory: any,
    contractHashRouter: any,
    contractAddressRouter: any,
    codeIdPair: any,
    contractHashPair: any,
    codeIdToken: any,
    contractHashToken: any,
    codeIdStaking: any,
    contractHashStaking: any;

  if (
    !isContractInitialized(
      contractsData,
      "TokenX" || !isContractInitialized(contractsData, "TokenY")
    )
  ) {
    [tokenX, tokenY] = await initializeSnip20Contract(
      client,
      build + "snip25.wasm"
    );
    contractsData["TokenX"] = tokenX;
    contractsData["TokenY"] = tokenY;

    writeContractAddresses(contractFilePath, contractsData);
    await sleep();
  } else {
    tokenX = contractsData["TokenX"];
    tokenY = contractsData["TokenY"];
  }

  // Assuming the contractsData object and other required variables are already defined

  // Factory Contract
  if (!isContractInitialized(contractsData, "AdminAuth")) {
    [contractHashAdminAuth, contractAddressAdminAuth] =
      await initializeAdminAuth(
        client,
        build + "admin.wasm",
        client.address,
        ""
      );
    contractsData["AdminAuth"] = {
      address: contractAddressFactory,
      hash: contractHashFactory,
    };
    writeContractAddresses(contractFilePath, contractsData);
    await sleep();
  } else {
    contractHashFactory = contractsData["AdminAuth"].hash;
  }

  // Factory Contract
  if (!isContractInitialized(contractsData, "LBFactory")) {
    [contractHashFactory, contractAddressFactory] =
      await initializeFactoryContract(
        client,
        build_direct_to_target + "lb_factory.wasm",
        contractAddressAdminAuth,
        contractHashAdminAuth
      );
    contractsData["LBFactory"] = {
      address: contractAddressFactory,
      hash: contractHashFactory,
    };
    writeContractAddresses(contractFilePath, contractsData);
    await sleep();
  } else {
    contractHashFactory = contractsData["LBFactory"].hash;
    contractAddressFactory = contractsData["LBFactory"].address;
  }

  // Router Contract
  if (!isContractInitialized(contractsData, "Router")) {
    [contractHashRouter, contractAddressRouter] =
      await initializeRouterContract(
        client,
        build_direct_to_target + "router.wasm",
        contractHashFactory,
        contractAddressFactory
      );
    contractsData["Router"] = {
      address: contractAddressRouter,
      hash: contractHashRouter,
    };
    writeContractAddresses(contractFilePath, contractsData);
    await sleep();
  } else {
    contractHashRouter = contractsData["Router"].hash;
    contractAddressRouter = contractsData["Router"].address;
  }

  // Pair Contract
  if (!isContractInitialized(contractsData, "LBPair")) {
    [codeIdPair, contractHashPair] = await uploadPairContract(
      client,
      build_direct_to_target + "lb_pair.wasm"
    );
    contractsData["LBPair"] = {
      codeId: codeIdPair,
      hash: contractHashPair,
    };
    writeContractAddresses(contractFilePath, contractsData);
    await sleep();
  } else {
    codeIdPair = contractsData["LBPair"].codeId;
    contractHashPair = contractsData["LBPair"].hash;
  }

  // Token Contract
  if (!isContractInitialized(contractsData, "LBToken")) {
    [codeIdToken, contractHashToken] = await uploadTokenContract(
      client,
      build_direct_to_target + "lb_token.wasm"
    );
    contractsData["LBToken"] = {
      codeId: codeIdToken,
      hash: contractHashToken,
    };
    writeContractAddresses(contractFilePath, contractsData);
    await sleep();
  } else {
    codeIdToken = contractsData["LBToken"].codeId;
    contractHashToken = contractsData["LBToken"].hash;
  }

  // Staking Contract
  if (!isContractInitialized(contractsData, "LBStaking")) {
    [codeIdStaking, contractHashStaking] = await uploadStakingContract(
      client,
      build_direct_to_target + "lb_staking.wasm"
    );
    contractsData["LBStaking"] = {
      codeId: codeIdToken,
      hash: contractHashToken,
    };
    writeContractAddresses(contractFilePath, contractsData);
    await sleep();
  } else {
    codeIdStaking = contractsData["LBStaking"].codeId;
    contractHashStaking = contractsData["LBStaking"].hash;
  }

  var clientInfo = [
    client,
    contractHashFactory,
    contractAddressFactory,
    contractHashRouter,
    contractAddressRouter,
    codeIdPair,
    contractHashPair,
    codeIdToken,
    contractHashToken,
    codeIdStaking,
    contractHashStaking,
    tokenX,
    tokenY,
  ];

  let info: clientInfo = {
    client,
    contractHashFactory,
    contractAddressFactory,
    contractHashRouter,
    contractAddressRouter,
    codeIdPair,
    contractHashPair,
    codeIdToken,
    contractHashToken,
    codeIdStaking,
    contractHashStaking,
    tokenX,
    tokenY,
    contractAddressPair: "",
    contractAddressToken: "",
    contractAddressStaking: "",
  };

  return info;
}

export async function test_configure_factory(clientInfo: clientInfo) {
  await executeSetLBPairImplementation(
    clientInfo.client,
    clientInfo.contractHashFactory,
    clientInfo.contractAddressFactory,
    clientInfo.codeIdPair,
    clientInfo.contractHashPair
  );
  await sleep();

  await queryLBPairImplementation(
    clientInfo.client,
    clientInfo.contractHashFactory,
    clientInfo.contractAddressFactory
  );

  await executeSetLBTokenImplementation(
    clientInfo.client,
    clientInfo.contractHashFactory,
    clientInfo.contractAddressFactory,
    clientInfo.codeIdToken,
    clientInfo.contractHashToken
  );
  await sleep();

  await executeSetLBStakingImplementation(
    clientInfo.client,
    clientInfo.contractHashFactory,
    clientInfo.contractAddressFactory,
    clientInfo.codeIdStaking,
    clientInfo.contractHashStaking
  );
  await sleep();

  await queryLBTokenImplementation(
    clientInfo.client,
    clientInfo.contractHashFactory,
    clientInfo.contractAddressFactory
  );

  let contractAddressLbPair;
  let contractAddressLbToken;
  let contractAddressLbStaking;

  const base_factor: number = 0;
  const filter_period = 3;
  const decay_period = 60;
  const reduction_factor = 500;
  const variable_fee_control = 0;
  const protocol_share = 1000;
  const max_volatility_accumulator = 350000;
  const is_open = true;
  const epoch_staking_duration = 10;
  const epoch_staking_index = 1;
  const rewards_distribution_algorithm = "time_based_rewards";
  const total_reward_bins = 100;

  let bins_steps = [100];

  for (const bin_step of bins_steps) {
    await executeSetPreset(
      clientInfo.client,
      clientInfo.contractHashFactory,
      clientInfo.contractAddressFactory,
      bin_step,
      base_factor,
      filter_period,
      decay_period,
      reduction_factor,
      variable_fee_control,
      protocol_share,
      max_volatility_accumulator,
      is_open,
      epoch_staking_duration,
      epoch_staking_index,
      rewards_distribution_algorithm,
      total_reward_bins
    );
    await sleep();
  }

  // TOKENY

  await sleep();

  if ("custom_token" in clientInfo.tokenY) {
    await executeAddQuoteAsset(
      clientInfo.client,
      clientInfo.contractHashFactory,
      clientInfo.contractAddressFactory,
      clientInfo.tokenY.custom_token.token_code_hash,
      clientInfo.tokenY.custom_token.contract_addr
    );
  }
  await queryPreset(
    clientInfo.client,
    clientInfo.contractHashFactory,
    clientInfo.contractAddressFactory
  );

  const active_id = 8388608;

  if (
    "custom_token" in clientInfo.tokenX &&
    "custom_token" in clientInfo.tokenY
  ) {
    const bin_step = 100;
    await executeCreateLBPair(
      clientInfo.client,
      clientInfo.contractHashFactory,
      clientInfo.contractAddressFactory,
      clientInfo.tokenX.custom_token.token_code_hash,
      clientInfo.tokenX.custom_token.contract_addr,
      clientInfo.tokenY.custom_token.token_code_hash,
      clientInfo.tokenY.custom_token.contract_addr,
      active_id,
      bin_step
    );
    // query lb_pair from lb_factory

    let lb_pair_info = await queryLBPairInformation(
      clientInfo.client,
      clientInfo.contractHashFactory,
      clientInfo.contractAddressFactory,
      clientInfo.tokenX.custom_token.token_code_hash,
      clientInfo.tokenX.custom_token.contract_addr,
      clientInfo.tokenY.custom_token.token_code_hash,
      clientInfo.tokenY.custom_token.contract_addr,
      100
    );

    clientInfo.contractAddressPair =
      lb_pair_info.lb_pair_information.info.contract.address;
    let lb_token_info = await queryLbToken(
      clientInfo.client,
      clientInfo.contractHashPair,
      clientInfo.contractAddressPair
    );
    clientInfo.contractAddressToken = lb_token_info.contract.address;

    await setViewingKey(
      clientInfo.client,
      clientInfo.contractHashToken,
      clientInfo.contractAddressToken
    );

    let lb_staking_info = await queryLbStaking(
      clientInfo.client,
      clientInfo.contractHashPair,
      clientInfo.contractAddressPair
    );

    clientInfo.contractAddressStaking = lb_staking_info.contract.address;
  }

  await sleep();

  return clientInfo;
}

export interface clientInfo {
  client: SecretNetworkClient;
  contractHashFactory: string;
  contractAddressFactory: string;
  contractHashRouter: string;
  contractAddressRouter: string;
  codeIdPair: number;
  contractAddressPair: string;
  contractHashPair: string;
  codeIdToken: number;
  contractAddressToken: string;
  contractHashToken: string;
  codeIdStaking: number;
  contractHashStaking: string;
  contractAddressStaking: string;
  tokenX: TokenType;
  tokenY: TokenType;
}

// Your initialization procedure, modified to use the above functions
// export async function initializeAndUploadContractDummy() {
//   const client = await initializeClient(endpoint, chainId);
//   const contractFilePath = path.join(__dirname, "contract_address_log.json");
//   const contractsData = readContractAddresses(contractFilePath);

//   if (chainId == "secretdev-1") {
//     await fillUpFromFaucet(client, 100_000_000);
//   }

//   // Pair Contract
//   await uploadPairContract(client, build + "lb_pair.wasm");
// }

export async function runTestFunction(
  tester: (clientInfo: clientInfo) => void,
  clientInfo: clientInfo
) {
  console.log(`Testing ${tester.name}`);
  tester(clientInfo);
  console.log(`[\x1b[32m SUCCESS \x1b[0m] ${tester.name}`);
}
