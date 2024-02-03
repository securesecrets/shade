import { SecretNetworkClient, Wallet } from "secretjs";
import * as fs from "fs";
import dotenv from "dotenv";
dotenv.config();

const wallet = new Wallet(process.env.MNEMONIC);

const lend_token = fs.readFileSync("lend_token.wasm.gz");
const lend_market = fs.readFileSync("lend_market.wasm.gz");
const credit_agency = fs.readFileSync("credit_agency.wasm.gz");

const query_auth = "secret1e0k5jza9jqctc5dt7mltnxmwpu3a3kqe0a6hf3";
const oracle = "secret17z47r9u4nqytpdgvewxq4jqd965sfj2wpsnlak";

const secretjs = new SecretNetworkClient({
  chainId: "pulsar-3",
  url: "https://api.pulsar.scrttestnet.com",
  wallet: wallet,
  walletAddress: wallet.address,
});

let instantiate_contracts = async () => {
  let lend_token_tx = await secretjs.tx.compute.storeCode(
    {
      sender: wallet.address,
      wasm_byte_code: lend_token,
      source: "",
      builder: "",
    },
    {
      gasLimit: 4_000_000,
    }
  );

  const lendTokenCodeId = Number(
    lend_token_tx.arrayLog.find((log) => log.type === "message" && log.key === "code_id")
      .value
  );
  console.log("Lend token codeId: ", lendTokenCodeId);
  const lendTokenCodeHash = (
    await secretjs.query.compute.codeHashByCodeId({ code_id: lendTokenCodeId })
  ).code_hash;
  console.log(`Lend Token hash: ${lendTokenCodeHash}`);

  let lend_market_tx = await secretjs.tx.compute.storeCode(
    {
      sender: wallet.address,
      wasm_byte_code: lend_market,
      source: "",
      builder: "",
    },
    {
      gasLimit: 4_000_000,
    }
  );

  const lendMarketCodeId = Number(
    lend_market_tx.arrayLog.find((log) => log.type === "message" && log.key === "code_id")
      .value
  );
  console.log("Lend market codeId: ", lendMarketCodeId);
  const lendMarketCodeHash = (
    await secretjs.query.compute.codeHashByCodeId({ code_id: lendMarketCodeId })
  ).code_hash;
  console.log(`Lend Market hash: ${lendMarketCodeHash}`);

  let credit_agency_tx = await secretjs.tx.compute.storeCode(
    {
      sender: wallet.address,
      wasm_byte_code: credit_agency,
      source: "",
      builder: "",
    },
    {
      gasLimit: 4_000_000,
    }
  );

  const creditAgencyCodeId = Number(
    credit_agency_tx.arrayLog.find((log) => log.type === "message" && log.key === "code_id")
      .value
  );
  console.log("Credit Agency codeId: ", creditAgencyCodeId);
  const creditAgencyCodeHash = (
    await secretjs.query.compute.codeHashByCodeId({ code_id: creditAgencyCodeId })
  ).code_hash;
  console.log(`Credit Agency hash: ${creditAgencyCodeHash}`);

  const queryAuthCodeHash = (
    await secretjs.query.compute.codeHashByContractAddress({ contract_address: query_auth })
  ).code_hash;
  console.log(`Query Auth hash: ${queryAuthCodeHash}`);

  const ca_initMsg = {
      gov_contract: {
        address: wallet.address,
        code_hash: "1"
      },
      query_auth: {
        address: "secret1e0k5jza9jqctc5dt7mltnxmwpu3a3kqe0a6hf3",
        code_hash: queryAuthCodeHash
      },
      lend_market_id: lendMarketCodeId,
      lend_market_code_hash: lendMarketCodeHash,
      market_viewing_key: "key",
      ctoken_token_id: lendTokenCodeId,
      ctoken_code_hash: lendTokenCodeHash,
      reward_token: {
        Cw20: {
          address: "secret1e0k5jza9jqctc5dt7mltnxmwpu3a3kqe0a6hf3",
          code_hash: "1"
        }
      },
      common_token: {
        Cw20: {
          address: "secret1e0k5jza9jqctc5dt7mltnxmwpu3a3kqe0a6hf3",
          code_hash: "1"
        }
      },
      liquidation_price: "0.92",
      liquidation_threshold: "0.02",
      borrow_limit_ratio: "0.01",
      default_estimate_multiplier: "1"
    };


  let caInstantiateTx = await secretjs.tx.compute.instantiateContract(
    {
      code_id: creditAgencyCodeId,
      sender: wallet.address,
      code_hash: creditAgencyCodeHash,
      init_msg: ca_initMsg,
      label: "Credit Agency" + Math.random(),
    },
    {
      gasLimit: 400_000,
    }
  );

  // console.log(`INIT caInstantiateTx: ${JSON.stringify(caInstantiateTx, null, 2)}\n`);

  const contractAddress = caInstantiateTx.arrayLog.find(
    (log) => log.type === "message" && log.key === "contract_address"
  ).value;

  console.log(`Credit Agency contract address: ${contractAddress}`);

  // -------------------------------------------------------------

  const oracleCodeHash = (
    await secretjs.query.compute.codeHashByContractAddress({ contract_address: oracle })
  ).code_hash;
  console.log(`Oracle hash: ${oracleCodeHash}`);

  const createMarketMsg = {
    create_market: {
      name: "FRST",
      symbol: "FRS",
      decimals: 6,
      market_token: {
        Cw20: {
          address: "secret1e0k5jza9jqctc5dt7mltnxmwpu3a3kqe0a6hf3",
          code_hash: "1"
        }
      },
      market_cap: null,
      interest_rate: {
        linear: {
          base: "0.02",
          slope: "0.1"
        }
      },
      interest_charge_period: 3600,
      collateral_ratio: "0.8",
      price_oracle: {
        address: oracle,
        code_hash: oracleCodeHash
      },
      reserve_factor: "0.1"
    }
  };

  let createMarketTx = await secretjs.tx.compute.executeContract(
    {
      sender: wallet.address,
      contract_address: contractAddress,
      code_hash: creditAgencyCodeHash,
      msg: createMarketMsg,
      sent_funds: [],
    },
    {
      gasLimit: 1_000_000,
    },
  );

  console.log(`Create market: ${JSON.stringify(createMarketTx, null, 2)}\n`);

};

instantiate_contracts();
