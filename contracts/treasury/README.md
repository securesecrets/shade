# Treasury Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [RegisterAsset](#RegisterAsset)
        * Queries
            * [GetConfig](#GetConfig)
            * [GetBalance](#GetBalance)
# Introduction
The treasury contract holds network funds from things such as mint commission and pending airdrop funds

# Sections

## Init
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | string   |  contract owner/admin; a valid bech32 address; Controls funds

## Admin

### Messages
#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | string   |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |
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

### Queries

#### GetConfig
Gets the contract's configuration variables
##### Response
```json
{
  "config": {
    "config": {
      "owner": "Owner address",
    }
  }
}
```

#### GetBalance
Get the treasury balance for a given snip20 asset
Note: Snip20 assets must be registered to have viewing key set
##### Response
```json
{
  "get_balance": {
    "contract": "asset address",
  }
}
```
