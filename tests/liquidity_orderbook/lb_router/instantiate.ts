import fs from "fs";
import { SecretNetworkClient } from "secretjs";
import { logGasToFile, logToFile, sleep } from "../helper";
import { Contract, InitMsg } from "./types";

export const initializeRouterContract = async (
  client: SecretNetworkClient,
  contractPath: string,
  adminAuthAddress: string,
  adminAuthHash: string
) => {
  const wasmCode = fs.readFileSync(contractPath);
  console.log("\nUploading contract");

  const uploadReceipt = await client.tx.compute.storeCode(
    {
      wasm_byte_code: wasmCode,
      sender: client.address,
      source: "",
      builder: "",
    },
    {
      gasLimit: 5_000_000,
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

  const codeId = Number(codeIdKv!.value);
  console.log("Router Contract codeId: ", codeId);

  await sleep();
  const contractCodeHash = (
    await client.query.compute.codeHashByCodeId({ code_id: String(codeId) })
  ).code_hash;

  if (contractCodeHash === undefined) {
    throw new Error(`Failed to get code hash`);
  }

  console.log(`Router Contract hash: ${contractCodeHash}`);

  let admin_auth_contract: Contract = {
    address: adminAuthAddress,
    code_hash: adminAuthHash,
  };

  const init_msg: InitMsg = {
    admin_auth: admin_auth_contract,
    airdrop_address: null,
    entropy: "",
    prng_seed: "",
  };

  const contract = await client.tx.compute.instantiateContract(
    {
      sender: client.address,
      code_id: codeId,
      init_msg: init_msg,
      code_hash: contractCodeHash,
      label: "LBRouter" + Math.ceil(Math.random() * 10000), // The label should be unique for every contract, add random string in order to maintain uniqueness
    },
    {
      gasLimit: 1000000,
    }
  );

  if (contract.code !== 0) {
    throw new Error(
      `Failed to instantiate the contract with the following error ${contract.rawLog}`
    );
  }

  const contractAddress = contract.arrayLog!.find(
    (log) => log.type === "message" && log.key === "contract_address"
  )!.value;

  console.log(`Router Contract address: ${contractAddress}`);

  logGasToFile(`Router Upload used ${uploadReceipt.gasUsed} gas`);
  logGasToFile(`Router Instantiation used ${contract.gasUsed} gas`);

  logToFile(`LB_ROUTER_ADDRESS="${contractAddress}"`);
  logToFile(`LB_ROUTER_CODE_HASH="${contractCodeHash}"`);

  const contractInfo: [string, string] = [contractCodeHash, contractAddress];
  return contractInfo;
};
