
# Mint Router Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [UpdateMintLimit](#UpdateMintLimit)
            * [RegisterAsset](#RegisterAsset)
            * [RemoveAsset](#RemoveAsset)
    * [User](#User)
        * Messages
            * [Receive](#Receive)
        * Queries
            * [NativeAsset](#GetNativeAsset)
            * [Config](#GetConfig)
            * [SupportedAssets](#GetSupportedAssets)
            * [Asset](#GetAsset)
# Introduction
Contract responsible to mint a paired snip20 asset

# Sections

## Init
##### Request
|Name             |Type        |Description                                                                    | optional |
|-----------------|------------|-------------------------------------------------------------------------------|----------|
|admin            | string     |  New contract owner; SHOULD be a valid bech32 address                         |  yes     |
|oracle           | Contract   |  Oracle contract                                                              |  no      |
|peg              | String     |  Symbol to peg to when querying oracle (defaults to native_asset symbol)      |  yes     |
|treasury         | Contract   |  Treasury contract                                                            |  yes     |
|secondary_burn   | Addrr |  Where non-burnable assets will go                                            |  yes     |
## Admin

### Messages
#### UpdateConfig
Updates the given values
##### Request
|Name           |Type        |Description                                            | optional |
|---------------|------------|-------------------------------------------------------|----------|
|admin          | string     |  New contract admin; SHOULD be a valid bech32 address |  yes     |
|oracle         | Contract   |  Oracle contract                                      |  yes     |
|treasury       | Contract   |  Treasury contract                                    |  yes     |
|secondary_burn | Addrr |  Where non-burnable assets will go                    |  yes     |
##### Response
```json
{
  "update_config": {
    "status": "success"
  }
}
```

#### UpdateMintLimit
Updates the mint limit and epoch time
##### Request
|Name             |Type      |Description                                                                     | optional |
|-----------------|----------|--------------------------------------------------------------------------------|----------|
|start_epoch      | String   |  The starting epoch                                                            |  yes     |
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

##### Request
|Name        |Type    |Description                          | optional |
|------------|--------|-------------------------------------|----------|
|contract    | Contract |  Type explained [here](#Contract) |  no      |
##### Response
```json
{
  "register_asset": {
    "status": "success"
  }
}
```

#### RemoveAsset
Remove a registered asset.
##### Request
|Name        |Type    |Description                     | optional |
|------------|--------|--------------------------------|----------|
|address     | String |  The asset to remove's address |  no      |
##### Response
```json
{
  "remove_asset": {
    "status": "success"
  }
}
```

##User

### Messages

#### Receive
To mint the user must use a supported asset's send function and send the amount over to the contract's address. The contract will take care of the rest.

In the msg field of a snip20 send command you must send a base64 encoded json like this one
```json
{"minimum_expected_amount": "Uint128" }
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
      "admin": "Owner address",
      "oracle": {
        "address": "Asset contract address",
        "code_hash": "Asset callback code hash"
        },
      "treasury": {
        "address": "Asset contract address",
        "code_hash": "Asset callback code hash"
      },
      "secondary_burn": "Optional burn address",
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
