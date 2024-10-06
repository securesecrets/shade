import { Wallet, SecretNetworkClient } from "secretjs";
const grpcWebUrl = "https://grpc.testnet.secretsaturn.net/";

import * as dotenv from "dotenv"; // see https://github.com/motdotla/dotenv#how-do-i-use-dotenv-with-import
dotenv.config({ path: "../.env" });
import fs from "fs";
import { debug } from "console";

async function end_lottery_loop() {
  const wallet = new Wallet(process.env.MNEMONIC);
  const myAddress = wallet.address;

  // To create a signer secret.js client, also pass in a wallet
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId: "pulsar-2",
    wallet: wallet,
    walletAddress: myAddress,
  });

  let bufferData = fs.readFileSync("./../contract_details.json");
  let stData = bufferData.toString();
  let data = JSON.parse(stData);

  const contractAddress = data.address;

  const codeHash = data.hash;

  // let gasLimit = Math.ceil(sim.gasInfo.gasUsed * 1.9)
  let gasLimit = 1000000;

  setInterval(async () => {
    console.log("STARTING");
    try {
      const tx = await secretjs.tx.compute.executeContract(
        {
          sender: myAddress,
          contractAddress: contractAddress,
          codeHash: codeHash, // optional but way faster
          msg: {
            end_round: {},
          },
        },
        {
          gasLimit,
        }
      );
      console.log(tx);
      console.log(tx.tx.body.messages);
    } catch (err) {
      console.log(err);
    }
    console.log("Waiting");
  }, 60 * 32 * 1000);
}

export default end_lottery_loop;
