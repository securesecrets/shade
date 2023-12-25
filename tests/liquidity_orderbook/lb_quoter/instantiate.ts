import { SecretNetworkClient } from "secretjs";
import * as LBQuoter from "./types"
import fs from "fs";
import { logGasToFile, logToFile, sleep } from "../integration";

export const initializeQuoterContract = async (
    client: SecretNetworkClient,
    contractPath: string
  ) => {
    const wasmCode = fs.readFileSync(contractPath);
    console.log("Uploading contract");
  
    const uploadReceipt = await client.tx.compute.storeCode(
      {
        wasm_byte_code: wasmCode,
        sender: client.address,
        source: "",
        builder: "",
      },
      {
        gasLimit: 5000000,
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
    console.log("Contract codeId: ", codeId);
    
    await sleep();
    const contractCodeHash = (await client.query.compute.codeHashByCodeId({code_id: String(codeId)})).code_hash;
  
    if (contractCodeHash === undefined) {
      throw new Error(`Failed to get code hash`);
    }
  
    console.log(`Contract hash: ${contractCodeHash}`);
  
    const contract = await client.tx.compute.instantiateContract(
      {
        sender: client.address,
        code_id: codeId,
        init_msg: {
          
        },
        code_hash: contractCodeHash,
        label: "LBQuoter" + Math.ceil(Math.random() * 10000), // The label should be unique for every contract, add random string in order to maintain uniqueness
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
  
    console.log(`Contract address: ${contractAddress}`);
  
    const contractInfo: [string, string] = [contractCodeHash, contractAddress];
    return contractInfo;
  };