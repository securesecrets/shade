import contract_config from "./pool_contract/public_queries/contract_config.js";
import contract_status from "./pool_contract/public_queries/contract_status.js";

import current_rewards from "./pool_contract/public_queries/current_rewards.js";
import example from "./pool_contract/public_queries/example.js";
import lottery_info from "./pool_contract/public_queries/lottery_info.js";
import pool_info from "./pool_contract/public_queries/pool_info.js";
import pool_liq_stats from "./pool_contract/public_queries/pool_liquidity_stats.js";
import pool_liq_stats_specific from "./pool_contract/public_queries/pool_liquidity_stats_specific.js";
import sponsors from "./pool_contract/public_queries/sponsors.js";
import sponsors_msg_req_check from "./pool_contract/public_queries/sponsors_msg_req_check.js";
import reward_stats from "./pool_contract/public_queries/reward_stats.js";

import delegated from "./pool_contract/private_queries/delegated.js";
import withdrawable from "./pool_contract/private_queries/withdrawable.js";
import unbondings from "./pool_contract/private_queries/unbondings.js";
import records from "./pool_contract/private_queries/records.js";

import sscrt_balance from "./sscrt/sscrt_balance.js";

const args = process.argv.slice(2);

let passed = 0;
let failed = 0;
let starting_time = Date.now();
//running test

console.log("Running tests");
let total_test = 15;
console.log(
  "**************************************************Test**************************************************"
);
if (args.includes("--all") || args.includes("--test")) {
  analyse_status(await example());
  total_test -= 1;
}
console.log(
  "************************************************** Public **************************************************"
);

if (args.includes("--all") || args.includes("--contract_config")) {
  analyse_status(await contract_config());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--contract_status")) {
  analyse_status(await contract_status());
  total_test -= 1;
}

if (args.includes("--all") || args.includes("--current_rewards")) {
  analyse_status(await current_rewards());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--lottery_info")) {
  analyse_status(await lottery_info());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--pool_info")) {
  analyse_status(await pool_info());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--pool_liq_stats")) {
  analyse_status(await pool_liq_stats());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--pool_liq_stats_specific")) {
  analyse_status(await pool_liq_stats_specific());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--reward_stats")) {
  analyse_status(await reward_stats());
  total_test -= 1;
}

if (args.includes("--all") || args.includes("--sponsors")) {
  analyse_status(await sponsors());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--sponsors_msg_req_check")) {
  analyse_status(await sponsors_msg_req_check());
  total_test -= 1;
}

console.log(
  "************************************************** Private **************************************************"
);

if (args.includes("--all") || args.includes("--sscrt_balance")) {
  analyse_status(await sscrt_balance());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--delegated")) {
  analyse_status(await delegated());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--withdrawable")) {
  analyse_status(await withdrawable());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--unbondings")) {
  analyse_status(await unbondings());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--records")) {
  analyse_status(await records());
  total_test -= 1;
}

let passed_time = (Date.now() - starting_time) / 1000;
console.log(
  `test result: ok. ${passed} passed; ${failed} failed; ${total_test} ignored; 0 measured; 0 filtered out; finished in ${passed_time}s`
);

function analyse_status(status) {
  if (status === true) {
    passed += 1;
  } else {
    failed += 1;
  }
}
