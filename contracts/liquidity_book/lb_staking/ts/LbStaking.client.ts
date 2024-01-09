/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { ExecuteMsg, Uint256, Addr, Binary, Uint128, RewardsDistribution, Snip1155ReceiveMsg, Snip20ReceiveMsg, ContractInfo, RawContract, InstantiateMsg, QueryAnswer, TxAction, OwnerBalance, Liquidity, Tx, Reward, RewardToken, QueryMsg, QueryTxnType, TokenPermissions, QueryWithPermit, PermitForTokenPermissions, PermitParamsForTokenPermissions, PermitSignature, PubKey } from "./LbStaking.types";
export interface LbStakingReadOnlyInterface {
  contractAddress: string;
  contractInfo: () => Promise<ContractInfoResponse>;
  registeredTokens: () => Promise<RegisteredTokensResponse>;
  idTotalBalance: ({
    id
  }: {
    id: string;
  }) => Promise<IdTotalBalanceResponse>;
  balance: ({
    key,
    owner,
    tokenId
  }: {
    key: string;
    owner: Addr;
    tokenId: string;
  }) => Promise<BalanceResponse>;
  allBalances: ({
    key,
    owner,
    page,
    pageSize
  }: {
    key: string;
    owner: Addr;
    page?: number;
    pageSize?: number;
  }) => Promise<AllBalancesResponse>;
  liquidity: ({
    key,
    owner,
    roundIndex,
    tokenIds
  }: {
    key: string;
    owner: Addr;
    roundIndex?: number;
    tokenIds: number[];
  }) => Promise<LiquidityResponse>;
  transactionHistory: ({
    key,
    owner,
    page,
    pageSize,
    txnType
  }: {
    key: string;
    owner: Addr;
    page?: number;
    pageSize?: number;
    txnType: QueryTxnType;
  }) => Promise<TransactionHistoryResponse>;
  withPermit: ({
    permit,
    query
  }: {
    permit: PermitForTokenPermissions;
    query: QueryWithPermit;
  }) => Promise<WithPermitResponse>;
}
export class LbStakingQueryClient implements LbStakingReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.contractInfo = this.contractInfo.bind(this);
    this.registeredTokens = this.registeredTokens.bind(this);
    this.idTotalBalance = this.idTotalBalance.bind(this);
    this.balance = this.balance.bind(this);
    this.allBalances = this.allBalances.bind(this);
    this.liquidity = this.liquidity.bind(this);
    this.transactionHistory = this.transactionHistory.bind(this);
    this.withPermit = this.withPermit.bind(this);
  }

  contractInfo = async (): Promise<ContractInfoResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      contract_info: {}
    });
  };
  registeredTokens = async (): Promise<RegisteredTokensResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      registered_tokens: {}
    });
  };
  idTotalBalance = async ({
    id
  }: {
    id: string;
  }): Promise<IdTotalBalanceResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      id_total_balance: {
        id
      }
    });
  };
  balance = async ({
    key,
    owner,
    tokenId
  }: {
    key: string;
    owner: Addr;
    tokenId: string;
  }): Promise<BalanceResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      balance: {
        key,
        owner,
        token_id: tokenId
      }
    });
  };
  allBalances = async ({
    key,
    owner,
    page,
    pageSize
  }: {
    key: string;
    owner: Addr;
    page?: number;
    pageSize?: number;
  }): Promise<AllBalancesResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      all_balances: {
        key,
        owner,
        page,
        page_size: pageSize
      }
    });
  };
  liquidity = async ({
    key,
    owner,
    roundIndex,
    tokenIds
  }: {
    key: string;
    owner: Addr;
    roundIndex?: number;
    tokenIds: number[];
  }): Promise<LiquidityResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      liquidity: {
        key,
        owner,
        round_index: roundIndex,
        token_ids: tokenIds
      }
    });
  };
  transactionHistory = async ({
    key,
    owner,
    page,
    pageSize,
    txnType
  }: {
    key: string;
    owner: Addr;
    page?: number;
    pageSize?: number;
    txnType: QueryTxnType;
  }): Promise<TransactionHistoryResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      transaction_history: {
        key,
        owner,
        page,
        page_size: pageSize,
        txn_type: txnType
      }
    });
  };
  withPermit = async ({
    permit,
    query
  }: {
    permit: PermitForTokenPermissions;
    query: QueryWithPermit;
  }): Promise<WithPermitResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      with_permit: {
        permit,
        query
      }
    });
  };
}
export interface LbStakingInterface extends LbStakingReadOnlyInterface {
  contractAddress: string;
  sender: string;
  claimRewards: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  endEpoch: ({
    rewardsDistribution
  }: {
    rewardsDistribution: RewardsDistribution;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  unstake: ({
    amounts,
    tokenIds
  }: {
    amounts: Uint256[];
    tokenIds: number[];
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  snip1155Receive: ({
    amount,
    from,
    memo,
    msg,
    sender,
    tokenId
  }: {
    amount: Uint256;
    from: Addr;
    memo?: string;
    msg?: Binary;
    sender: Addr;
    tokenId: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  receive: ({
    amount,
    from,
    memo,
    msg,
    sender
  }: {
    amount: Uint128;
    from: string;
    memo?: string;
    msg?: Binary;
    sender: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  registerRewardTokens: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateConfig: ({
    adminAuth,
    epochDuration,
    expiryDuration,
    queryAuth
  }: {
    adminAuth?: RawContract;
    epochDuration?: number;
    expiryDuration?: number;
    queryAuth?: RawContract;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  recoverFunds: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  createViewingKey: ({
    entropy
  }: {
    entropy: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setViewingKey: ({
    key
  }: {
    key: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  revokePermit: ({
    permitName
  }: {
    permitName: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class LbStakingClient extends LbStakingQueryClient implements LbStakingInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.claimRewards = this.claimRewards.bind(this);
    this.endEpoch = this.endEpoch.bind(this);
    this.unstake = this.unstake.bind(this);
    this.snip1155Receive = this.snip1155Receive.bind(this);
    this.receive = this.receive.bind(this);
    this.registerRewardTokens = this.registerRewardTokens.bind(this);
    this.updateConfig = this.updateConfig.bind(this);
    this.recoverFunds = this.recoverFunds.bind(this);
    this.createViewingKey = this.createViewingKey.bind(this);
    this.setViewingKey = this.setViewingKey.bind(this);
    this.revokePermit = this.revokePermit.bind(this);
  }

  claimRewards = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      claim_rewards: {}
    }, fee, memo, _funds);
  };
  endEpoch = async ({
    rewardsDistribution
  }: {
    rewardsDistribution: RewardsDistribution;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      end_epoch: {
        rewards_distribution: rewardsDistribution
      }
    }, fee, memo, _funds);
  };
  unstake = async ({
    amounts,
    tokenIds
  }: {
    amounts: Uint256[];
    tokenIds: number[];
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      unstake: {
        amounts,
        token_ids: tokenIds
      }
    }, fee, memo, _funds);
  };
  snip1155Receive = async ({
    amount,
    from,
    memo,
    msg,
    sender,
    tokenId
  }: {
    amount: Uint256;
    from: Addr;
    memo?: string;
    msg?: Binary;
    sender: Addr;
    tokenId: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      snip1155_receive: {
        amount,
        from,
        memo,
        msg,
        sender,
        token_id: tokenId
      }
    }, fee, memo, _funds);
  };
  receive = async ({
    amount,
    from,
    memo,
    msg,
    sender
  }: {
    amount: Uint128;
    from: string;
    memo?: string;
    msg?: Binary;
    sender: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      receive: {
        amount,
        from,
        memo,
        msg,
        sender
      }
    }, fee, memo, _funds);
  };
  registerRewardTokens = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      register_reward_tokens: {}
    }, fee, memo, _funds);
  };
  updateConfig = async ({
    adminAuth,
    epochDuration,
    expiryDuration,
    queryAuth
  }: {
    adminAuth?: RawContract;
    epochDuration?: number;
    expiryDuration?: number;
    queryAuth?: RawContract;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_config: {
        admin_auth: adminAuth,
        epoch_duration: epochDuration,
        expiry_duration: expiryDuration,
        query_auth: queryAuth
      }
    }, fee, memo, _funds);
  };
  recoverFunds = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      recover_funds: {}
    }, fee, memo, _funds);
  };
  createViewingKey = async ({
    entropy
  }: {
    entropy: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      create_viewing_key: {
        entropy
      }
    }, fee, memo, _funds);
  };
  setViewingKey = async ({
    key
  }: {
    key: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_viewing_key: {
        key
      }
    }, fee, memo, _funds);
  };
  revokePermit = async ({
    permitName
  }: {
    permitName: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      revoke_permit: {
        permit_name: permitName
      }
    }, fee, memo, _funds);
  };
}