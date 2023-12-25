import { SecretNetworkClient } from "secretjs";
import * as LBToken from "./types";
import fs from "fs";
import { logGasToFile, logToFile, sleep } from "../integration";

export const initializeTokenContract = async (
    client: SecretNetworkClient,
    contractPath: string,
    // lb_pair: string,
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
        gasLimit: 3_000_000,
      }
    );
  
    if (uploadReceipt.code !== 0) {
      console.log(
        `Failed to get code id: ${JSON.stringify(uploadReceipt.rawLog)}`
      );
      throw new Error(`Failed to upload contract`);
    }

    console.log(`Upload used ${uploadReceipt.gasUsed} gas`);

    const codeIdKv = uploadReceipt.jsonLog![0].events[0].attributes.find(
      (a: any) => {
        return a.key === "code_id";
      }
    );
  
    const codeId = Number(codeIdKv!.value);
    console.log("Token Contract codeId: ", codeId);
    
    await sleep();
    const contractCodeHash = (await client.query.compute.codeHashByCodeId({code_id: String(codeId)})).code_hash;
  
    if (contractCodeHash === undefined) {
      throw new Error(`Failed to get code hash`);
    }
  
    console.log(`Token Contract hash: ${contractCodeHash}`);

    // const initMsg: LBToken.InstantiateMsg = {
    //   name: "token name",
    //   symbol: "token symbol",
    //   decimals: 18,
    //   lb_pair: lb_pair,
    // }
  
    // const contract = await client.tx.compute.instantiateContract(
    //   {
    //     sender: client.address,
    //     code_id: codeId,
    //     init_msg: initMsg,
    //     code_hash: contractCodeHash,
    //     label: "LBToken" + Math.ceil(Math.random() * 10000), // The label should be unique for every contract, add random string in order to maintain uniqueness
    //   },
    //   {
    //     gasLimit: 1000000,
    //   }
    // );
  
    // if (contract.code !== 0) {
    //   throw new Error(
    //     `Failed to instantiate the contract with the following error ${contract.rawLog}`
    //   );
    // }
  
    // const contractAddress = contract.arrayLog!.find(
    //   (log) => log.type === "message" && log.key === "contract_address"
    // )!.value;
  
    // console.log(`Token Contract address: ${contractAddress}`);

    logGasToFile(`Token Upload used ${uploadReceipt.gasUsed} gas`);

    logToFile(`LB_TOKEN_CODE_ID=${codeId}`);
    logToFile(`LB_TOKEN_CODE_HASH="${contractCodeHash}"`);
  
    const codeInfo: [number, string] = [codeId, contractCodeHash];
    return codeInfo;
  };