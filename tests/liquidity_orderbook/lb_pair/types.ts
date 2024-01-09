/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

export interface ActiveIdResponse {
  active_id: number;
}
export interface AllBinsResponse {
  current_block_height: number;
  last_id: number;
  reserves: BinResponse[];
}
export interface BinResponse {
  bin_id: number;
  bin_reserve_x: number;
  bin_reserve_y: number;
}
export interface BinStepResponse {
  bin_step: number;
}
export type BinUpdatingHeightsResponse = number[];
export type BinsResponse = BinResponse[];
export type ExecuteMsg =
  | {
      swap_tokens: {
        expected_return?: Uint128 | null;
        offer: TokenAmount;
        padding?: string | null;
        to?: string | null;
      };
    }
  | {
      receive: Snip20ReceiveMsg;
    }
  | {
      add_liquidity: {
        liquidity_parameters: LiquidityParameters;
      };
    }
  | {
      remove_liquidity: {
        remove_liquidity_params: RemoveLiquidity;
      };
    }
  | {
      collect_protocol_fees: {};
    }
  | {
      increase_oracle_length: {
        new_length: number;
      };
    }
  | {
      set_static_fee_parameters: {
        base_factor: number;
        decay_period: number;
        filter_period: number;
        max_volatility_accumulator: number;
        protocol_share: number;
        reduction_factor: number;
        variable_fee_control: number;
      };
    }
  | {
      force_decay: {};
    }
  | {
      calculate_rewards: {};
    }
  | {
      reset_rewards_config: {
        base_rewards_bins?: number | null;
        distribution?: RewardsDistributionAlgorithm | null;
      };
    }
  | {
      set_contract_status: {
        contract_status: ContractStatus;
      };
    };

export type InvokeMsg = {
  swap_tokens: {
    expected_return?: Uint128 | null;
    padding?: string | null;
    to?: string | null;
  };
};
export type Uint128 = string;
export type TokenType =
  | {
      custom_token: {
        contract_addr: Addr;
        token_code_hash: string;
      };
    }
  | {
      native_token: {
        denom: string;
      };
    };
export type Addr = string;
export type Binary = string;
export type Uint256 = string;
export type RewardsDistributionAlgorithm =
  | "time_based_rewards"
  | "volume_based_rewards";
export type ContractStatus = "active" | "freeze_all" | "lp_withdraw_only";
export interface TokenAmount {
  amount: Uint128;
  token: TokenType;
  [k: string]: unknown;
}
export interface Snip20ReceiveMsg {
  amount: Uint128;
  from: string;
  memo?: string | null;
  msg?: Binary | null;
  sender: string;
}
export interface LiquidityParameters {
  active_id_desired: number;
  amount_x: Uint128;
  amount_x_min: Uint128;
  amount_y: Uint128;
  amount_y_min: Uint128;
  bin_step: number;
  deadline: number;
  delta_ids: number[];
  distribution_x: number[];
  distribution_y: number[];
  id_slippage: number;
  token_x: TokenType;
  token_y: TokenType;
}
export interface RemoveLiquidity {
  amount_x_min: Uint128;
  amount_y_min: Uint128;
  amounts: Uint256[];
  bin_step: number;
  deadline: number;
  ids: number[];
  token_x: TokenType;
  token_y: TokenType;
}
export interface FactoryResponse {
  factory: Addr;
}
export type Decimal256 = string;
export interface GetPairInfoResponse {
  amount_0: Uint128;
  amount_1: Uint128;
  contract_version: number;
  factory?: ContractInfo | null;
  fee_info: FeeInfo;
  liquidity_token: ContractInfo;
  pair: TokenPair;
  stable_info?: StablePairInfoResponse | null;
  total_liquidity: Uint256;
}
export interface ContractInfo {
  address: Addr;
  code_hash?: string;
  [k: string]: unknown;
}
export interface FeeInfo {
  lp_fee: Fee;
  shade_dao_address: Addr;
  shade_dao_fee: Fee;
  stable_lp_fee: Fee;
  stable_shade_dao_fee: Fee;
}
export interface Fee {
  denom: number;
  nom: number;
}
export interface TokenPair {
  token_0: TokenType;
  token_1: TokenType;
}
export interface StablePairInfoResponse {
  p?: Decimal256 | null;
  stable_params: StableParams;
  stable_token0_data: StableTokenData;
  stable_token1_data: StableTokenData;
}
export interface StableParams {
  a: Decimal256;
  custom_iteration_controls?: CustomIterationControls | null;
  gamma1: Uint256;
  gamma2: Uint256;
  max_price_impact_allowed: Decimal256;
  min_trade_size_x_for_y: Decimal256;
  min_trade_size_y_for_x: Decimal256;
  oracle: Contract;
}
export interface CustomIterationControls {
  epsilon: Uint256;
  max_iter_bisect: number;
  max_iter_newton: number;
}
export interface Contract {
  address: Addr;
  code_hash: string;
}
export interface StableTokenData {
  decimals: number;
  oracle_key: string;
}
export interface IdFromPriceResponse {
  id: number;
}
export interface InstantiateMsg {
  active_id: number;
  admin_auth: RawContract;
  bin_step: number;
  entropy: string;
  epoch_staking_duration: number;
  epoch_staking_index: number;
  expiry_staking_duration?: number | null;
  factory: ContractInfo;
  lb_token_implementation: ContractInstantiationInfo;
  pair_parameters: StaticFeeParameters;
  protocol_fee_recipient: Addr;
  recover_staking_funds_receiver: Addr;
  rewards_distribution_algorithm: RewardsDistributionAlgorithm;
  staking_contract_implementation: ContractInstantiationInfo;
  token_x: TokenType;
  token_y: TokenType;
  total_reward_bins?: number | null;
  viewing_key: string;
}
export interface RawContract {
  address: string;
  code_hash: string;
}
export interface ContractInstantiationInfo {
  code_hash: string;
  id: number;
}
export interface StaticFeeParameters {
  base_factor: number;
  decay_period: number;
  filter_period: number;
  max_volatility_accumulator: number;
  protocol_share: number;
  reduction_factor: number;
  variable_fee_control: number;
}
export interface LbTokenResponse {
  contract: ContractInfo;
}

