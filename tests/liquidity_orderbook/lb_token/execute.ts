import { SecretNetworkClient } from "secretjs";
import * as LBToken from "./types";

let approveForAll = {
  "approveForAll": {
    "spender": "secret1mz0cdjxk72mnqfuy4v6y9c6",
    "approved": true
  }
}

let batchTransferFrom = {
  "batchTransferFrom": {
    "from": "secret1mz0cdjxk72mnqfuy4v6y9c6",
    "to": "secret1mf7tzqxzvqhpv7m62ccq3gq",
    "ids": [1, 2, 3],
    "amounts": ["1000000000000000000", "2000000000000000000", "3000000000000000000"]
  }
}

let mint: LBToken.MintMsg = {
  "mint": {
      "recipient": "secret1mz0cdjxk72mnqfuy4v6y9c6",
      "id": 123,
      "amount": "1000000000000000000"
  }
}

let burn = {
  "burn": {
    "owner": "secret1mz0cdjxk72mnqfuy4v6y9c6",
    "id": 123,
    "amount": "1000000000000000000"
  }
}


async function executeMint(
    client: SecretNetworkClient,
    contractHash: string,
    contractAddess: string
  ) {
    const tx = await client.tx.compute.executeContract(
      {
        sender: client.address,
        contract_address: contractAddess,
        code_hash: contractHash,
        msg: mint,
        sent_funds: [],
      },
      {
        gasLimit: 300000,
      }
    );
  
    //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
    console.log(`Mint TX used ${tx.gasUsed} gas`);
  }
  