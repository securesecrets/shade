/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type ExecuteMsg = {
  update_registry: {
    action: RegistryAction;
  };
} | {
  update_registry_bulk: {
    actions: RegistryAction[];
  };
} | {
  transfer_super: {
    new_super: string;
  };
} | {
  self_destruct: {};
} | {
  toggle_status: {
    new_status: AdminAuthStatus;
  };
};
export type RegistryAction = {
  register_admin: {
    user: string;
  };
} | {
  grant_access: {
    permissions: string[];
    user: string;
  };
} | {
  revoke_access: {
    permissions: string[];
    user: string;
  };
} | {
  delete_admin: {
    user: string;
  };
};
export type AdminAuthStatus = "active" | "maintenance" | "shutdown";
export interface InstantiateMsg {
  super_admin?: string | null;
}
export type QueryMsg = {
  get_config: {};
} | {
  get_admins: {};
} | {
  get_permissions: {
    user: string;
  };
} | {
  validate_admin_permission: {
    permission: string;
    user: string;
  };
};