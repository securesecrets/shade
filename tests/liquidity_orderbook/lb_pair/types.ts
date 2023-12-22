import { TokenType } from "../lb_factory/types";

export interface ContractInfo {
  address: string,
  code_hash: string,
}

export interface StaticFeeParameters {
  base_factor: number,
  filter_period: number,
  decay_period: number,
  reduction_factor: number,
  variable_fee_control: number,
  protocol_share: number,
  max_volatility_accumulator: number,
}

export interface LiquidityParameters {
  token_x: TokenType,
  token_y: TokenType,
  bin_step: number,
  amount_x: string,
  amount_y: string,
  amount_x_min: string,
  amount_y_min: string,
  active_id_desired: number,
  id_slippage: number,    //TODO figure this out
  delta_ids: number[], //TODO this as well
  distribution_x: number[],
  distribution_y: number[],
  // to: string,
  deadline: number,
}

export interface RemoveLiquidity {
  token_x: TokenType,
  token_y: TokenType,
  bin_step: number,
  amount_x_min: string,
  amount_y_min: string,
  ids: number[],
  amounts: string[],
  deadline: number,
}


export interface InstantiateMsg {
  factory: string;
  token_x: ContractInfo;
  token_y: ContractInfo;
  bin_step: number;
  pair_parameters: StaticFeeParameters;
  active_id: number;
  lb_token_implementation: {
    id: number;
    code_hash: string;
  };
}

export interface SwapMsg {
  swap: {
    swap_for_y: boolean;
    to: string;
    amount_received: string;
  }
}

export interface AddLiquidityMsg {
  add_liquidity: {
    liquidity_parameters: LiquidityParameters,
  }
}

export interface RemoveLiquidityMsg {
  remove_liquidity: {
    remove_liquidity_params: RemoveLiquidity,
  }
}

export interface FlashLoanMsg {
  flash_loan: {
    // TODO
  }
}

export interface MintMsg {
  mint: {
    to: string;
    // TODO: figure out proper way to send Bytes32
    liquidity_configs: string[];
    refund_to: string;
    amount_received_x: string;
    amount_received_y: string;
  }
}

export interface BurnMsg {
  burn: {
    from: string;
    to: string;
    ids: string[];
    amounts_to_burn: string[];
  }
}

export interface CollectProtocolFeesMsg {
  collect_protocol_fees: {
    // TODO
  }
}

export interface IncreaseOracleLengthMsg {
  increase_oracle_length: {
    new_length: number;
  }
}

export interface SetStaticFeeParametersMsg {
  set_static_fee_parameters: {
    active_id: number;
    base_factor: number;
    filter_period: number;
    decay_period: number;
    reduction_factor: number;
    variable_fee_control: number;
    protocol_share: number;
    max_volatility_accumulator: number;
  }
}

export interface ForceDecayMsg {
  force_decay: {
    // TODO
  }
}

export interface GetFactoryQuery {
  get_factory: {};
}

export interface GetTokenXQuery {
  get_token_x: {};
}

export interface GetTokenYQuery {
  get_token_y: {};
}

export interface GetBinStepQuery {
  get_bin_step: {};
}

export interface GetReservesQuery {
  get_reserves: {};
}

export interface GetActiveIdQuery {
  get_active_id: {};
}

export interface GetBinQuery {
  get_bin: {
    id: number;
  };
}

export interface GetNextNonEmptyBinQuery {
  get_next_non_empty_bin: {
    swap_for_y: boolean;
    id: number;
  };
}

export interface GetProtocolFeesQuery {
  get_protocol_fees: {};
}

export interface GetStaticFeeParametersQuery {
  get_static_fee_parameters: {};
}

export interface GetVariableFeeParametersQuery {
  get_variable_fee_parameters: {};
}

export interface GetOracleParametersQuery {
  get_oracle_parameters: {};
}

export interface GetOracleSampleAtQuery {
  get_oracle_sample_at: {
    look_up_timestamp: number;
  };
}

export interface GetPriceFromIdQuery {
  get_price_from_id: {
    id: number;
  };
}

export interface GetIdFromPriceQuery {
  get_id_from_price: {
    price: string;
  };
}

export interface GetSwapInQuery {
  get_swap_in: {
    amount_out: string;
    swap_for_y: boolean;
  };
}

export interface GetSwapOutQuery {
  get_swap_out: {
    amount_in: string;
    swap_for_y: boolean;
  };
}
export interface GetTotalSupplyQuery {
  total_supply: {
  id:number
  };
}
export interface FactoryResponse {
  factory: string;
}

export interface TokenXResponse {
  token_x: string;
}

export interface TokenYResponse {
  token_y: string;
}

export interface BinStepResponse {
  bin_step: number;
}

export interface ReservesResponse {
  reserve_x: string;
  reserve_y: string;
}

export interface ActiveIdResponse {
  active_id: number;
}

export interface BinResponse {
  bin_reserve_x: string;
  bin_reserve_y: string;
}

export interface NextNonEmptyBinResponse {
  next_id: number;
}

export interface ProtocolFeesResponse {
  protocol_fee_x: string;
  protocol_fee_y: string;
}

export interface StaticFeeParametersResponse {
  base_factor: number;
  filter_period: number;
  decay_period: number;
  reduction_factor: number;
  variable_fee_control: number;
  protocol_share: number;
  max_volatility_accumulator: number;
}

export interface VariableFeeParametersResponse {
  volatility_accumulator: number;
  volatility_reference: number;
  id_reference: number;
  time_of_last_update: number;
}

export interface OracleParametersResponse {
  sample_lifetime: number;
  size: number;
  active_size: number;
  last_updated: number;
  first_timestamp: number;
}

export interface OracleSampleAtResponse {
  cumulative_id: number;
  cumulative_volatility: number;
  cumulative_bin_crossed: number;
}

export interface PriceFromIdResponse {
  price: string;
}

export interface IdFromPriceResponse {
  id: number;
}

export interface SwapInResponse {
  amount_in: string;
  amount_out_left: string;
  fee: string;
}

export interface SwapOutResponse {
  amount_in_left: string;
  amount_out: string;
  fee: string;
}

export interface TotalSupplyResponse {
  total_supply: string;
}
