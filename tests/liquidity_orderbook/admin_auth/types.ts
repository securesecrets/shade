export interface InstantiateMsg {
  super_admin?: string | null;
}

export type ExecuteMsg =
  | {
      update_registry: {
        action: RegistryAction;
      };
    }
  | {
      update_registry_bulk: {
        actions: RegistryAction[];
      };
    }
  | {
      transfer_super: {
        new_super: string;
      };
    }
  | {
      self_destruct: {};
    }
  | {
      toggle_status: {
        new_status: AdminAuthStatus;
      };
    };
export type RegistryAction =
  | {
      register_admin: {
        user: string;
      };
    }
  | {
      grant_access: {
        permissions: string[];
        user: string;
      };
    }
  | {
      revoke_access: {
        permissions: string[];
        user: string;
      };
    }
  | {
      delete_admin: {
        user: string;
      };
    };
export type AdminAuthStatus = "active" | "maintenance" | "shutdown";
