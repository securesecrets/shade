import fs from "fs";
import { SecretNetworkClient } from "secretjs";
import { logGasToFile, logToFile, sleep } from "../helper";

export const initializeStakingContract = async (
  client: SecretNetworkClient,
  contractPath: string
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
      gasLimit: 6_000_000,
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
  console.log("Staking Contract codeId: ", codeId);

  await sleep();
  const contractCodeHash = (
    await client.query.compute.codeHashByCodeId({ code_id: String(codeId) })
  ).code_hash;

  if (contractCodeHash === undefined) {
    throw new Error(`Failed to get code hash`);
  }

  console.log(`Staking Contract hash: ${contractCodeHash}`);

  logGasToFile(`Staking Upload used ${uploadReceipt.gasUsed} gas`);

  logToFile(`LB_STAKING_CODE_ID=${codeId}`);
  logToFile(`LB_STAKING_CODE_HASH="${contractCodeHash}"`);

  const codeInfo: [number, string] = [codeId, contractCodeHash];
  return codeInfo;
};
