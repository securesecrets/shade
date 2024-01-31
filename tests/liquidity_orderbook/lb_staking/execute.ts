import { SecretNetworkClient } from "secretjs";
import * as LBToken from "../lb_token/types";
import * as LBStaking from "./types";

export async function executeStake(
  client: SecretNetworkClient,
  contractHashLbToken: string,
  contractAddressLbToken: string,
  contractHashStaking: string,
  contractAddressStaking: string,
  ids: number[],
  amounts: string[]
) {
  const staking_msg: LBStaking.InvokeMsg = {
    stake: { from: client.address },
  };

  let actions: LBToken.SendAction[] = [];

  // Loop to fill actions
  for (let i = 0; i < ids.length; i++) {
    const action: LBToken.SendAction = {
      amount: amounts[i],
      from: client.address,
      recipient: contractAddressStaking,
      token_id: ids[i].toString(),
      msg: Buffer.from(JSON.stringify(staking_msg)).toString("base64"),
    };
    actions.push(action);
  }

  const msg: LBToken.ExecuteMsg = {
    batch_send: {
      actions: actions,
    },
  };

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressLbToken,
      code_hash: contractHashLbToken,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 16000000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  console.log(`Staking TX used ${tx.gasUsed} gas`);
}

export async function executeUnstake(
  client: SecretNetworkClient,
  contractHashStaking: string,
  contractAddressStaking: string,
  ids: number[],
  amounts: string[]
) {
  const msg: LBStaking.ExecuteMsg = {
    unstake: {
      amounts,
      token_ids: ids,
    },
  };

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: contractAddressStaking,
      code_hash: contractHashStaking,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 16000000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }
  console.log(`Unstaking TX used ${tx.gasUsed} gas`);
}
