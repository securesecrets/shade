import pkg from "secretjs";
const { SecretNetworkClient, grpc } = pkg;
const grpcWebUrl = "https://grpc.testnet.secretsaturn.net/";
import fs from "fs";
import { debug } from "console";

async function config() {
  // To create a readonly secret.js client, just pass in a gRPC-web endpoint
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId: "pulsar-2",
  });

  let bufferData = fs.readFileSync("../contract_details.json");
  let stData = bufferData.toString();
  let data = JSON.parse(stData);

  const contract = data.address;

  const hash = data.hash;

  let round_obj = await secretjs.query.compute.queryContract({
    contractAddress: contract,
    codeHash: hash, // optional but way faster
    query: { round: {} },
  });

  // debug(round_obj)

  console.log(round_obj.end_time);
  console.log(Number(Date.now() / 1000));

  let time_left = 0;
  if (Date.now() / 1000 < round_obj.end_time) {
    time_left = parseInt(
      Number(round_obj.end_time - Number(Date.now() / 1000))
    );
  }
  console.log(
    "-------------------------------- Round Info Start --------------------------------"
  );

  console.log(`duration: ${round_obj.duration}`);
  console.log(round_obj.rewards_distribution);
  console.log(`current_round_index: ${round_obj.current_round_index}`);
  console.log(`ticket_price: ${round_obj.ticket_price}`);
  console.log(`rewards_expiry_duration: ${round_obj.rewards_expiry_duration}`);
  console.log(round_obj.admin_share);
  console.log(
    `triggerer_share_percentage: ${round_obj.triggerer_share_percentage}`
  );
  console.log(round_obj.unclaimed_distribution);

  console.log("Time left " + time_left + " for round");

  return true;
}

export default config;
