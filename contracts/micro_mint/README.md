
# Mint Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [UpdateMintLimit](#UpdateMintLimit)
            * [RegisterAsset](#RegisterAsset)
        * Queries
            * [GetNativeAsset](#GetNativeAsset)
            * [GetConfig](#GetConfig)
            * [MintLimit](#GetMintLimit)
            * [SupportedAssets](#GetSupportedAssets)
            * [GetAsset](#GetAsset)
    * [User](#User)
        * Messages
            * [Receive](#Receive)
# Introduction
Contract responsible to mint a paired snip20 asset

# Sections

## Init
##### Request
|Name             |Type      |Description                                                                                                        | optional |
|-----------------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|admin            | string   |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |
|native_asset     | Contract |  Asset to mint                                                                                                    |  no      |
|peg              | String   |  Symbol to peg to when querying oracle (defaults to native_asset symbol)                                          |  yes     |
|treasury         | Contract |  Treasury contract                                                                                                |  yes     |
|oracle           | Contract |  Oracle contract                                                                                                  |  no      |
|epoch_frequency  | String   |  The frequency in which the mint limit resets, if 0 then no limit is enforced                                     |  yes     |
|epoch_mint_limit | String   |  The limit of uTokens to mint per epoch                                                                           |  yes     |
## Admin

### Messages
#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | string   |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |
|treasury  | Contract |  Treasury contract                                                                                                |  yes     |
|oracle    | Contract |  Oracle contract                                                                                                  |  yes     |
##### Response
```json
{
  "update_config": {
    "status": "success"
  }
}
```

#### UpdateMintLimit
Updates the given values
##### Request
|Name      |Type      |Description                                                                            | optional |
|----------|----------|---------------------------------------------------------------------------------------|----------|
|epoch_frequency  | String   |  The frequency in which the mint limit resets, if 0 then no limit is enforced  |  yes     |
|epoch_mint_limit | String   |  The limit of uTokens to mint per epoch                                        |  yes     |
##### Response
```json
{
  "update_mint_limit": {
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

### Queries

#### GetNativeAsset
Gets the contract's minted asset
#### Response
```json
{
  "native_asset": {
    "asset": "Snip20Asset Object",
    "peg": "Pegged symbol"
  }
}
```

#### GetConfig
Gets the contract's configuration variables
##### Response
```json
{
  "config": {
    "config": {
      "owner": "Owner address",
      "oracle": {
        "address": "Asset contract address",
        "code_hash": "Asset callback code hash"
        },
      "treasury": {
        "address": "Asset contract address",
        "code_hash": "Asset callback code hash"
      },
      "activated": "Boolean of contract's actviation status"
    }
  }
}
```

#### GetMintLimit
Gets the contract's configuration variables
##### Response
```json
{
  "limit": {
    "mint_limit": {
      "frequency": "Frequency per epoch reset",
      "mint_capacity": "Mint capacity per epoch",
      "total_minted": "Total minted in current epoch",
      "next_epoch": "Timestamp for the next epoch"
    }
  }
}
```

#### GetSupportedAssets
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
      "Snip20Asset": {
        "contract": "Asset contract",
        "token_info": "Token info as per Snip20",
        "token_config": "Optional information about the config if the Snip20 supports it"
      },
      "burned": "Total burned on this contract"
    }
  }
}
```

##User

### Messages

#### Receive
To mint the user must use a supported asset's send function and send the amount over to the contract's address. The contract will take care of the rest.

In the msg field of a snip20 send command you must send a base64 encoded json like this one
```json
{"minimum_expected_amount": "Uint128", "mint_type": { "coin_to_silk": { } } }
```

The currently supported mint types are ```coin_to_silk``` , ```coin_to_shade```, ```convert_to_silk``` and ```convert_to_shade```

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