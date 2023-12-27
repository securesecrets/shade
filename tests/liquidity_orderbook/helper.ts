import axios from "axios";
import dotenv from "dotenv";
import fs from "fs";
import { SecretNetworkClient, Wallet } from "secretjs";
import { TokenType, initializeFactoryContract } from "./lb_factory";
import { initializePairContract } from "./lb_pair";
import { initializeRouterContract } from "./lb_router";
import { initializeTokenContract } from "./lb_token";

dotenv.config({ path: ".env" });

const build = "./wasm/";

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
        amount: "340282366920938463463374607431768211453",
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

  // second token contract

  const init_msg_y = {
    name: "token y",
    admin: client.address,
    symbol: "TOKENY",
    decimals: 6,
    initial_balances: [
      {
        address: client.address,
        amount: "340282366920938463463374607431768211453",
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

// Initialization procedure
// export async function initializeAndUploadContract() {
//   const client = await initializeClient(endpoint, chainId);

//   if (chainId == "secretdev-1") {
//     await fillUpFromFaucet(client, 100_000_000);
//   }

//   const [tokenX, tokenY] = await initializeSnip20Contract(
//     client,
//     build + "snip25.wasm"
//   );
//   await sleep();

//   const [contractHashFactory, contractAddressFactory] =
//     await initializeFactoryContract(
//       client,
//       build + "lb_factory.wasm",
//       client.address,
//       ""
//     );
//   await sleep();

//   const [contractHashRouter, contractAddressRouter] =
//     await initializeRouterContract(
//       client,
//       build + "router.wasm",
//       contractHashFactory,
//       contractAddressFactory
//     );
//   await sleep();

//   const [codeIdPair, contractHashPair] = await initializePairContract(
//     client,
//     build + "lb_pair.wasm"
//   );
//   await sleep();

//   const [codeIdToken, contractHashToken] = await initializeTokenContract(
//     client,
//     build + "lb_token.wasm"
//   );
//   await sleep();

//   var clientInfo: [
//     SecretNetworkClient,
//     string,
//     string,
//     string,
//     string,
//     number,
//     string,
//     number,
//     string,
//     TokenType,
//     TokenType
//   ] = [
//     client,
//     contractHashFactory,
//     contractAddressFactory,
//     contractHashRouter,
//     contractAddressRouter,
//     codeIdPair,
//     contractHashPair,
//     codeIdToken,
//     contractHashToken,
//     tokenX,
//     tokenY,
//   ];
//   return clientInfo;
// }

import path from "path";
import { initializeAdminAuth } from "./admin_auth/instantiate";

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
  const contractFilePath = path.join(__dirname, "contract_address_log.json");
  const contractsData = readContractAddresses(contractFilePath);

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
    contractHashToken: any;

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
        build + "lb_factory.wasm",
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
        build + "router.wasm",
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
    [codeIdPair, contractHashPair] = await initializePairContract(
      client,
      build + "lb_pair.wasm"
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
    [codeIdToken, contractHashToken] = await initializeTokenContract(
      client,
      build + "lb_token.wasm"
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
    tokenX,
    tokenY,
  ];
  return clientInfo;
}

export async function runTestFunction(
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
    tokenX: TokenType,
    tokenY: TokenType
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
  tokenX: TokenType,
  tokenY: TokenType
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
    tokenY
  );
  console.log(`[\x1b[32m SUCCESS \x1b[0m] ${tester.name}`);
}
