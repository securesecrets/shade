import { SecretNetworkClient } from "secretjs";
import * as LBRouter from "./types"
import { logGasToFile } from "../integration";
import { TokenType } from "../lb_factory";

export async function executeCreateLBPairUsingRouter(
    client: SecretNetworkClient,
    contractHashRouter: string, 
    contractAddressRouter: string,
    contractHashTokenA: string,
    contractAddressTokenA: string,
    contractHashTokenB: string,
    contractAddressTokenB: string,
    active_id: number,  // 8388608 is the middle bin
    bin_step: number,   // 100 represents a 1% bin step
  ) {
    const msg: LBRouter.CreateLBPairMsg = {
      create_lb_pair: {
        token_x: {
          custom_token: {
            contract_addr: contractAddressTokenA,
            token_code_hash: contractHashTokenA,
          }
        },
        token_y: {
          custom_token: {
            contract_addr: contractAddressTokenB,
            token_code_hash: contractHashTokenB,
          }
        },
        active_id: active_id,
        bin_step: bin_step,
      }
    }
  
    const tx = await client.tx.compute.executeContract(
      {
        sender: client.address,
        contract_address: contractAddressRouter,
        code_hash: contractHashRouter,
        msg: msg,
        sent_funds: [],
      },
      {
        gasLimit: 1_000_000,
      }
    );
  
    if (tx.code !== 0) {
      throw new Error(
        `Failed with the following error:\n ${tx.rawLog}`
      );
    };
  
    //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
    console.log(`CreateLBPair via Router TX used ${tx.gasUsed} gas`);
    logGasToFile(`CreateLBPair via Router TX used ${tx.gasUsed} gas`);
  }

  export async function executeSwapTokensForExact(
    client: SecretNetworkClient,
    contractHashRouter: string, 
    contractAddressRouter: string,
    contractHashPair: string,
    contractAddressPair: string,
    tokenX: TokenType,
    amount: string,
  ) {
    const tokenAmount: LBRouter.TokenAmount = {
      token: tokenX,
      amount: amount,
    }

    const hop: LBRouter.Hop = {
      addr: contractAddressPair,
      code_hash: contractHashPair,
    }

    const msg: LBRouter.SwapTokensForExactMsg = {
      swap_tokens_for_exact: {
        offer: tokenAmount,
        path: [hop]
      }
    }
  
    const tx = await client.tx.compute.executeContract(
      {
        sender: client.address,
        contract_address: contractAddressRouter,
        code_hash: contractHashRouter,
        msg: msg,
        sent_funds: [],
      },
      {
        gasLimit: 3_000_000,
      }
    );
  
    if (tx.code !== 0) {
      throw new Error(
        `Failed with the following error:\n ${tx.rawLog}`
      );
    };
  
    //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
    console.log(`SwapTokensforExact TX used ${tx.gasUsed} gas`);
    logGasToFile(`SwapTokensforExact TX used ${tx.gasUsed} gas`);
  }
  