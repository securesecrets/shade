// TODO put the common types somewhere else

export interface CustomToken {
  custom_token: {
    contract_addr: string;
    token_code_hash: string;
    // viewing_key: string;
  };
}

export interface NativeToken {
  native_token: {
    denom: string;
  };
}

export type TokenType = CustomToken | NativeToken;


export interface LBPair {
  token_x: string;
  token_y: string;
  bin_step: number;
  contract: {
    address: string;
    code_hash: string;
  };
}

export interface LBPairInformation {
  bin_step: number;
  lb_pair: LBPair;
  created_by_owner: boolean;
  ignored_for_routing: boolean;
}

export interface InstantiateMsg {
  owner?: string;
  fee_recipient: string;
  flash_loan_fee: number;
}

export interface SetLBPairImplementationMsg {
  set_lb_pair_implementation: {
    lb_pair_implementation: {
      id: number;
      code_hash: string;
    };
  };
}

export interface SetLBTokenImplementationMsg {
  set_lb_token_implementation: {
    lb_token_implementation: {
      id: number;
      code_hash: string;
    };
  };
}

export interface CreateLBPairMsg {
  create_lb_pair: {
    token_x: TokenType;
    token_y: TokenType;
    active_id: number;
    bin_step: number;
  }
}

export interface SetLBPairIgnoredMsg {
  set_lb_pair_ignored: {
    token_a: TokenType;
    token_b: TokenType;
    bin_step: number;
    ignored: boolean;
  }
}

export interface SetPresetMsg {
  set_preset: {
    bin_step: number;
    base_factor: number;
    filter_period: number;
    decay_period: number;
    reduction_factor: number;
    variable_fee_control: number;
    protocol_share: number;
    max_volatility_accumulator: number;
    is_open: boolean;
  }
}

export interface SetPresetOpenStateMsg {
  bin_step: number;
  is_open: boolean;
}

export interface RemovePresetMsg {
  bin_step: number;
}

export interface SetFeeParametersOnPairMsg {
  token_x: {
    address: string;
    code_hash: string;
  };
  token_y: {
    address: string;
    code_hash: string;
  };
  bin_step: number;
  base_factor: number;
  filter_period: number;
  decay_period: number;
  reduction_factor: number;
  variable_fee_control: number;
  protocol_share: number;
  max_volatility_accumulator: number;
}

export interface SetFeeRecipientMsg {
  fee_recipient: string;
}

export interface SetFlashLoanFeeMsg {
  flash_loan_fee: number;
}

export interface AddQuoteAssetMsg {
  add_quote_asset: {
    asset: TokenType;
  }
}

export interface RemoveQuoteAssetMsg {
  asset: TokenType;
}

export interface ForceDecayMsg {
  pair: {
    address: string;
    code_hash: string;
  };
}

export interface GetMinBinStepQuery {
  get_min_bin_step: {};
}

export interface GetFeeRecipientQuery {
  get_fee_recipient: {};
}

export interface GetMaxFlashLoanFeeQuery {
  get_max_flash_loan_fee: {};
}

export interface GetFlashLoanFeeQuery {
  get_flash_loan_fee: {};
}

export interface GetLBPairImplementationQuery {
  get_lb_pair_implementation: {};
}

export interface GetLBTokenImplementationQuery {
  get_lb_token_implementation: {};
}

export interface GetNumberOfLBPairsQuery {
  get_number_of_lb_pairs: {};
}

export interface GetLBPairAtIndexQuery {
  get_lb_pair_at_index: {
    index: number;
  };
}

export interface GetNumberOfQuoteAssetsQuery {
  get_number_of_quote_assets: {};
}

export interface GetQuoteAssetAtIndexQuery {
  get_quote_asset_at_index: {
    index: number;
  };
}

export interface IsQuoteAssetQuery {
  is_quote_asset: {
    token: TokenType;
  };
}

export interface GetLBPairInformationQuery {
  get_lb_pair_information: {
    token_a: TokenType;
    token_b: TokenType;
    bin_step: number;
  };
}

export interface GetPresetQuery {
  get_preset: {
    bin_step: number;
  };
}

export interface GetAllBinStepsQuery {
  get_all_bin_steps: {};
}

export interface GetOpenBinStepsQuery {
  get_open_bin_steps: {};
}

export interface GetAllLBPairsQuery {
  get_all_lb_pairs: {
    token_x: string;
    token_y: string;
  };
}

export interface MinBinStepResponse {
  min_bin_step: number;
}

export interface FeeRecipientResponse {
  fee_recipient: string;
}

export interface MaxFlashLoanFeeResponse {
  max_fee: number;
}

export interface FlashLoanFeeResponse {
  flash_loan_fee: number;
}

export interface LBPairImplementationResponse {
  lb_pair_implementation: string;
}

export interface LBTokenImplementationResponse {
  lb_token_implementation: string;
}

export interface NumberOfLBPairsResponse {
  lb_pair_number: number;
}

export interface LBPairAtIndexResponse {
  lb_pair: LBPair;
}

export interface NumberOfQuoteAssetsResponse {
  number_of_quote_assets: number;
}

export interface QuoteAssetAtIndexResponse {
  asset: string;
}

export interface IsQuoteAssetResponse {
  is_quote: boolean;
}

export interface LBPairInformationResponse {
  lb_pair_information: LBPairInformation;
}

export interface PresetResponse {
  base_factor: number;
  filter_period: number;
  decay_period: number;
  reduction_factor: number;
  variable_fee_control: number;
  protocol_share: number;
  max_volatility_accumulator: number;
  is_open: boolean;
}

export interface AllBinStepsResponse {
  bin_step_with_preset: number[];
}

export interface OpenBinStepsResponse {
  open_bin_steps: number[];
}

export interface AllLBPairsResponse {
  lb_pairs_available: LBPairInformation[];
}
