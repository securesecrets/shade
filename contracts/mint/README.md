# Mint Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
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
|Name                      |Type    |Description                                                                                                          | optional |
|--------------------------|--------|---------------------------------------------------------------------------------------------------------------------|----------|
|silk_contract             | string |  Silk contract address; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well   |  no     |
|silk_contract_code_hash   | string |  Silk contract callback hash                                                                                        |  no     |
|oracle_contract           | string |  Oracle contract address; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well |  no     |
|oracle_contract_code_hash | string |  Oracle contract callback hash                                                                                      |  no     |

## Admin

### Messages

#### UpdateConfig
Updates the given values
##### Request
|Name                      |Type    |Description                                                                                                          | optional |
|--------------------------|--------|---------------------------------------------------------------------------------------------------------------------|----------|
|owner                     | string |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well      |  yes     |
|silk_contract             | string |  Silk contract address; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well   |  yes     |
|silk_contract_code_hash   | string |  Silk contract callback hash                                                                                        |  yes     |
|oracle_contract           | string |  Oracle contract address; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well |  yes     |
|oracle_contract_code_hash | string |  Oracle contract callback hash                                                                                      |  yes     |
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
|contract    | string |  Snip20 contract address; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well   |  no      |
|code_hash   | string |  Contract callback hash                                                                                               |  no      |
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
|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|asset       | string |  Asset to update; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well           |  no      |
|contract    | string |  Snip20 contract address; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well   |  no      |
|code_hash   | string |  Contract callback hash                                                                                               |  no      |
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
      "silk_contract": "Silk contract address",
      "silk_contract_code_hash": "Silk contract callback code hash",
      "oracle_contract": "Oracle contract address",
      "oracle_contract_code_hash": "Oracle contract callback code hash"
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
      "contract": "Asset contract address",
      "code_hash": "Asset callback code hash",
      "burned_tokens": "Total burned on this contract"
    }
  }
}
```

##User

### Messages

#### Receive
To mint the user must use a supported asset's send function and send the amount over to the contract's address. The contract will take care of the rest.
