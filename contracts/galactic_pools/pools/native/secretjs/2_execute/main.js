import deposit from "./users/deposit.js";
import request_withdraw from "./users/request_withdraw.js";
import withdraw from "./users/withdraw.js";
import claim_rewards from "./users/claim_rewards.js";

import sponsor from "./sponsors/sponsor.js";
import sponsor_request_withdraw from "./sponsors/sponsor_request_withdraw.js";
import sponsor_withdraw from "./sponsors/sponsor_withdraw.js";

import end_lottery from "./admins/end_lottery.js";
import end_lottery_loop from "./admins/end_lottery_loop.js";
import unbond_batch_loop from "./admins/unbond_batch_loop.js";
import unbond_batch from "./admins/unbond_batch.js";
import rebalance_val_set from "./admins/rebalance_val_set.js";

import all_in_one from "./allinone.js";

const args = process.argv.slice(2);

let passed = 0;
let failed = 0;
let starting_time = Date.now();
//running test

console.log(args);

console.log("Running tests");
let total_test = 1;
console.log(
  "************************************************** User **************************************************"
);

if (args.includes("--all") || args.includes("--allinone")) {
  all_in_one();

  total_test -= 1;
}

if (args.includes("--all") || args.includes("--wrap")) {
  if (args.includes("--amount")) {
    analyse_status(await wrap(parseInt(args[2])));
  } else {
    analyse_status(await wrap());
  }
  total_test -= 1;
}

if (args.includes("--all") || args.includes("--deposit")) {
  if (args.includes("--amount")) {
    console.log(args);
    analyse_status(await deposit(parseInt(args[2])));
  } else {
    analyse_status(await deposit());
  }

  total_test -= 1;
}

if (args.includes("--all") || args.includes("--request_withdraw")) {
  if (args.includes("--amount")) {
    analyse_status(await request_withdraw(parseInt(args[2])));
  } else {
    analyse_status(await request_withdraw());
  }
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--withdraw")) {
  if (args.includes("--amount")) {
    analyse_status(await withdraw(parseInt(args[2])));
  } else {
    analyse_status(await withdraw());
  }
  total_test -= 1;
}

if (args.includes("--all") || args.includes("--claim_rewards")) {
  analyse_status(await claim_rewards());
  total_test -= 1;
}

console.log(
  "************************************************** Sponsor ************************************************** "
);

if (args.includes("--all") || args.includes("--sponsor")) {
  if (args.includes("--amount")) {
    analyse_status(await sponsor(parseInt(args[2])));
  } else {
    analyse_status(await sponsor());
  }

  total_test -= 1;
}

if (args.includes("--all") || args.includes("--sponsor_request_withdraw")) {
  if (args.includes("--amount")) {
    analyse_status(await sponsor_request_withdraw(parseInt(args[2])));
  } else {
    analyse_status(await sponsor_request_withdraw());
  }

  total_test -= 1;
}
if (args.includes("--all") || args.includes("--sponsor_withdraw")) {
  if (args.includes("--amount")) {
    analyse_status(await sponsor_withdraw(parseInt(args[2])));
  } else {
    analyse_status(await sponsor_withdraw());
  }

  total_test -= 1;
}

console.log(
  "************************************************** Admin ************************************************** "
);

if (args.includes("--all") || args.includes("--end_lottery")) {
  analyse_status(await end_lottery());
  total_test -= 1;
}

if (args.includes("--all") || args.includes("--end_lottery_loop")) {
  analyse_status(await end_lottery_loop());
  total_test -= 1;
}

if (args.includes("--all") || args.includes("--unbond_batch")) {
  analyse_status(await unbond_batch());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--unbond_batch_loop")) {
  analyse_status(await unbond_batch_loop());
  total_test -= 1;
}
if (args.includes("--all") || args.includes("--rebalance_val_set")) {
  analyse_status(await rebalance_val_set());
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
