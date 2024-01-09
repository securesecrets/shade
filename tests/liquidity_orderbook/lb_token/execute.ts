import { SecretNetworkClient } from "secretjs";
import * as LBToken from "./types";

export async function setViewingKey(
  client: SecretNetworkClient,
  contractHashLbToken: string,
  contractAddressLbToken: string
) {
  const msg: LBToken.ExecuteMsg = {
    set_viewing_key: {
      key: "viewing_key",
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
      gasLimit: 1_400_000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
}
