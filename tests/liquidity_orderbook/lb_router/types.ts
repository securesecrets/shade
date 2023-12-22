import { TokenType } from "../lb_factory/types";
import { ContractInfo } from "../lb_pair";

export interface InstantiateMsg {
  factory: ContractInfo,
  admins?: string[],
}

export interface CreateLBPairMsg {
  create_lb_pair: {
    token_x: TokenType;
    token_y: TokenType;
    active_id: number;
    bin_step: number;
  }
}

export interface SwapTokensForExactMsg {
  swap_tokens_for_exact: {
    offer: TokenAmount;
    expected_return?: number;
    path: Hop[];
    recipient?: string;
  }
}

export interface TokenAmount {
  token: TokenType,
  amount: string,
}

export interface Hop {
  addr: string,
  code_hash: string,
}


export interface GetFactoryQuery {
  get_factory: {};
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

export interface FactoryResponse {
  factory: string;
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