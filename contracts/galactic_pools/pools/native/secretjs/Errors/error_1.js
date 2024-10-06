import pkg from "secretjs";
const { Wallet, SecretNetworkClient, grpc, signAmino } = pkg;
const grpcWebUrl = "https://grpc.testnet.secretsaturn.net/";
import fs from "fs";
import { debug } from "console";
import * as dotenv from "dotenv"; // see https://github.com/motdotla/dotenv#how-do-i-use-dotenv-with-import
import fetch from "node-fetch";
dotenv.config({ path: "../.env" });

// ERROR OCCUR WHE USING "PULSAR-1" VALIDATOR AKA "secretvaloper1zd2j39tgjkv3z8eqqp86q54wylellsz5dyfcc4" ON PULSAR-2

// A for loop == 200
// *Deposit
// *Query the contract
// *Query the contract account

for (let i = 0; i < 100; i++) {
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
  let amount = String(1 * process.env.SCRT_TO_USCRT);

  let gasLimit = 1000000;
  try {
    const tx = await secretjs.tx.compute.executeContract(
      {
        sender: myAddress,
        contractAddress: contractAddress,
        codeHash: codeHash, // optional but way faster
        msg: {
          deposit: {},
        },
        sentFunds: [{ amount: amount, denom: "uscrt" }], // optional
      },
      {
        gasLimit,
      }
    );
    console.log(
      `Deposited ${amount / process.env.SCRT_TO_USCRT} Scrt successfully`
    );
  } catch (err) {
    console.log(err);
  }

  setTimeout(() => {
    console.log("waiting!");
  }, 20000);

  const allowedTokens = [data.address];
  const permissions = ["owner", "delegated"];
  const chainId = process.env.SECRET_CHAIN_ID;
  let permitName = "Permit1";

  let permit = await secretjs.utils.accessControl.permit.sign(
    myAddress,
    chainId,
    permitName,
    allowedTokens,
    permissions,
    false
  );
  let delegated;
  try {
    delegated = await secretjs.query.compute.queryContract({
      contractAddress: contractAddress,
      codeHash: codeHash, // optional but way faster
      query: {
        with_permit: {
          permit,

          query: {
            delegated: {},
          },
        },
      },
    });
  } catch (err) {
    console.log(err);
  }
  console.log("Total delegated: " + delegated.amount);

  let api_url = `https://api.pulsar.scrttestnet.com/cosmos/staking/v1beta1/delegations/${contractAddress}`;

  let balance = 0;
  fetch(api_url)
    .then(function (response) {
      // The API call was successful!
      return response.json();
    })
    .then(function (data) {
      // This is the JSON from our response
      for (let i = 0; i < data.delegation_responses.length; i++) {
        balance += parseInt(data.delegation_responses[i].balance.amount);
      }
      console.log("Total delegated using Api: " + balance);

      if (balance != parseInt(delegated.amount)) {
        console.warn(
          "Both amounts are not equal:",
          "Api:",
          balance,
          "contract state:",
          delegated.amount
        );
      } else {
        console.warn("Both amounts are equal");
      }
    })
    .catch(function (err) {
      // There was an error
      console.warn("Something went wrong.", err);
    });
}
