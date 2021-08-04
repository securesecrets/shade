# Mint Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [Migrate](#Migrate)
            * [UpdateConfig](#UpdateConfig)
            * [RegisterAsset](#RegisterAsset)
            * [UpdateAsset](#UpdateAsset)
        * Queries
            * [GetConfig](#GetConfig)
            * [SupportedAssets](#SupportedAssets)
            * [GetAsset](#getAsset)
    * [User](#User)
        * Messages
            * [Receive](#Receive)
# Introduction
The minting contract is used as a way to acquire newly minted Silk, sending a set amount from any supported contract will result in receiving x amount of silk.

# Sections

## Init
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | string   |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |
|silk      | Contract |  Silk contract                                                                                                    |  no      |
|oracle    | Contract |  Oracle contract                                                                                                  |  no      |

## Admin

### Messages
### Migrate
Migrates all the contracts state and data into a new contract
#### Request
| Name     | Type   | Description         | optional |
|----------| -------|---------------------|----------|
|label     | String | Contract label name | no       |
|code_id   | u64    | Contract ID         | no       |
|code_hash | String | Contract code hash  | no       |
##### Response
```json
{
  "update_config": {
    "status": "success"
  }
}
```

#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | string   |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |
|silk      | Contract |  Silk contract                                                                                                    |  no      |
|oracle    | Contract |  Oracle contract                                                                                                  |  no      |
##### Response
```json
{
  "update_config": {
    "status": "success"
  }
}
```

#### RegisterAsset
Registers a supported asset. The asset must be SNIP-20 compliant since [RegisterReceive](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md#RegisterReceive) is called.

Note: Will return an error if there's an asset with that address already registered.
##### Request
|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|contract    | Contract |  Type explained [here](#Contract)                                                                                     |  no      |
##### Response
```json
{
  "register_asset": {
    "status": "success"
  }
}
```

#### UpdateAsset
Updates a supported asset. The asset must be SNIP-20 compliant since [RegisterReceive](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md#RegisterReceive) is called.

Note: Will return an error if no asset exists already with that address.
##### Request
|Name        |Type      |Description                                                                                                            | optional |
|------------|----------|-----------------------------------------------------------------------------------------------------------------------|----------|
|asset       | string   |  Asset to update; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well           |  no      |
|contract    | Contract |  Type explained [here](#Contract)                                                                                     |  no      |
##### Response
```json
{
  "update_asset": {
    "status": "success"
  }
}
```

### Queries

#### GetConfig
Gets the contract's configuration variables
##### Response
```json
{
  "config": {
    "config": {
      "owner": "Owner address",
      "silk": {
        "address": "Asset contract address",
        "code_hash": "Asset callback code hash"
        },
      "oracle": {
        "address": "Asset contract address",
        "code_hash": "Asset callback code hash"
        },
      "activated": "Boolean of contract's actviation status"
    }
  }
}
```

#### SupportedAssets
Get all the contract's supported assets.
##### Response
```json
{
  "supported_assets": {
    "assets": ["asset address"]
  }
}
```

#### GetAsset
Get specific information on a supported asset.
##### Request
|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|contract    | string |  Snip20 contract address; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well   |  no      |
##### Response
```json
{
  "asset": {
    "asset": {
      "contract": {
        "address": "Asset contract address",
        "code_hash": "Asset callback code hash"
      },
      "burned_tokens": "Total burned on this contract"
    }
  }
}
```

##User

### Messages

#### Receive
To mint the user must use a supported asset's send function and send the amount over to the contract's address. The contract will take care of the rest.

## Contract
Type used in many of the admin commands
```json
{
  "config": {
    "address": "Asset contract address",
    "code_hash": "Asset callback code hash"
  }
}
```