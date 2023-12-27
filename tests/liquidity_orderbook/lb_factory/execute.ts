import { SecretNetworkClient } from "secretjs";
import { logGasToFile } from "../helper";
import * as LBFactory from "./types";

export async function executeSetLBPairImplementation(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  codeIdPair: number,
  contractHashPair: string
) {
  console.log(codeIdPair, contractHashPair);
  const msg: LBFactory.ExecuteMsg = {
    set_lb_pair_implementation: {
      implementation: {
        id: codeIdPair,
        code_hash: contractHashPair,
      },
    },
  };
  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressFactory,
      code_hash: contractHashFactory,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 200_000,
    }
  );

  console.log(tx);

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`SetLBPairImplementation TX used ${tx.gasUsed} gas`);
  logGasToFile(`SetLBPairImplementation TX used ${tx.gasUsed} gas`);
}

export async function executeSetLBTokenImplementation(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  codeIdToken: number,
  contractHashToken: string
) {
  const msg: LBFactory.ExecuteMsg = {
    set_lb_token_implementation: {
      implementation: {
        id: codeIdToken,
        code_hash: contractHashToken,
      },
    },
  };
  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressFactory,
      code_hash: contractHashFactory,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 200_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`SetLBTokenImplementation TX used ${tx.gasUsed} gas`);
  logGasToFile(`SetLBTokenImplementation TX used ${tx.gasUsed} gas`);
}

export async function executeCreateLBPair(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  contractHashTokenA: string,
  contractAddressTokenA: string,
  contractHashTokenB: string,
  contractAddressTokenB: string,
  active_id: number, // 8388607 is the middle bin
  bin_step: number // 100 represents a 1% bin step
) {
  const msg: LBFactory.ExecuteMsg = {
    create_lb_pair: {
      token_x: {
        custom_token: {
          contract_addr: contractAddressTokenA,
          token_code_hash: contractHashTokenA,
        },
      },
      token_y: {
        custom_token: {
          contract_addr: contractAddressTokenB,
          token_code_hash: contractHashTokenB,
        },
      },
      active_id: active_id,
      bin_step: bin_step,
      entropy: "",
      viewing_key: "",
    },
  };

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressFactory,
      code_hash: contractHashFactory,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 1_000_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`CreateLBPair TX used ${tx.gasUsed} gas`);
  logGasToFile(`CreateLBPair TX used ${tx.gasUsed} gas`);
}

export async function executeSetPreset(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  bin_step: number,
  base_factor: number,
  filter_period: number,
  decay_period: number,
  reduction_factor: number,
  variable_fee_control: number,
  protocol_share: number,
  max_volatility_accumulator: number,
  is_open: boolean,
  epoch_staking_duration: number,
  epoch_staking_index: number,
  rewards_distribution_algorithm: LBFactory.RewardsDistributionAlgorithm,
  total_reward_bins: number
) {
  const msg: LBFactory.ExecuteMsg = {
    set_pair_preset: {
      // TODO: figure out approprate values to use
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
      total_reward_bins,
    },
  };

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressFactory,
      code_hash: contractHashFactory,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 200_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`Preset Set for bin_step ${bin_step}`);
  console.log(`SetPreset TX used ${tx.gasUsed} gas`);
  logGasToFile(`SetPreset TX used ${tx.gasUsed} gas`);
}

export async function executeAddQuoteAsset(
  client: SecretNetworkClient,
  contractHashFactory: string,
  contractAddressFactory: string,
  contractHashQuoteAsset: string,
  contractAddressQuoteAsset: string
) {
  const msg: LBFactory.ExecuteMsg = {
    add_quote_asset: {
      asset: {
        custom_token: {
          contract_addr: contractAddressQuoteAsset,
          token_code_hash: contractHashQuoteAsset,
        },
      },
    },
  };

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressFactory,
      code_hash: contractHashFactory,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 200_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`AddQuoteAsset TX used ${tx.gasUsed} gas`);
  logGasToFile(`AddQuoteAsset TX used ${tx.gasUsed} gas`);
}
