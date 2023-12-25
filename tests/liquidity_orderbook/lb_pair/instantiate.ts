import { SecretNetworkClient } from "secretjs";
import * as LBPair from "./types"
import fs from "fs";
import { logGasToFile, logToFile, sleep } from "../integration";

export const initializePairContract = async (
    client: SecretNetworkClient,
    contractPath: string,
    // codeIdToken: number,
    // codeHashToken: string,
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
        gasLimit: 4_000_000,
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
    console.log("Pair Contract codeId: ", codeId);
  
    await sleep();
    const contractCodeHash = (await client.query.compute.codeHashByCodeId({code_id: String(codeId)})).code_hash;
  
    if (contractCodeHash === undefined) {
      throw new Error(`Failed to get code hash`);
    }
  
    console.log(`Pair Contract hash: ${contractCodeHash}`);

    // NOTE: factory will instantiate this

    // const initMsg: LBPair.InstantiateMsg = {
    //   factory: "secret1qxxlalvsdjd07p07y3rc5fu6ll8k4tme6e2scc",
    //   // TODO: populate these with real values
    //   token_x: {
    //     address: "secret1qxxlalvsdjd07p07y3rc5fu6ll8k4tme6e2scc",
    //     code_hash: "b69957a5c29cb7a64a15c089d9a0aa81e686de650c7a5a7d8644edab251a84d1",
    //   },
    //   token_y: {
    //     address: "secret1qxxlalvsdjd07p07y3rc5fu6ll8k4tme6e2scc",
    //     code_hash: "b69957a5c29cb7a64a15c089d9a0aa81e686de650c7a5a7d8644edab251a84d1",
    //   },
    //   bin_step: 100,
    //   pair_parameters: {
    //     base_factor: 1,
    //     filter_period: 1,
    //     decay_period: 1,
    //     reduction_factor: 1,
    //     variable_fee_control: 1,
    //     protocol_share: 1,
    //     max_volatility_accumulator: 1,
    //   },
    //   active_id: 8388607,
    //   lb_token_implementation: {
    //     id: codeIdToken,
    //     code_hash: codeHashToken,
    //   },
    // };
  
    // const contract = await client.tx.compute.instantiateContract(
    //   {
    //     sender: client.address,
    //     code_id: codeId,
    //     init_msg: initMsg,
    //     code_hash: contractCodeHash,
    //     label: "LBPair" + Math.ceil(Math.random() * 10000), // The label should be unique for every contract, add random string in order to maintain uniqueness
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
  
    // console.log(`Pair Contract address: ${contractAddress}`);

    logGasToFile(`Pair Upload used ${uploadReceipt.gasUsed} gas`);

    logToFile(`LB_PAIR_CODE_ID=${codeId}`);
    logToFile(`LB_PAIR_CODE_HASH="${contractCodeHash}"`);
  
    const codeInfo: [number, string] = [codeId, contractCodeHash];
    return codeInfo;
  };