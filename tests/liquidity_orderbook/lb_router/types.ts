export type ExecuteMsg =
  | {
      receive: Snip20ReceiveMsg;
    }
  | {
      swap_tokens_for_exact: {
        expected_return?: Uint128 | null;
        offer: TokenAmount;
        padding?: string | null;
        path: Hop[];
        recipient?: string | null;
      };
    }
  | {
      register_s_n_i_p20_token: {
        oracle_key?: string | null;
        padding?: string | null;
        token_addr: string;
        token_code_hash: string;
      };
    }
  | {
      recover_funds: {
        amount: Uint128;
        msg?: Binary | null;
        padding?: string | null;
        to: string;
        token: TokenType;
      };
    }
  | {
      set_config: {
        admin_auth?: Contract | null;
        padding?: string | null;
      };
    };
export type Uint128 = string;
export type Binary = string;
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
export type Addr = string;
export interface Snip20ReceiveMsg {
  amount: Uint128;
  from: string;
  memo?: string | null;
  msg?: Binary | null;
  sender: string;
}
export interface TokenAmount {
  amount: Uint128;
  token: TokenType;
  [k: string]: unknown;
}
export interface Hop {
  addr: string;
  code_hash: string;
}
export interface Contract {
  address: Addr;
  code_hash: string;
}
export interface InitMsg {
  admin_auth: Contract;
  airdrop_address?: Contract | null;
  entropy: Binary;
  prng_seed: Binary;
}
export type InvokeMsg = {
  swap_tokens_for_exact: {
    expected_return?: Uint128 | null;
    path: Hop[];
    recipient?: string | null;
  };
};
export type QueryMsgResponse =
  | {
      swap_simulation: {
        lp_fee_amount: Uint128;
        price: string;
        result: SwapResult;
        shade_dao_fee_amount: Uint128;
        total_fee_amount: Uint128;
      };
    }
  | {
      get_config: {
        admin_auth: Contract;
        airdrop_address?: Contract | null;
      };
    }
  | {
      registered_tokens: {
        tokens: Addr[];
      };
    };
export interface SwapResult {
  return_amount: Uint128;
}
export type QueryMsg =
  | {
      swap_simulation: {
        exclude_fee?: boolean | null;
        offer: TokenAmount;
        path: Hop[];
      };
    }
  | {
      get_config: {};
    }
  | {
      registered_tokens: {};
    };
