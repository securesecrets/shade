export interface InstantiateMsg {
  name: string;
  symbol: string;
  decimals: number;
  lb_pair: string;
}

export interface ApproveForAllMsg {
  approveForAll: {
    spender: string;
    approved: boolean;
  };
}

export interface BatchTransferFromMsg {
  batchTransferFrom: {
    from: string;
    to: string;
    ids: number[];
    amounts: string[];
  };
}

export interface MintMsg {
  mint: {
    recipient: string;
    id: number;
    amount: string;
  };
}

export interface BurnMsg {
  burn: {
    owner: string;
    id: number;
    amount: string;
  };
}

export interface NameQuery {
  name: {};
}

export interface NameResponse {
  name: string;
}

export interface SymbolQuery {
  symbol: {};
}

export interface SymbolResponse {
  symbol: string;
}

export interface DecimalsQuery {
  decimals: {};
}

export interface DecimalsResponse {
  decimals: number;
}

export interface TotalSupplyQuery {
  totalSupply: {
    id: number;
  };
}

export interface TotalSupplyResponse {
  total_supply: string;
}

export interface BalanceOfQuery {
  balanceOf: {
    owner: string;
    id: number;
  };
}

export interface BalanceOfResponse {
  balance: string;
}

export interface BalanceOfBatchQuery {
  balanceOfBatch: {
    owners: string[];
    ids: number[];
  };
}

export interface BalanceOfBatchResponse {
  balances: string[];
}

export interface IsApprovedForAllQuery {
  isApprovedForAll: {
    owner: string;
    spender: string;
  };
}

export interface IsApprovedForAllResponse {
  is_approved: boolean;
}
