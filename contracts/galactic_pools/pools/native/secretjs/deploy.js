const {
  EnigmaUtils,
  Secp256k1Pen,
  SigningCosmWasmClient,
  pubkeyToAddress,
  encodeSecp256k1Pubkey,
} = require("secretjs");

const fs = require("fs");

// Load environment variables
require("dotenv").config();

const customFees = {
  upload: {
    amount: [{ amount: "2000000", denom: "uscrt" }],
    gas: "3500000",
  },
  init: {
    amount: [{ amount: "500000", denom: "uscrt" }],
    gas: "1500000",
  },
  exec: {
    amount: [{ amount: "500000", denom: "uscrt" }],
    gas: "500000",
  },
  send: {
    amount: [{ amount: "80000", denom: "uscrt" }],
    gas: "80000",
  },
};

const main = async () => {
  const httpUrl = process.env.SECRET_REST_URL;

  // Use key created in tutorial #2
  const mnemonic = process.env.MNEMONIC;

  // A pen is the most basic tool you can think of for signing.
  // This wraps a single keypair and allows for signing.
  const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic).catch((err) => {
    throw new Error("Could not get signing pen: ${err}");
  });

  // Get the public key
  const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);

  // get the wallet address
  const accAddress = pubkeyToAddress(pubkey, "secret");

  // 1. Initialize client
  const txEncryptionSeed = EnigmaUtils.GenerateNewSeed();

  const client = new SigningCosmWasmClient(
    httpUrl,
    accAddress,
    (signBytes) => signingPen.sign(signBytes),
    txEncryptionSeed,
    customFees
  );
  console.log(`Wallet address=${accAddress}`);
  // 2. Upload the contract wasm

  const wasm = fs.readFileSync("../contract.wasm");

  console.log("Uploading contract");
  const uploadReceipt = await client.upload(wasm, {}).catch((err) => {
    throw new Error(`Could not upload contract: ${err}`);
  });

  // 3. Create an instance of the Counter contract
  // Get the code ID from the receipt
  const { codeId } = uploadReceipt;

  // Create an instance of the Counter contract, providing a starting count //change

  let common_divisor = 10000;

  let validator_vector = [
    {
      address: "secretvaloper1p0re3rp685fqsngfdvxg34wkwu9am2p4ckeq2h",
      weightage: (60 * common_divisor) / 100,
    },
    {
      address: "secretvaloper1zd2j39tgjkv3z8eqqp86q54wylellsz5dyfcc4",
      weightage: (40 * common_divisor) / 100,
    },
  ];

  let rewards_distribution = {
    tier_0: {
      total_number_of_winners: 1,
      percentage_of_rewards: (20 * 10000) / 100,
    },
    tier_1: {
      total_number_of_winners: 3,
      percentage_of_rewards: (10 * 10000) / 100,
    },
    tier_2: {
      percentage_of_rewards: (14 * 10000) / 100,
    },
    tier_3: {
      total_number_of_winners: 27,
      percentage_of_rewards: (12 * 10000) / 100,
    },
    tier_4: {
      total_number_of_winners: 81,
      percentage_of_rewards: (19 * 10000) / 100,
    },
    tier_5: {
      total_number_of_winners: 243,
      percentage_of_rewards: (25 * 10000) / 100,
    },
  };

  let init_msg = {
    admin: "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
    triggerer: "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
    reviewer: "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
    triggerer_share_percentage: (1 * 10000) / 100, //dividing by 1/100 * common_divisor
    denom: "uscrt",
    prng_seed: "ZW5pZ21hLXJvY2tzCg==",
    validator: validator_vector,
    unbonding_duration: 3600 * 24 * 21, //21 days
    sscrt: {
      address: "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg",
      hash: "9587D60B8E6B078ACE12014CEEEE089530B9FABCD76535D93666A6C127AD8813",
    },
    round_duration: 3600 * 24 * 7, //7 days
    rewards_distribution,
    ticket_price: 10 * 1000000,
    rewards_expiry_duration: 3888000, // 45 days
    common_divisor,
    total_admin_share: (10 * common_divisor) / 100,
    shade_percentage_share: (60 * common_divisor) / 100,
    galactic_pools_percentage_share: (40 * common_divisor) / 100,
    shade_rewards_address: "secret1zrpgrff3p2pc3ptqu07pz9nq4ezhjr9xauyhjf",
    galactic_pools_rewards_address:
      "secret1pa8ng8z5lwcukvqht83r5pu2zfq5vv3zdu36uu",
    reserve_percentage: (60 * common_divisor) / 100,
    is_sponosorship_admin_controlled: false,
    unbonding_batch_duration: 3600 * 24 * 3,
    minimum_deposit_amount: 1 * 1000000,
    grand_prize_address: "secret1pa8ng8z5lwcukvqht83r5pu2zfq5vv3zdu36uu",
  };

  const contract = await client
    .instantiate(codeId, init_msg, "breaking_test_1", {
      fee: 300_000,
    })
    .catch((err) => {
      throw new Error(`Could not instantiate contract: ${err}`);
    });
  const { contractAddress } = contract;
  console.log("contract: ", contract);

  //   // // 4. Query the counter

  //   // // 5. Increment the counter

  //   // Query again to confirm it worked
  //   console.log("Querying contract for updated count");
  //   response = await client
  //     .queryContractSmart(contractAddress, { lottery_info: {} })
  //     .catch((err) => {
  //       throw new Error(`Could not query contract: ${err}`);
  //     });

  //   console.log(`New Count=${response.lottery_info.start_height}`);
  // };

  // main().catch((err) => {
  //   console.error(err);
  // });

  // const wasm = fs.readFileSync(
  //   "/Users/haseebsaeed/codes/stakepool/contract.wasm"
  // );
  // // const wasm = fs.readFileSync(
  // //   "/Users/haseebsaeed/codes/sefi-testing/sefi-staking-testing/contract.wasm"
  // // );

  // console.log("Uploading contract");
  // const uploadReceipt = await client.upload(wasm, {}).catch((err) => {
  //   throw new Error(`Could not upload contract: ${err}`);
  // });

  // // 3. Create an instance of the Counter contract
  // // Get the code ID from the receipt
  // const { codeId } = uploadReceipt;

  // // Create an instance of the Counter contract, providing a starting count //change
  // const initMsg = {
  //   admin: "secret14v6h248vatcsur9hwqjekvj7t6jd8anf8ykw4n",
  //   triggerer: "secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7",
  //   denom: "uscrt",
  //   prng_seed: "ZW5pZ21hLXJvY2tzCg==",
  //   validator: "secretvaloper1xey4ymz4tmlgy6pp54e2ccj307ff6kx647p3hq",
  //   unbonding_period: 3600,
  // };

  // const contract = await client
  //   .instantiate(codeId, initMsg, "saucy_stakepool_v10")
  //   .catch((err) => {
  //     throw new Error(`Could not instantiate contract: ${err}`);
  //   });
  // const { contractAddress } = contract;
  // console.log("contract: ", contract);

  // // // 4. Query the counter

  // // // 5. Increment the counter

  // // Query again to confirm it worked
  // console.log("Querying contract for updated count");
  // response = await client
  //   .queryContractSmart(contractAddress, { lottery_info: {} })
  //   .catch((err) => {
  //     throw new Error(`Could not query contract: ${err}`);
  //   });

  // console.log(`New Count=${response.lottery_info.start_time}`);
};

main().catch((err) => {
  console.error(err);
});
