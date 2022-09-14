# Treasury Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [DAO Adapter](/packages/shade_protocol/src/DAO_ADAPTER.md)
    * [Init](#Init)
    * [Interface](#Interface)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [RegisterAsset](#RegisterAsset)
            * [Allocate](#Allocate)
        * Queries
            * [Config](#Config)
            * [Assets](#Assets)
            * [PendingAllowance](#PendingAllowance)
# Introduction
The treasury contract holds network funds from things such as mint commission and pending airdrop funds

# Sections

## Init
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|admin     | Addr|  Admin address
|viewing_key | String |  Key set on relevant SNIP-20's
|treasury    | Addr |  treasury that is owner of funds

## Interface

### Messages
#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|config    | Config   |  New contract config
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

#### Allocate
Registers a supported asset. The asset must be SNIP-20 compliant since [RegisterReceive](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md#RegisterReceive) is called.

Note: Will return an error if there's an asset with that address already registered.
##### Request
|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|asset       | Addr |  Desired SNIP-20
|allocation  | Allocation | Allocation data
##### Response
```json
{
  "allocate": {
    "status": "success"
  }
}
```

### Queries

#### Config
Gets the contract's configuration variables
##### Response
```json
{
  "config": {
    "config": { .. }
  }
}
```

#### Assets
Get the list of registered assets
##### Response
```json
{
  "assets": {
    "assets": ["asset address", ..],
  }
}
```

#### Allocations
Get the allocations for a given asset

##### Request
|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|asset      | Addr | Address of desired SNIP-20 asset

##### Response
```json
{
  "allocations": {
    "allocations": [
      {
        "allocation": {},
      },
      ..
    ],
  }
}
```

#### PendingAllowance
Get the pending allowance for a given asset

##### Request
|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|asset      | Addr | Address of desired SNIP-20 asset

##### Response
```json
{
  "pending_allowance": {
    "amount": "100000",
  }
}
```
