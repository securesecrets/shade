/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

export interface AllBinStepsResponse {
  bin_step_with_preset: number[];
}
export type Addr = string;
export type TokenType =
  | {
      custom_token: {
        contract_addr: Addr;
        token_code_hash: string;
        [k: string]: unknown;
      };
    }
  | {
      native_token: {
        denom: string;
        [k: string]: unknown;
      };
    };
export interface AllLBPairsResponse {
  lb_pairs_available: LBPairInformation[];
}
export interface LBPairInformation {
  bin_step: number;
  created_by_owner: boolean;
  ignored_for_routing: boolean;
  lb_pair: LBPair;
}
export interface LBPair {
  bin_step: number;
  contract: ContractInfo;
  token_x: TokenType;
  token_y: TokenType;
}
export interface ContractInfo {
  address: Addr;
  code_hash?: string;
  [k: string]: unknown;
}
export type ExecuteMsg =
  | {
      set_lb_pair_implementation: {
        implementation: ContractInstantiationInfo;
      };
    }
  | {
      set_lb_token_implementation: {
        implementation: ContractInstantiationInfo;
      };
    }
  | {
      set_staking_contract_implementation: {
        implementation: ContractInstantiationInfo;
      };
    }
  | {
      create_lb_pair: {
        active_id: number;
        bin_step: number;
        entropy: string;
        token_x: TokenType;
        token_y: TokenType;
        viewing_key: string;
      };
    }
  | {
      set_pair_preset: {
        base_factor: number;
        bin_step: number;
        decay_period: number;
        epoch_staking_duration: number;
        epoch_staking_index: number;
        expiry_staking_duration?: number | null;
        filter_period: number;
        is_open: boolean;
        max_volatility_accumulator: number;
        protocol_share: number;
        reduction_factor: number;
        rewards_distribution_algorithm: RewardsDistributionAlgorithm;
        total_reward_bins: number;
        variable_fee_control: number;
      };
    }
  | {
      set_preset_open_state: {
        bin_step: number;
        is_open: boolean;
      };
    }
  | {
      remove_preset: {
        bin_step: number;
      };
    }
  | {
      set_fee_parameters_on_pair: {
        base_factor: number;
        bin_step: number;
        decay_period: number;
        filter_period: number;
        max_volatility_accumulator: number;
        protocol_share: number;
        reduction_factor: number;
        token_x: TokenType;
        token_y: TokenType;
        variable_fee_control: number;
      };
    }
  | {
      set_fee_recipient: {
        fee_recipient: Addr;
      };
    }
  | {
      add_quote_asset: {
        asset: TokenType;
      };
    }
  | {
      remove_quote_asset: {
        asset: TokenType;
      };
    }
  | {
      force_decay: {
        pair: LBPair;
      };
    };
export type RewardsDistributionAlgorithm =
  | "time_based_rewards"
  | "volume_based_rewards";
export interface ContractInstantiationInfo {
  code_hash: string;
  id: number;
}
export interface FeeRecipientResponse {
  fee_recipient: Addr;
}
export interface InstantiateMsg {
  admin_auth: RawContract;
  fee_recipient: Addr;
  owner?: Addr | null;
  recover_staking_funds_receiver: Addr;
}
export interface RawContract {
  address: string;
  code_hash: string;
}
export interface IsQuoteAssetResponse {
  is_quote: boolean;
}
export interface LBPairAtIndexResponse {
  lb_pair: LBPair;
}
export interface LBPairImplementationResponse {
  lb_pair_implementation: ContractInstantiationInfo;
}
export interface LBPairInformationResponse {
  lb_pair_information: LBPairInformation;
}
export interface LBTokenImplementationResponse {
  lb_token_implementation: ContractInstantiationInfo;
}
export interface MinBinStepResponse {
  min_bin_step: number;
}
export interface NumberOfLBPairsResponse {
  lb_pair_number: number;
}
export interface NumberOfQuoteAssetsResponse {
  number_of_quote_assets: number;
}
export interface OpenBinStepsResponse {
  open_bin_steps: number[];
}
export interface PresetResponse {
  base_factor: number;
  decay_period: number;
  filter_period: number;
  is_open: boolean;
  max_volatility_accumulator: number;
  protocol_share: number;
  reduction_factor: number;
  variable_fee_control: number;
}
export type QueryMsg =
  | {
      get_min_bin_step: {};
    }
  | {
      get_fee_recipient: {};
    }
  | {
      get_lb_pair_implementation: {};
    }
  | {
      get_lb_token_implementation: {};
    }
  | {
      get_number_of_lb_pairs: {};
    }
  | {
      get_lb_pair_at_index: {
        index: number;
      };
    }
  | {
      get_number_of_quote_assets: {};
    }
  | {
      get_quote_asset_at_index: {
        index: number;
      };
    }
  | {
      is_quote_asset: {
        token: TokenType;
      };
    }
  | {
      get_lb_pair_information: {
        bin_step: number;
        token_x: TokenType;
        token_y: TokenType;
      };
    }
  | {
      get_preset: {
        bin_step: number;
      };
    }
  | {
      get_all_bin_steps: {};
    }
  | {
      get_open_bin_steps: {};
    }
  | {
      get_all_lb_pairs: {
        token_x: TokenType;
        token_y: TokenType;
      };
    };
export interface QuoteAssetAtIndexResponse {
  asset: TokenType;
}
