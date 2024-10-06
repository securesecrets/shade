#!/usr/bin/node

import { Wallet, SecretNetworkClient } from "secretjs";
const grpcWebUrl = "https://grpc.testnet.secretsaturn.net/";

import * as dotenv from "dotenv"; // see https://github.com/motdotla/dotenv#how-do-i-use-dotenv-with-import
dotenv.config({ path: "../.env" });
import fs from "fs";

const wallet = new Wallet(process.env.MNEMONIC);
const myAddress = wallet.address;

// To create a signer secret.js client, also pass in a wallet
const secretjs = await SecretNetworkClient.create({
  grpcWebUrl,
  chainId: "pulsar-2",
  wallet: wallet,
  walletAddress: myAddress,
});

const tx1 = await secretjs.tx.compute.storeCode(
  {
    sender: myAddress,
    wasmByteCode: fs.readFileSync(`../../contract.wasm.gz`),
    source: "",
    builder: "",
  },
  {
    broadcastCheckIntervalMs: 100,
    gasLimit: 4_000_000,
  }
);
console.log(tx1);

const codeId = Number(
  tx1.arrayLog.find((log) => log.type === "message" && log.key === "code_id")
    .value
);

console.log("Uploaded.....");

const {
  codeInfo: { codeHash },
} = await secretjs.query.compute.code(codeId);

let common_divisor = 10000;

let validator_vector = [
  {
    address: "secretvaloper1p0re3rp685fqsngfdvxg34wkwu9am2p4ckeq2h",
    weightage: (60 * common_divisor) / 100,
  },
  {
    address: "secretvaloper1wcxr6l4hk7cf7yjlz2v68wxqvdvtf5cwwyymu2",
    weightage: (40 * common_divisor) / 100,
  },
];

let rewards_distribution = {
  tier_0: {
    total_number_of_winners: String(1),
    percentage_of_rewards: (20 * 10000) / 100,
  },
  tier_1: {
    total_number_of_winners: String(3),
    percentage_of_rewards: (10 * 10000) / 100,
  },
  tier_2: {
    total_number_of_winners: String(9),
    percentage_of_rewards: (14 * 10000) / 100,
  },
  tier_3: {
    total_number_of_winners: String(27),
    percentage_of_rewards: (12 * 10000) / 100,
  },
  tier_4: {
    total_number_of_winners: String(81),
    percentage_of_rewards: (19 * 10000) / 100,
  },
  tier_5: {
    total_number_of_winners: String(243),
    percentage_of_rewards: (25 * 10000) / 100,
  },
};

let admins = ["secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7"];
let triggerers = [
  "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
  "secret1dr6vh9zlcjj69vv3av8mmpemwdg9z5jjad8u94",
];
let reviewers = ["secret1dr6vh9zlcjj69vv3av8mmpemwdg9z5jjad8u94"];

let initMsg = {
  admin: admins,
  triggerer: triggerers,
  reviewer: reviewers,
  triggerer_share_percentage: (1 * 10000) / 100, //dividing by 1/100 * common_divisor
  denom: "uscrt",
  prng_seed: "ZW5pZ21hLXJvY2tzCg==",
  validator: validator_vector,
  unbonding_duration: 60 * 14, //21 days/ 14 mins on testnet
  // round_duration: 60 * 3, //7 days
  round_duration: 60 * 30, //7 days -  30 mins

  rewards_distribution,
  ticket_price: String(1 * 1000000),
  rewards_expiry_duration: 3600 * 24, // 45 days - 24 hours
  common_divisor,
  total_admin_share: (10 * common_divisor) / 100,
  shade_percentage_share: (60 * common_divisor) / 100,
  galactic_pools_percentage_share: (40 * common_divisor) / 100,
  shade_rewards_address: "secret1zrpgrff3p2pc3ptqu07pz9nq4ezhjr9xauyhjf",
  galactic_pools_rewards_address:
    "secret1pa8ng8z5lwcukvqht83r5pu2zfq5vv3zdu36uu",
  reserve_percentage: (60 * common_divisor) / 100,
  is_sponosorship_admin_controlled: false,
  unbonding_batch_duration: 60 * 2, // 3 days / 2 minutes on testnet
  grand_prize_address: "secret1pa8ng8z5lwcukvqht83r5pu2zfq5vv3zdu36uu",
  number_of_tickers_per_transaction: String(1000000),
  sponsor_msg_edit_fee: String(1000000),
};

const txInit = await secretjs.tx.compute.instantiateContract(
  {
    sender: myAddress,
    codeId,
    codeHash,
    initMsg,
    label: `label-${Date.now()}`,
    initFunds: [],
  },
  {
    broadcastCheckIntervalMs: 100,
    gasLimit: 4_000_000,
  }
);

console.log(txInit);

const contractAddress = txInit.arrayLog.find(
  (log) => log.type === "message" && log.key === "contract_address"
).value;
console.log(contractAddress);

var contract = { address: contractAddress, hash: codeHash };

//2- make it JSON:

var contract_details = JSON.stringify(contract);

/* 3- save your json file and dont forget that fs.writeFile(...) 
requires a third (or fourth) parameter which is a callback 
function to be invoked when the operation completes. */

fs.writeFile(
  "../contract_details.json",
  contract_details,
  function (err, result) {
    if (err) console.log("error", err);
  }
);

try {
  fs.unlink("../users_details.json", contract_details, function (err, result) {
    if (err) console.log("error", err);
  });
} catch (e) {
  //nothing
}