export interface NextNonEmptyBinResponse {
  next_id: number;
}
export interface OracleParametersResponse {
  active_size: number;
  first_timestamp: number;
  last_updated: number;
  sample_lifetime: number;
  size: number;
}
export interface OracleSampleAtResponse {
  cumulative_bin_crossed: number;
  cumulative_id: number;
  cumulative_volatility: number;
}
export interface PriceFromIdResponse {
  price: Uint256;
}
export interface ProtocolFeesResponse {
  protocol_fee_x: number;
  protocol_fee_y: number;
}
export type QueryMsg =
  | {
      get_staking_contract: {};
    }
  | {
      get_lb_token: {};
    }
  | {
      get_pair_info: {};
    }
  | {
      swap_simulation: {
        exclude_fee?: boolean | null;
        offer: TokenAmount;
      };
    }
  | {
      get_factory: {};
    }
  | {
      get_tokens: {};
    }
  | {
      get_token_x: {};
    }
  | {
      get_token_y: {};
    }
  | {
      get_bin_step: {};
    }
  | {
      get_reserves: {};
    }
  | {
      get_active_id: {};
    }
  | {
      get_bin_reserves: {
        id: number;
      };
    }
  | {
      get_bins_reserves: {
        ids: number[];
      };
    }
  | {
      get_all_bins_reserves: {
        id?: number | null;
        page?: number | null;
        page_size?: number | null;
      };
    }
  | {
      get_updated_bin_at_height: {
        height: number;
      };
    }
  | {
      get_updated_bin_at_multiple_heights: {
        heights: number[];
      };
    }
  | {
      get_updated_bin_after_height: {
        height: number;
        page?: number | null;
        page_size?: number | null;
      };
    }
  | {
      get_bin_updating_heights: {
        page?: number | null;
        page_size?: number | null;
      };
    }
  | {
      get_next_non_empty_bin: {
        id: number;
        swap_for_y: boolean;
      };
    }
  | {
      get_protocol_fees: {};
    }
  | {
      get_static_fee_parameters: {};
    }
  | {
      get_variable_fee_parameters: {};
    }
  | {
      get_oracle_parameters: {};
    }
  | {
      get_oracle_sample_at: {
        look_up_timestamp: number;
      };
    }
  | {
      get_price_from_id: {
        id: number;
      };
    }
  | {
      get_id_from_price: {
        price: Uint256;
      };
    }
  | {
      get_swap_in: {
        amount_out: Uint128;
        swap_for_y: boolean;
      };
    }
  | {
      get_swap_out: {
        amount_in: Uint128;
        swap_for_y: boolean;
      };
    }
  | {
      total_supply: {
        id: number;
      };
    }
  | {
      get_rewards_distribution: {
        epoch_id?: number | null;
      };
    };
export interface ReservesResponse {
  reserve_x: number;
  reserve_y: number;
}
export interface RewardsDistributionResponse {
  distribution: RewardsDistribution;
}
export interface RewardsDistribution {
  denominator: number;
  ids: number[];
  weightages: number[];
}
export interface StakingResponse {
  contract: ContractInfo;
}
export interface StaticFeeParametersResponse {
  base_factor: number;
  decay_period: number;
  filter_period: number;
  max_volatility_accumulator: number;
  protocol_share: number;
  reduction_factor: number;
  variable_fee_control: number;
}
export interface SwapInResponse {
  amount_in: Uint128;
  amount_out_left: Uint128;
  fee: Uint128;
}
export interface SwapOutResponse {
  amount_in_left: Uint128;
  amount_out: Uint128;
  lp_fees: Uint128;
  shade_dao_fees: Uint128;
  total_fees: Uint128;
}
export interface SwapSimulationResponse {
  lp_fee_amount: Uint128;
  price: string;
  result: SwapResult;
  shade_dao_fee_amount: Uint128;
  total_fee_amount: Uint128;
}
export interface SwapResult {
  return_amount: Uint128;
}
export interface TokenXResponse {
  token_x: TokenType;
}
export interface TokenYResponse {
  token_y: TokenType;
}
export interface TokensResponse {
  token_x: TokenType;
  token_y: TokenType;
}
export interface TotalSupplyResponse {
  total_supply: Uint256;
}
export interface UpdatedBinsAfterHeightResponse {
  bins: BinResponse[];
  current_block_height: number;
}
export type UpdatedBinsAtHeightResponse = BinResponse[];
export type UpdatedBinsAtMultipleHeightResponse = BinResponse[];
export interface VariableFeeParametersResponse {
  id_reference: number;
  time_of_last_update: number;
  volatility_accumulator: number;
  volatility_reference: number;
}
