import { Wallet, SecretNetworkClient } from "secretjs";
const grpcWebUrl = "https://grpc.testnet.secretsaturn.net/";

import * as dotenv from "dotenv"; // see https://github.com/motdotla/dotenv#how-do-i-use-dotenv-with-import
dotenv.config({ path: "../.env" });
import fs from "fs";
import { debug } from "console";

async function deposits(amt = 10) {
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
  let amount = String(amt * process.env.SCRT_TO_USCRT);

  let sim = await secretjs.tx.compute.executeContract.simulate({
    sender: myAddress,
    contractAddress: contractAddress,
    codeHash: codeHash, // optional but way faster
    msg: {
      deposit: {},
    },
    sentFunds: [{ amount: amount, denom: "uscrt" }], // optional
  });

  let gasLimit = Math.ceil(sim.gasInfo.gasUsed * 1.34);
  console.log(gasLimit);
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

  let user = {
    address: myAddress,
    deposits: amount,
  };
  let usersbufferData;
  try {
    usersbufferData = fs.readFileSync("../users_details.json");
  } catch (err) {
    let vec = [];
    vec.push(user);
    var user_details = JSON.stringify(vec);
    console.log(user_details);
    fs.writeFile("../users_details.json", user_details, function (err, result) {
      if (err) console.log("error", err);
    });
  }
  if (usersbufferData != null) {
    // if user exists

    let usersStData = usersbufferData.toString();
    let usersdata = JSON.parse(usersStData);

    var newUser = user;

    let exisits = false;
    usersdata.forEach(function (userdata) {
      if (newUser.address === userdata.address) {
        userdata.deposits = String(
          Number(userdata.deposits) + Number(newUser.deposits)
        );
        exisits = true;
      }
    });
    if (!exisits) {
      usersdata.push(user);
    }
    var user_details = JSON.stringify(usersdata);

    fs.writeFile("../users_details.json", user_details, function (err, result) {
      if (err) console.log("error", err);
    });
  }
  return true;
}

export default deposits;
