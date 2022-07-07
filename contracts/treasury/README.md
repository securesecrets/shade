# Treasury
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [DAO Adapter](/packages/shade_protocol/src/DAO_ADAPTER.md)
    * [Interface](#Interface)
        * Messages
            * [Receive](#Receive)
            * [UpdateConfig](#UpdateConfig)
            * [RegisterAsset](#RegisterAsset)
            * [RegisterManager](#RegisterManager)
            * [Allowance](#Allowance)
            * [AddAccount](#AddAccount)
            * [CloseAccount](#CloseAccount)
        * Queries
            * [Config](#Config)
            * [Assets](#Assets)
            * [Allowances](#Allowances)
            * [CurrentAllowances](#CurrentAllowances)
            * [Allowance](#Allowance)
            * [Account](#Account)
# Introduction
The treasury contract holds network funds from things such as mint commission and pending airdrop funds

# Sections

## Init
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|admin | string   |  contract owner/admin; a valid bech32 address; Controls funds
|viewing_key | string   |  viewing key for all registered snip20 assets
|sscrt | Contract |  sSCRT contract for wrapping & unwrapping

## Interface

### Messages

#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|config | string   |  New config to be set for the contract

##### Response
```json
{
  "update_config": {
    "status": "success"
  }
}
```

#### RegisterAsset
Registers a SNIP-20 compliant asset since [RegisterReceive](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md#RegisterReceive) is called.

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

#### Config
Gets the contract's configuration
##### Response
```json
{
  "config": {
    "config": {
      "admin": "admin address",
      "sscrt": {
        "address": "",
        "code_hash": "",
      },
    }
  }
}
```

#### Assets
List of assets supported
##### Response
```json
{
  "assets": {
    "assets": ["asset address", ...]
  }
}
```

#### Allowances
List of configured allowances for things like treasury_manager & rewards
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset | Addr |  Asset to query balance of
##### Response
```json
{
  "allowances": {
    "allowances": [
    {
      "allowance": ...
    }, 
    ...]
  }
}
```

#### Allowance
List of configured allowances for things like treasury_manager & rewards
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset | Addr |  Asset to query allowance for
|spender | Addr |  Spender of allowance
##### Response
```json
{
  "allowances": {
    "allowances": [
      {
        "allowance": ...
      }, 
      ...
    ]
  }
}
```

#### Accounts
List of account holders
##### Response
```json
{
  "accounts": {
    "accounts": ["address0", ...],
  }
}
```

#### Account
Balance of a given account holders assets (e.g. SHD staking)
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|holder | Addr |  Holder of the account
|asset | Addr |  Asset to query balance of
##### Response
```json
{
  "account": {
    "account": {
      "balances": Uint128,
      "unbondings": Uint128,
      "claimable": Uint128,
      "status": ("active"|"disabled"|"closed"|"transferred"),
    }
  }
}
```
