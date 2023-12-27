/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { ExecuteMsg, RegistryAction, AdminAuthStatus, InstantiateMsg, QueryMsg } from "./Sg721.types";
export interface Sg721ReadOnlyInterface {
  contractAddress: string;
  getConfig: () => Promise<GetConfigResponse>;
  getAdmins: () => Promise<GetAdminsResponse>;
  getPermissions: ({
    user
  }: {
    user: string;
  }) => Promise<GetPermissionsResponse>;
  validateAdminPermission: ({
    permission,
    user
  }: {
    permission: string;
    user: string;
  }) => Promise<ValidateAdminPermissionResponse>;
}
export class Sg721QueryClient implements Sg721ReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.getConfig = this.getConfig.bind(this);
    this.getAdmins = this.getAdmins.bind(this);
    this.getPermissions = this.getPermissions.bind(this);
    this.validateAdminPermission = this.validateAdminPermission.bind(this);
  }

  getConfig = async (): Promise<GetConfigResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_config: {}
    });
  };
  getAdmins = async (): Promise<GetAdminsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_admins: {}
    });
  };
  getPermissions = async ({
    user
  }: {
    user: string;
  }): Promise<GetPermissionsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_permissions: {
        user
      }
    });
  };
  validateAdminPermission = async ({
    permission,
    user
  }: {
    permission: string;
    user: string;
  }): Promise<ValidateAdminPermissionResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      validate_admin_permission: {
        permission,
        user
      }
    });
  };
}
export interface Sg721Interface extends Sg721ReadOnlyInterface {
  contractAddress: string;
  sender: string;
  updateRegistry: ({
    action
  }: {
    action: RegistryAction;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateRegistryBulk: ({
    actions
  }: {
    actions: RegistryAction[];
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  transferSuper: ({
    newSuper
  }: {
    newSuper: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  selfDestruct: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  toggleStatus: ({
    newStatus
  }: {
    newStatus: AdminAuthStatus;
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
    this.updateRegistry = this.updateRegistry.bind(this);
    this.updateRegistryBulk = this.updateRegistryBulk.bind(this);
    this.transferSuper = this.transferSuper.bind(this);
    this.selfDestruct = this.selfDestruct.bind(this);
    this.toggleStatus = this.toggleStatus.bind(this);
  }

  updateRegistry = async ({
    action
  }: {
    action: RegistryAction;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_registry: {
        action
      }
    }, fee, memo, _funds);
  };
  updateRegistryBulk = async ({
    actions
  }: {
    actions: RegistryAction[];
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_registry_bulk: {
        actions
      }
    }, fee, memo, _funds);
  };
  transferSuper = async ({
    newSuper
  }: {
    newSuper: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      transfer_super: {
        new_super: newSuper
      }
    }, fee, memo, _funds);
  };
  selfDestruct = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      self_destruct: {}
    }, fee, memo, _funds);
  };
  toggleStatus = async ({
    newStatus
  }: {
    newStatus: AdminAuthStatus;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      toggle_status: {
        new_status: newStatus
      }
    }, fee, memo, _funds);
  };
}