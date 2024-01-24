/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { ActiveIdResponse, AllBinsResponse, BinResponse, BinStepResponse, BinUpdatingHeightsResponse, BinsResponse, ExecuteMsg, Uint128, TokenType, Addr, Binary, Uint256, RewardsDistributionAlgorithm, ContractStatus, TokenAmount, Snip20ReceiveMsg, LiquidityParameters, RemoveLiquidity, FactoryResponse, Decimal256, GetPairInfoResponse, ContractInfo, FeeInfo, Fee, TokenPair, StablePairInfoResponse, StableParams, CustomIterationControls, Contract, StableTokenData, IdFromPriceResponse, InstantiateMsg, RawContract, ContractInstantiationInfo, StaticFeeParameters, InvokeMsg, LbTokenResponse, MintResponse, NextNonEmptyBinResponse, OracleParametersResponse, OracleSampleAtResponse, PriceFromIdResponse, ProtocolFeesResponse, QueryMsg, ReservesResponse, RewardsDistributionResponse, RewardsDistribution, StakingResponse, StaticFeeParametersResponse, SwapInResponse, SwapOutResponse, SwapSimulationResponse, SwapResult, TokenXResponse, TokenYResponse, TokensResponse, TotalSupplyResponse, UpdatedBinsAfterHeightResponse, UpdatedBinsAtHeightResponse, UpdatedBinsAtMultipleHeightResponse, VariableFeeParametersResponse } from "./Sg721.types";
export interface Sg721ReadOnlyInterface {
  contractAddress: string;
  getStakingContract: () => Promise<GetStakingContractResponse>;
  getLbToken: () => Promise<GetLbTokenResponse>;
  getPairInfo: () => Promise<GetPairInfoResponse>;
  swapSimulation: ({
    excludeFee,
    offer
  }: {
    excludeFee?: boolean;
    offer: TokenAmount;
  }) => Promise<SwapSimulationResponse>;
  getFactory: () => Promise<GetFactoryResponse>;
  getTokens: () => Promise<GetTokensResponse>;
  getTokenX: () => Promise<GetTokenXResponse>;
  getTokenY: () => Promise<GetTokenYResponse>;
  getBinStep: () => Promise<GetBinStepResponse>;
  getReserves: () => Promise<GetReservesResponse>;
  getActiveId: () => Promise<GetActiveIdResponse>;
  getBinReserves: ({
    id
  }: {
    id: number;
  }) => Promise<GetBinReservesResponse>;
  getBinsReserves: ({
    ids
  }: {
    ids: number[];
  }) => Promise<GetBinsReservesResponse>;
  getAllBinsReserves: ({
    id,
    page,
    pageSize
  }: {
    id?: number;
    page?: number;
    pageSize?: number;
  }) => Promise<GetAllBinsReservesResponse>;
  getUpdatedBinAtHeight: ({
    height
  }: {
    height: number;
  }) => Promise<GetUpdatedBinAtHeightResponse>;
  getUpdatedBinAtMultipleHeights: ({
    heights
  }: {
    heights: number[];
  }) => Promise<GetUpdatedBinAtMultipleHeightsResponse>;
  getUpdatedBinAfterHeight: ({
    height,
    page,
    pageSize
  }: {
    height: number;
    page?: number;
    pageSize?: number;
  }) => Promise<GetUpdatedBinAfterHeightResponse>;
  getBinUpdatingHeights: ({
    page,
    pageSize
  }: {
    page?: number;
    pageSize?: number;
  }) => Promise<GetBinUpdatingHeightsResponse>;
  getNextNonEmptyBin: ({
    id,
    swapForY
  }: {
    id: number;
    swapForY: boolean;
  }) => Promise<GetNextNonEmptyBinResponse>;
  getProtocolFees: () => Promise<GetProtocolFeesResponse>;
  getStaticFeeParameters: () => Promise<GetStaticFeeParametersResponse>;
  getVariableFeeParameters: () => Promise<GetVariableFeeParametersResponse>;
  getOracleParameters: () => Promise<GetOracleParametersResponse>;
  getOracleSampleAt: ({
    lookUpTimestamp
  }: {
    lookUpTimestamp: number;
  }) => Promise<GetOracleSampleAtResponse>;
  getPriceFromId: ({
    id
  }: {
    id: number;
  }) => Promise<GetPriceFromIdResponse>;
  getIdFromPrice: ({
    price
  }: {
    price: Uint256;
  }) => Promise<GetIdFromPriceResponse>;
  getSwapIn: ({
    amountOut,
    swapForY
  }: {
    amountOut: Uint128;
    swapForY: boolean;
  }) => Promise<GetSwapInResponse>;
  getSwapOut: ({
    amountIn,
    swapForY
  }: {
    amountIn: Uint128;
    swapForY: boolean;
  }) => Promise<GetSwapOutResponse>;
  totalSupply: ({
    id
  }: {
    id: number;
  }) => Promise<TotalSupplyResponse>;
  getRewardsDistribution: ({
    epochId
  }: {
    epochId?: number;
  }) => Promise<GetRewardsDistributionResponse>;
}
export class Sg721QueryClient implements Sg721ReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.getStakingContract = this.getStakingContract.bind(this);
    this.getLbToken = this.getLbToken.bind(this);
    this.getPairInfo = this.getPairInfo.bind(this);
    this.swapSimulation = this.swapSimulation.bind(this);
    this.getFactory = this.getFactory.bind(this);
    this.getTokens = this.getTokens.bind(this);
    this.getTokenX = this.getTokenX.bind(this);
    this.getTokenY = this.getTokenY.bind(this);
    this.getBinStep = this.getBinStep.bind(this);
    this.getReserves = this.getReserves.bind(this);
    this.getActiveId = this.getActiveId.bind(this);
    this.getBinReserves = this.getBinReserves.bind(this);
    this.getBinsReserves = this.getBinsReserves.bind(this);
    this.getAllBinsReserves = this.getAllBinsReserves.bind(this);
    this.getUpdatedBinAtHeight = this.getUpdatedBinAtHeight.bind(this);
    this.getUpdatedBinAtMultipleHeights = this.getUpdatedBinAtMultipleHeights.bind(this);
    this.getUpdatedBinAfterHeight = this.getUpdatedBinAfterHeight.bind(this);
    this.getBinUpdatingHeights = this.getBinUpdatingHeights.bind(this);
    this.getNextNonEmptyBin = this.getNextNonEmptyBin.bind(this);
    this.getProtocolFees = this.getProtocolFees.bind(this);
    this.getStaticFeeParameters = this.getStaticFeeParameters.bind(this);
    this.getVariableFeeParameters = this.getVariableFeeParameters.bind(this);
    this.getOracleParameters = this.getOracleParameters.bind(this);
    this.getOracleSampleAt = this.getOracleSampleAt.bind(this);
    this.getPriceFromId = this.getPriceFromId.bind(this);
    this.getIdFromPrice = this.getIdFromPrice.bind(this);
    this.getSwapIn = this.getSwapIn.bind(this);
    this.getSwapOut = this.getSwapOut.bind(this);
    this.totalSupply = this.totalSupply.bind(this);
    this.getRewardsDistribution = this.getRewardsDistribution.bind(this);
  }

  getStakingContract = async (): Promise<GetStakingContractResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_staking_contract: {}
    });
  };
  getLbToken = async (): Promise<GetLbTokenResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_lb_token: {}
    });
  };
  getPairInfo = async (): Promise<GetPairInfoResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_pair_info: {}
    });
  };
  swapSimulation = async ({
    excludeFee,
    offer
  }: {
    excludeFee?: boolean;
    offer: TokenAmount;
  }): Promise<SwapSimulationResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      swap_simulation: {
        exclude_fee: excludeFee,
        offer
      }
    });
  };
  getFactory = async (): Promise<GetFactoryResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_factory: {}
    });
  };
  getTokens = async (): Promise<GetTokensResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_tokens: {}
    });
  };
  getTokenX = async (): Promise<GetTokenXResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_token_x: {}
    });
  };
  getTokenY = async (): Promise<GetTokenYResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_token_y: {}
    });
  };
  getBinStep = async (): Promise<GetBinStepResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_bin_step: {}
    });
  };
  getReserves = async (): Promise<GetReservesResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_reserves: {}
    });
  };
  getActiveId = async (): Promise<GetActiveIdResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_active_id: {}
    });
  };
  getBinReserves = async ({
    id
  }: {
    id: number;
  }): Promise<GetBinReservesResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_bin_reserves: {
        id
      }
    });
  };
  getBinsReserves = async ({
    ids
  }: {
    ids: number[];
  }): Promise<GetBinsReservesResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_bins_reserves: {
        ids
      }
    });
  };
  getAllBinsReserves = async ({
    id,
    page,
    pageSize
  }: {
    id?: number;
    page?: number;
    pageSize?: number;
  }): Promise<GetAllBinsReservesResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_all_bins_reserves: {
        id,
        page,
        page_size: pageSize
      }
    });
  };
  getUpdatedBinAtHeight = async ({
    height
  }: {
    height: number;
  }): Promise<GetUpdatedBinAtHeightResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_updated_bin_at_height: {
        height
      }
    });
  };
  getUpdatedBinAtMultipleHeights = async ({
    heights
  }: {
    heights: number[];
  }): Promise<GetUpdatedBinAtMultipleHeightsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_updated_bin_at_multiple_heights: {
        heights
      }
    });
  };
  getUpdatedBinAfterHeight = async ({
    height,
    page,
    pageSize
  }: {
    height: number;
    page?: number;
    pageSize?: number;
  }): Promise<GetUpdatedBinAfterHeightResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_updated_bin_after_height: {
        height,
        page,
        page_size: pageSize
      }
    });
  };
  getBinUpdatingHeights = async ({
    page,
    pageSize
  }: {
    page?: number;
    pageSize?: number;
  }): Promise<GetBinUpdatingHeightsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_bin_updating_heights: {
        page,
        page_size: pageSize
      }
    });
  };
  getNextNonEmptyBin = async ({
    id,
    swapForY
  }: {
    id: number;
    swapForY: boolean;
  }): Promise<GetNextNonEmptyBinResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_next_non_empty_bin: {
        id,
        swap_for_y: swapForY
      }
    });
  };
  getProtocolFees = async (): Promise<GetProtocolFeesResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_protocol_fees: {}
    });
  };
  getStaticFeeParameters = async (): Promise<GetStaticFeeParametersResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_static_fee_parameters: {}
    });
  };
  getVariableFeeParameters = async (): Promise<GetVariableFeeParametersResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_variable_fee_parameters: {}
    });
  };
  getOracleParameters = async (): Promise<GetOracleParametersResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_oracle_parameters: {}
    });
  };
  getOracleSampleAt = async ({
    lookUpTimestamp
  }: {
    lookUpTimestamp: number;
  }): Promise<GetOracleSampleAtResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_oracle_sample_at: {
        look_up_timestamp: lookUpTimestamp
      }
    });
  };
  getPriceFromId = async ({
    id
  }: {
    id: number;
  }): Promise<GetPriceFromIdResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_price_from_id: {
        id
      }
    });
  };
  getIdFromPrice = async ({
    price
  }: {
    price: Uint256;
  }): Promise<GetIdFromPriceResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_id_from_price: {
        price
      }
    });
  };
  getSwapIn = async ({
    amountOut,
    swapForY
  }: {
    amountOut: Uint128;
    swapForY: boolean;
  }): Promise<GetSwapInResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_swap_in: {
        amount_out: amountOut,
        swap_for_y: swapForY
      }
    });
  };
  getSwapOut = async ({
    amountIn,
    swapForY
  }: {
    amountIn: Uint128;
    swapForY: boolean;
  }): Promise<GetSwapOutResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_swap_out: {
        amount_in: amountIn,
        swap_for_y: swapForY
      }
    });
  };
  totalSupply = async ({
    id
  }: {
    id: number;
  }): Promise<TotalSupplyResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      total_supply: {
        id
      }
    });
  };
  getRewardsDistribution = async ({
    epochId
  }: {
    epochId?: number;
  }): Promise<GetRewardsDistributionResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_rewards_distribution: {
        epoch_id: epochId
      }
    });
  };
}
export interface Sg721Interface extends Sg721ReadOnlyInterface {
  contractAddress: string;
  sender: string;
  swapTokens: ({
    expectedReturn,
    offer,
    padding,
    to
  }: {
    expectedReturn?: Uint128;
    offer: TokenAmount;
    padding?: string;
    to?: string;
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
  addLiquidity: ({
    liquidityParameters
  }: {
    liquidityParameters: LiquidityParameters;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  removeLiquidity: ({
    removeLiquidityParams
  }: {
    removeLiquidityParams: RemoveLiquidity;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  collectProtocolFees: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  increaseOracleLength: ({
    newLength
  }: {
    newLength: number;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setStaticFeeParameters: ({
    baseFactor,
    decayPeriod,
    filterPeriod,
    maxVolatilityAccumulator,
    protocolShare,
    reductionFactor,
    variableFeeControl
  }: {
    baseFactor: number;
    decayPeriod: number;
    filterPeriod: number;
    maxVolatilityAccumulator: number;
    protocolShare: number;
    reductionFactor: number;
    variableFeeControl: number;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  forceDecay: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  calculateRewards: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  resetRewardsConfig: ({
    baseRewardsBins,
    distribution
  }: {
    baseRewardsBins?: number;
    distribution?: RewardsDistributionAlgorithm;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setContractStatus: ({
    contractStatus
  }: {
    contractStatus: ContractStatus;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class Sg721Client extends Sg721QueryClient implements Sg721Interface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.swapTokens = this.swapTokens.bind(this);
    this.receive = this.receive.bind(this);
    this.addLiquidity = this.addLiquidity.bind(this);
    this.removeLiquidity = this.removeLiquidity.bind(this);
    this.collectProtocolFees = this.collectProtocolFees.bind(this);
    this.increaseOracleLength = this.increaseOracleLength.bind(this);
    this.setStaticFeeParameters = this.setStaticFeeParameters.bind(this);
    this.forceDecay = this.forceDecay.bind(this);
    this.calculateRewards = this.calculateRewards.bind(this);
    this.resetRewardsConfig = this.resetRewardsConfig.bind(this);
    this.setContractStatus = this.setContractStatus.bind(this);
  }

  swapTokens = async ({
    expectedReturn,
    offer,
    padding,
    to
  }: {
    expectedReturn?: Uint128;
    offer: TokenAmount;
    padding?: string;
    to?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      swap_tokens: {
        expected_return: expectedReturn,
        offer,
        padding,
        to
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
  addLiquidity = async ({
    liquidityParameters
  }: {
    liquidityParameters: LiquidityParameters;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      add_liquidity: {
        liquidity_parameters: liquidityParameters
      }
    }, fee, memo, _funds);
  };
  removeLiquidity = async ({
    removeLiquidityParams
  }: {
    removeLiquidityParams: RemoveLiquidity;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      remove_liquidity: {
        remove_liquidity_params: removeLiquidityParams
      }
    }, fee, memo, _funds);
  };
  collectProtocolFees = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      collect_protocol_fees: {}
    }, fee, memo, _funds);
  };
  increaseOracleLength = async ({
    newLength
  }: {
    newLength: number;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      increase_oracle_length: {
        new_length: newLength
      }
    }, fee, memo, _funds);
  };
  setStaticFeeParameters = async ({
    baseFactor,
    decayPeriod,
    filterPeriod,
    maxVolatilityAccumulator,
    protocolShare,
    reductionFactor,
    variableFeeControl
  }: {
    baseFactor: number;
    decayPeriod: number;
    filterPeriod: number;
    maxVolatilityAccumulator: number;
    protocolShare: number;
    reductionFactor: number;
    variableFeeControl: number;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_static_fee_parameters: {
        base_factor: baseFactor,
        decay_period: decayPeriod,
        filter_period: filterPeriod,
        max_volatility_accumulator: maxVolatilityAccumulator,
        protocol_share: protocolShare,
        reduction_factor: reductionFactor,
        variable_fee_control: variableFeeControl
      }
    }, fee, memo, _funds);
  };
  forceDecay = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      force_decay: {}
    }, fee, memo, _funds);
  };
  calculateRewards = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      calculate_rewards: {}
    }, fee, memo, _funds);
  };
  resetRewardsConfig = async ({
    baseRewardsBins,
    distribution
  }: {
    baseRewardsBins?: number;
    distribution?: RewardsDistributionAlgorithm;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      reset_rewards_config: {
        base_rewards_bins: baseRewardsBins,
        distribution
      }
    }, fee, memo, _funds);
  };
  setContractStatus = async ({
    contractStatus
  }: {
    contractStatus: ContractStatus;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_contract_status: {
        contract_status: contractStatus
      }
    }, fee, memo, _funds);
  };
}