import fs from "fs";
import { SecretNetworkClient } from "secretjs";
import { logGasToFile, logToFile, sleep } from "../helper";

export const initializePairContract = async (
  client: SecretNetworkClient,
  contractPath: string
  // codeAddressFactory: string,
  // codeHashFactory: string
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
  console.log("Pair Contract codeId: ", codeId);

  await sleep();
  const contractCodeHash = (
    await client.query.compute.codeHashByCodeId({ code_id: String(codeId) })
  ).code_hash;

  if (contractCodeHash === undefined) {
    throw new Error(`Failed to get code hash`);
  }

  console.log(`Pair Contract hash: ${contractCodeHash}`);

  // let lb_factory_contract: ContractInfo = {
  //   address: "factory_lol",
  //   code_hash: "hash",
  // };

  // let admin_auth: RawContract = {
  //   address: "anbc",
  //   code_hash: "xyz",
  // };

  // let impl: ContractInstantiationInfo = {
  //   code_hash: "xyz",
  //   id: 1,
  // };

  // let parameter: StaticFeeParameters = {
  //   base_factor: 0,
  //   decay_period: 0,
  //   filter_period: 0,
  //   max_volatility_accumulator: 0,
  //   protocol_share: 0,
  //   reduction_factor: 0,
  //   variable_fee_control: 0,
  // };
  // const initMsg: InstantiateMsg = {
  //   factory: lb_factory_contract,
  //   token_x: {
  //     custom_token: {
  //       contract_addr: "xyz",
  //       token_code_hash: "abc",
  //     },
  //   },
  //   token_y: {
  //     custom_token: {
  //       contract_addr: "lmoa",
  //       token_code_hash: "lmao",
  //     },
  //   },
  //   active_id: 83000,
  //   admin_auth,
  //   bin_step: 0,
  //   entropy: "",
  //   epoch_staking_duration: 0,
  //   epoch_staking_index: 0,
  //   lb_token_implementation: impl,
  //   pair_parameters: parameter,
  //   protocol_fee_recipient: "",
  //   recover_staking_funds_receiver: "",
  //   rewards_distribution_algorithm: "time_based_rewards",
  //   staking_contract_implementation: impl,
  //   viewing_key: "",
  // };

  // const contract = await client.tx.compute.instantiateContract(
  //   {
  //     sender: client.address,
  //     code_id: codeId,
  //     init_msg: initMsg,
  //     code_hash: contractCodeHash,
  //     label: "LB_PAIR" + Math.ceil(Math.random() * 10000), // The label should be unique for every contract, add random string in order to maintain uniqueness
  //   },
  //   {
  //     gasLimit: 200_000,
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

  // console.log(`LBPAIR Contract address: ${contractAddress}`);
  // console.log(`Instantiation used ${contract.gasUsed} gas`);

  logGasToFile(`LBPAIR Upload used ${uploadReceipt.gasUsed} gas`);

  logGasToFile(`Pair Upload used ${uploadReceipt.gasUsed} gas`);

  logToFile(`LB_PAIR_CODE_ID=${codeId}`);
  logToFile(`LB_PAIR_CODE_HASH="${contractCodeHash}"`);

  const codeInfo: [number, string] = [codeId, contractCodeHash];
  return codeInfo;
};
