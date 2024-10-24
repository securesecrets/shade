/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

import { Coin, StdFee } from "@cosmjs/amino";
import {
  CosmWasmClient,
  ExecuteResult,
  SigningCosmWasmClient,
} from "@cosmjs/cosmwasm-stargate";
import {
  Binary,
  Contract,
  Hop,
  TokenAmount,
  TokenType,
  Uint128,
} from "./Router.types";
export interface RouterReadOnlyInterface {
  contractAddress: string;
  swapSimulation: ({
    excludeFee,
    offer,
    path,
  }: {
    excludeFee?: boolean;
    offer: TokenAmount;
    path: Hop[];
  }) => Promise<SwapSimulationResponse>;
  getConfig: () => Promise<GetConfigResponse>;
  registeredTokens: () => Promise<RegisteredTokensResponse>;
}
export class RouterQueryClient implements RouterReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.swapSimulation = this.swapSimulation.bind(this);
    this.getConfig = this.getConfig.bind(this);
    this.registeredTokens = this.registeredTokens.bind(this);
  }

  swapSimulation = async ({
    excludeFee,
    offer,
    path,
  }: {
    excludeFee?: boolean;
    offer: TokenAmount;
    path: Hop[];
  }): Promise<SwapSimulationResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      swap_simulation: {
        exclude_fee: excludeFee,
        offer,
        path,
      },
    });
  };
  getConfig = async (): Promise<GetConfigResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_config: {},
    });
  };
  registeredTokens = async (): Promise<RegisteredTokensResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      registered_tokens: {},
    });
  };
}
export interface RouterInterface extends RouterReadOnlyInterface {
  contractAddress: string;
  sender: string;
  receive: (
    {
      amount,
      from,
      memo,
      msg,
      sender,
    }: {
      amount: Uint128;
      from: string;
      memo?: string;
      msg?: Binary;
      sender: string;
    },
    fee?: number | StdFee | "auto",
    memo?: string,
    _funds?: Coin[]
  ) => Promise<ExecuteResult>;
  swapTokensForExact: (
    {
      expectedReturn,
      offer,
      padding,
      path,
      recipient,
    }: {
      expectedReturn?: Uint128;
      offer: TokenAmount;
      padding?: string;
      path: Hop[];
      recipient?: string;
    },
    fee?: number | StdFee | "auto",
    memo?: string,
    _funds?: Coin[]
  ) => Promise<ExecuteResult>;
  registerSNIP20Token: (
    {
      oracleKey,
      padding,
      tokenAddr,
      tokenCodeHash,
    }: {
      oracleKey?: string;
      padding?: string;
      tokenAddr: string;
      tokenCodeHash: string;
    },
    fee?: number | StdFee | "auto",
    memo?: string,
    _funds?: Coin[]
  ) => Promise<ExecuteResult>;
  recoverFunds: (
    {
      amount,
      msg,
      padding,
      to,
      token,
    }: {
      amount: Uint128;
      msg?: Binary;
      padding?: string;
      to: string;
      token: TokenType;
    },
    fee?: number | StdFee | "auto",
    memo?: string,
    _funds?: Coin[]
  ) => Promise<ExecuteResult>;
  setConfig: (
    {
      adminAuth,
      padding,
    }: {
      adminAuth?: Contract;
      padding?: string;
    },
    fee?: number | StdFee | "auto",
    memo?: string,
    _funds?: Coin[]
  ) => Promise<ExecuteResult>;
}
export class RouterClient extends RouterQueryClient implements RouterInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(
    client: SigningCosmWasmClient,
    sender: string,
    contractAddress: string
  ) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.receive = this.receive.bind(this);
    this.swapTokensForExact = this.swapTokensForExact.bind(this);
    this.registerSNIP20Token = this.registerSNIP20Token.bind(this);
    this.recoverFunds = this.recoverFunds.bind(this);
    this.setConfig = this.setConfig.bind(this);
  }

  receive = async (
    {
      amount,
      from,
      memo,
      msg,
      sender,
    }: {
      amount: Uint128;
      from: string;
      memo?: string;
      msg?: Binary;
      sender: string;
    },
    fee: number | StdFee | "auto" = "auto",
    memo?: string,
    _funds?: Coin[]
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        receive: {
          amount,
          from,
          memo,
          msg,
          sender,
        },
      },
      fee,
      memo,
      _funds
    );
  };
  swapTokensForExact = async (
    {
      expectedReturn,
      offer,
      padding,
      path,
      recipient,
    }: {
      expectedReturn?: Uint128;
      offer: TokenAmount;
      padding?: string;
      path: Hop[];
      recipient?: string;
    },
    fee: number | StdFee | "auto" = "auto",
    memo?: string,
    _funds?: Coin[]
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        swap_tokens_for_exact: {
          expected_return: expectedReturn,
          offer,
          padding,
          path,
          recipient,
        },
      },
      fee,
      memo,
      _funds
    );
  };
  registerSNIP20Token = async (
    {
      oracleKey,
      padding,
      tokenAddr,
      tokenCodeHash,
    }: {
      oracleKey?: string;
      padding?: string;
      tokenAddr: string;
      tokenCodeHash: string;
    },
    fee: number | StdFee | "auto" = "auto",
    memo?: string,
    _funds?: Coin[]
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        register_s_n_i_p20_token: {
          oracle_key: oracleKey,
          padding,
          token_addr: tokenAddr,
          token_code_hash: tokenCodeHash,
        },
      },
      fee,
      memo,
      _funds
    );
  };
  recoverFunds = async (
    {
      amount,
      msg,
      padding,
      to,
      token,
    }: {
      amount: Uint128;
      msg?: Binary;
      padding?: string;
      to: string;
      token: TokenType;
    },
    fee: number | StdFee | "auto" = "auto",
    memo?: string,
    _funds?: Coin[]
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        recover_funds: {
          amount,
          msg,
          padding,
          to,
          token,
        },
      },
      fee,
      memo,
      _funds
    );
  };
  setConfig = async (
    {
      adminAuth,
      padding,
    }: {
      adminAuth?: Contract;
      padding?: string;
    },
    fee: number | StdFee | "auto" = "auto",
    memo?: string,
    _funds?: Coin[]
  ): Promise<ExecuteResult> => {
    return await this.client.execute(
      this.sender,
      this.contractAddress,
      {
        set_config: {
          admin_auth: adminAuth,
          padding,
        },
      },
      fee,
      memo,
      _funds
    );
  };
}