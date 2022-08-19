# Admin Auth Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [AddContract](#AddContract)
            * [RemoveContract](#RemoveContract)
            * [AddAuthorization](#AddAuthorization)
            * [RemoveAuthorization](#RemoveAuthorization)
            * [AddSuper](#AddSuper)
            * [RemoveSuper](#RemoveSuper)
    * [User](#User)
        * Queries
            * [GetSuperAdmins](#GetSuperAdmins)
            * [GetContracts](#GetContracts)
            * [GetAuthorizedUsers](#GetAuthorizedUsers)
            * [ValidateAdminPermission](#ValidateAdminPermission)
# Introduction
This contract is used to centrally authorize the owners of a contracts. A contract can query the Shade Admin Contract to confirm whether the original caller has the relevant permissions against the calling contract.

# Sections

## Admin

### Messages
#### AddContract
##### Request
Add a contract.
| Name             | Type   | Description                     | optional |
|------------------|--------|---------------------------------|----------|
| contract | String | Address of contract to be added | no       |

#### RemoveContract
##### Request
Remove a contract.
| Name             | Type   | Description                       | optional |
|------------------|--------|-----------------------------------|----------|
| contract | String | Address of contract to be removed | no       |

#### AddAuthorization
##### Request
Authorize a user with admin perms for the inputted contract.
| Name             | Type   | Description                                  | optional |
|------------------|--------|----------------------------------------------|----------|
| contract | String | Address of contract                          | no       |
| admin    | String | Address of user to be given admin privileges | no       |

#### RemoveAuthorization
##### Request
Deauthorize a user for the inputted contract.
| Name             | Type   | Description                              | optional |
|------------------|--------|------------------------------------------|----------|
| contract | String | Address of contract                      | no       |
| admin    | String | Address of user to lose admin privileges | no       |

#### AddSuper
##### Request
Authorize a user to be given super-admin perms.
| Name          | Type   | Description                                        | optional |
|---------------|--------|----------------------------------------------------|----------|
| super_address | String | Address of user to be given super-admin privileges | no       |

#### RemoveSuper
##### Request
Deauthorize a user from super-admin perms.
| Name          | Type   | Description                                    | optional |
|---------------|--------|------------------------------------------------|----------|
| super_address | String | Address of user to lose super-admin privileges | no       |


## User

### Queries

#### GetSuperAdmins
Gets a list of the super-admin addresses.
##### Response
```json
{
  "SuperAdminResponse": {
    "super_admins": "Vector of strings of the super-admin addresses"
  }
}
```

#### GetContracts
Gets a list of all of the contracts and the users' that have perms over them.
##### Response
```json
{
  "ContractsResponse": {
    "contracts": "Vector containing tuples of the contract addresses and a vector of strings of the authorized users"
  }
}
```

#### GetAuthorizedUsers
Gets a vector of strings of the users' that have perms for the inputted contract address.
##### Request
| Name             | Type   | Description         | optional |
|------------------|--------|---------------------|----------|
| contract | String | Address of contract | no       |
##### Response
```json
{
  "AuthorizedUsersResponse": {
    "authorized_users": "Vector of strings of the authorized users",
  }
}
```

#### ValidateAdminPermission
Determines if inputted admin has admin perms over contract.
##### Request
| Name             | Type   | Description                     | optional |
|------------------|--------|---------------------------------|----------|
| contract | String | Address of contract             | no       |
| admin    | String | Address of user to be validated | no       |
##### Response
```json
{
  "ValidateAdminPermissionResponse": {
    "error_msg": "Option determining if user has perms",
  }
}
```