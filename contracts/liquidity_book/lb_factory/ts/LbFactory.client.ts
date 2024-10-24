/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { AllBinStepsResponse, Addr, TokenType, AllLBPairsResponse, LBPairInformation, LBPair, ContractInfo, ExecuteMsg, RewardsDistributionAlgorithm, ContractInstantiationInfo, FeeRecipientResponse, InstantiateMsg, RawContract, IsQuoteAssetResponse, LBPairAtIndexResponse, LBPairImplementationResponse, LBPairInformationResponse, LBTokenImplementationResponse, MinBinStepResponse, NumberOfLBPairsResponse, NumberOfQuoteAssetsResponse, OpenBinStepsResponse, PresetResponse, QueryMsg, QuoteAssetAtIndexResponse } from "./LbFactory.types";
export interface LbFactoryReadOnlyInterface {
  contractAddress: string;
  getMinBinStep: () => Promise<GetMinBinStepResponse>;
  getFeeRecipient: () => Promise<GetFeeRecipientResponse>;
  getLbPairImplementation: () => Promise<GetLbPairImplementationResponse>;
  getLbTokenImplementation: () => Promise<GetLbTokenImplementationResponse>;
  getNumberOfLbPairs: () => Promise<GetNumberOfLbPairsResponse>;
  getLbPairAtIndex: ({
    index
  }: {
    index: number;
  }) => Promise<GetLbPairAtIndexResponse>;
  getNumberOfQuoteAssets: () => Promise<GetNumberOfQuoteAssetsResponse>;
  getQuoteAssetAtIndex: ({
    index
  }: {
    index: number;
  }) => Promise<GetQuoteAssetAtIndexResponse>;
  isQuoteAsset: ({
    token
  }: {
    token: TokenType;
  }) => Promise<IsQuoteAssetResponse>;
  getLbPairInformation: ({
    binStep,
    tokenX,
    tokenY
  }: {
    binStep: number;
    tokenX: TokenType;
    tokenY: TokenType;
  }) => Promise<GetLbPairInformationResponse>;
  getPreset: ({
    binStep
  }: {
    binStep: number;
  }) => Promise<GetPresetResponse>;
  getAllBinSteps: () => Promise<GetAllBinStepsResponse>;
  getOpenBinSteps: () => Promise<GetOpenBinStepsResponse>;
  getAllLbPairs: ({
    tokenX,
    tokenY
  }: {
    tokenX: TokenType;
    tokenY: TokenType;
  }) => Promise<GetAllLbPairsResponse>;
}
export class LbFactoryQueryClient implements LbFactoryReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.getMinBinStep = this.getMinBinStep.bind(this);
    this.getFeeRecipient = this.getFeeRecipient.bind(this);
    this.getLbPairImplementation = this.getLbPairImplementation.bind(this);
    this.getLbTokenImplementation = this.getLbTokenImplementation.bind(this);
    this.getNumberOfLbPairs = this.getNumberOfLbPairs.bind(this);
    this.getLbPairAtIndex = this.getLbPairAtIndex.bind(this);
    this.getNumberOfQuoteAssets = this.getNumberOfQuoteAssets.bind(this);
    this.getQuoteAssetAtIndex = this.getQuoteAssetAtIndex.bind(this);
    this.isQuoteAsset = this.isQuoteAsset.bind(this);
    this.getLbPairInformation = this.getLbPairInformation.bind(this);
    this.getPreset = this.getPreset.bind(this);
    this.getAllBinSteps = this.getAllBinSteps.bind(this);
    this.getOpenBinSteps = this.getOpenBinSteps.bind(this);
    this.getAllLbPairs = this.getAllLbPairs.bind(this);
  }

  getMinBinStep = async (): Promise<GetMinBinStepResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_min_bin_step: {}
    });
  };
  getFeeRecipient = async (): Promise<GetFeeRecipientResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_fee_recipient: {}
    });
  };
  getLbPairImplementation = async (): Promise<GetLbPairImplementationResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_lb_pair_implementation: {}
    });
  };
  getLbTokenImplementation = async (): Promise<GetLbTokenImplementationResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_lb_token_implementation: {}
    });
  };
  getNumberOfLbPairs = async (): Promise<GetNumberOfLbPairsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_number_of_lb_pairs: {}
    });
  };
  getLbPairAtIndex = async ({
    index
  }: {
    index: number;
  }): Promise<GetLbPairAtIndexResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_lb_pair_at_index: {
        index
      }
    });
  };
  getNumberOfQuoteAssets = async (): Promise<GetNumberOfQuoteAssetsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_number_of_quote_assets: {}
    });
  };
  getQuoteAssetAtIndex = async ({
    index
  }: {
    index: number;
  }): Promise<GetQuoteAssetAtIndexResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_quote_asset_at_index: {
        index
      }
    });
  };
  isQuoteAsset = async ({
    token
  }: {
    token: TokenType;
  }): Promise<IsQuoteAssetResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      is_quote_asset: {
        token
      }
    });
  };
  getLbPairInformation = async ({
    binStep,
    tokenX,
    tokenY
  }: {
    binStep: number;
    tokenX: TokenType;
    tokenY: TokenType;
  }): Promise<GetLbPairInformationResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_lb_pair_information: {
        bin_step: binStep,
        token_x: tokenX,
        token_y: tokenY
      }
    });
  };
  getPreset = async ({
    binStep
  }: {
    binStep: number;
  }): Promise<GetPresetResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_preset: {
        bin_step: binStep
      }
    });
  };
  getAllBinSteps = async (): Promise<GetAllBinStepsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_all_bin_steps: {}
    });
  };
  getOpenBinSteps = async (): Promise<GetOpenBinStepsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_open_bin_steps: {}
    });
  };
  getAllLbPairs = async ({
    tokenX,
    tokenY
  }: {
    tokenX: TokenType;
    tokenY: TokenType;
  }): Promise<GetAllLbPairsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_all_lb_pairs: {
        token_x: tokenX,
        token_y: tokenY
      }
    });
  };
}
export interface LbFactoryInterface extends LbFactoryReadOnlyInterface {
  contractAddress: string;
  sender: string;
  setLbPairImplementation: ({
    implementation
  }: {
    implementation: ContractInstantiationInfo;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setLbTokenImplementation: ({
    implementation
  }: {
    implementation: ContractInstantiationInfo;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setStakingContractImplementation: ({
    implementation
  }: {
    implementation: ContractInstantiationInfo;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  createLbPair: ({
    activeId,
    binStep,
    entropy,
    tokenX,
    tokenY,
    viewingKey
  }: {
    activeId: number;
    binStep: number;
    entropy: string;
    tokenX: TokenType;
    tokenY: TokenType;
    viewingKey: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setPairPreset: ({
    baseFactor,
    binStep,
    decayPeriod,
    epochStakingDuration,
    epochStakingIndex,
    expiryStakingDuration,
    filterPeriod,
    isOpen,
    maxVolatilityAccumulator,
    protocolShare,
    reductionFactor,
    rewardsDistributionAlgorithm,
    totalRewardBins,
    variableFeeControl
  }: {
    baseFactor: number;
    binStep: number;
    decayPeriod: number;
    epochStakingDuration: number;
    epochStakingIndex: number;
    expiryStakingDuration?: number;
    filterPeriod: number;
    isOpen: boolean;
    maxVolatilityAccumulator: number;
    protocolShare: number;
    reductionFactor: number;
    rewardsDistributionAlgorithm: RewardsDistributionAlgorithm;
    totalRewardBins: number;
    variableFeeControl: number;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setPresetOpenState: ({
    binStep,
    isOpen
  }: {
    binStep: number;
    isOpen: boolean;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  removePreset: ({
    binStep
  }: {
    binStep: number;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setFeeParametersOnPair: ({
    baseFactor,
    binStep,
    decayPeriod,
    filterPeriod,
    maxVolatilityAccumulator,
    protocolShare,
    reductionFactor,
    tokenX,
    tokenY,
    variableFeeControl
  }: {
    baseFactor: number;
    binStep: number;
    decayPeriod: number;
    filterPeriod: number;
    maxVolatilityAccumulator: number;
    protocolShare: number;
    reductionFactor: number;
    tokenX: TokenType;
    tokenY: TokenType;
    variableFeeControl: number;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setFeeRecipient: ({
    feeRecipient
  }: {
    feeRecipient: Addr;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  addQuoteAsset: ({
    asset
  }: {
    asset: TokenType;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  removeQuoteAsset: ({
    asset
  }: {
    asset: TokenType;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  forceDecay: ({
    pair
  }: {
    pair: LBPair;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class LbFactoryClient extends LbFactoryQueryClient implements LbFactoryInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.setLbPairImplementation = this.setLbPairImplementation.bind(this);
    this.setLbTokenImplementation = this.setLbTokenImplementation.bind(this);
    this.setStakingContractImplementation = this.setStakingContractImplementation.bind(this);
    this.createLbPair = this.createLbPair.bind(this);
    this.setPairPreset = this.setPairPreset.bind(this);
    this.setPresetOpenState = this.setPresetOpenState.bind(this);
    this.removePreset = this.removePreset.bind(this);
    this.setFeeParametersOnPair = this.setFeeParametersOnPair.bind(this);
    this.setFeeRecipient = this.setFeeRecipient.bind(this);
    this.addQuoteAsset = this.addQuoteAsset.bind(this);
    this.removeQuoteAsset = this.removeQuoteAsset.bind(this);
    this.forceDecay = this.forceDecay.bind(this);
  }

  setLbPairImplementation = async ({
    implementation
  }: {
    implementation: ContractInstantiationInfo;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_lb_pair_implementation: {
        implementation
      }
    }, fee, memo, _funds);
  };
  setLbTokenImplementation = async ({
    implementation
  }: {
    implementation: ContractInstantiationInfo;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_lb_token_implementation: {
        implementation
      }
    }, fee, memo, _funds);
  };
  setStakingContractImplementation = async ({
    implementation
  }: {
    implementation: ContractInstantiationInfo;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_staking_contract_implementation: {
        implementation
      }
    }, fee, memo, _funds);
  };
  createLbPair = async ({
    activeId,
    binStep,
    entropy,
    tokenX,
    tokenY,
    viewingKey
  }: {
    activeId: number;
    binStep: number;
    entropy: string;
    tokenX: TokenType;
    tokenY: TokenType;
    viewingKey: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      create_lb_pair: {
        active_id: activeId,
        bin_step: binStep,
        entropy,
        token_x: tokenX,
        token_y: tokenY,
        viewing_key: viewingKey
      }
    }, fee, memo, _funds);
  };
  setPairPreset = async ({
    baseFactor,
    binStep,
    decayPeriod,
    epochStakingDuration,
    epochStakingIndex,
    expiryStakingDuration,
    filterPeriod,
    isOpen,
    maxVolatilityAccumulator,
    protocolShare,
    reductionFactor,
    rewardsDistributionAlgorithm,
    totalRewardBins,
    variableFeeControl
  }: {
    baseFactor: number;
    binStep: number;
    decayPeriod: number;
    epochStakingDuration: number;
    epochStakingIndex: number;
    expiryStakingDuration?: number;
    filterPeriod: number;
    isOpen: boolean;
    maxVolatilityAccumulator: number;
    protocolShare: number;
    reductionFactor: number;
    rewardsDistributionAlgorithm: RewardsDistributionAlgorithm;
    totalRewardBins: number;
    variableFeeControl: number;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_pair_preset: {
        base_factor: baseFactor,
        bin_step: binStep,
        decay_period: decayPeriod,
        epoch_staking_duration: epochStakingDuration,
        epoch_staking_index: epochStakingIndex,
        expiry_staking_duration: expiryStakingDuration,
        filter_period: filterPeriod,
        is_open: isOpen,
        max_volatility_accumulator: maxVolatilityAccumulator,
        protocol_share: protocolShare,
        reduction_factor: reductionFactor,
        rewards_distribution_algorithm: rewardsDistributionAlgorithm,
        total_reward_bins: totalRewardBins,
        variable_fee_control: variableFeeControl
      }
    }, fee, memo, _funds);
  };
  setPresetOpenState = async ({
    binStep,
    isOpen
  }: {
    binStep: number;
    isOpen: boolean;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_preset_open_state: {
        bin_step: binStep,
        is_open: isOpen
      }
    }, fee, memo, _funds);
  };
  removePreset = async ({
    binStep
  }: {
    binStep: number;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      remove_preset: {
        bin_step: binStep
      }
    }, fee, memo, _funds);
  };
  setFeeParametersOnPair = async ({
    baseFactor,
    binStep,
    decayPeriod,
    filterPeriod,
    maxVolatilityAccumulator,
    protocolShare,
    reductionFactor,
    tokenX,
    tokenY,
    variableFeeControl
  }: {
    baseFactor: number;
    binStep: number;
    decayPeriod: number;
    filterPeriod: number;
    maxVolatilityAccumulator: number;
    protocolShare: number;
    reductionFactor: number;
    tokenX: TokenType;
    tokenY: TokenType;
    variableFeeControl: number;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_fee_parameters_on_pair: {
        base_factor: baseFactor,
        bin_step: binStep,
        decay_period: decayPeriod,
        filter_period: filterPeriod,
        max_volatility_accumulator: maxVolatilityAccumulator,
        protocol_share: protocolShare,
        reduction_factor: reductionFactor,
        token_x: tokenX,
        token_y: tokenY,
        variable_fee_control: variableFeeControl
      }
    }, fee, memo, _funds);
  };
  setFeeRecipient = async ({
    feeRecipient
  }: {
    feeRecipient: Addr;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_fee_recipient: {
        fee_recipient: feeRecipient
      }
    }, fee, memo, _funds);
  };
  addQuoteAsset = async ({
    asset
  }: {
    asset: TokenType;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      add_quote_asset: {
        asset
      }
    }, fee, memo, _funds);
  };
  removeQuoteAsset = async ({
    asset
  }: {
    asset: TokenType;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      remove_quote_asset: {
        asset
      }
    }, fee, memo, _funds);
  };
  forceDecay = async ({
    pair
  }: {
    pair: LBPair;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      force_decay: {
        pair
      }
    }, fee, memo, _funds);
  };
}