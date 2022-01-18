# Oracle Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [User](#User)
        * Queries
            * [GetScrtPrice](#GetScrtPrice)
# Introduction
The oracle contract is used to query the price of different currencies

# Sections

## Init

##### Request

|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|admin     | string   |  New contract admin; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |
|sscrt     | Contract |  sSCRT snip20 token contract |  no      |
|band      | Contract |  Band protocol contract   |  no      |

## User

### Messages

#### UpdateConfig
Updates the given values

##### Request

|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | string   |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |
|silk      | Contract |  Silk contract                                                                                                    |  no      |
|oracle    | Contract |  Oracle contract                                                                                                  |  no      |

##### Response

|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|status    | string   | Always 'success'                                                                                                  |  no      |

###### Example

```json
{
  "update_config": {
    "status": "success"
  }
}
```

#### RegisterSswapPair 
Registers a Secret Swap pair that can then be queried

##### Request

|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|pair      | Contract |  A Secret Swap Pair contract where one of the tokens must be sSCRT                                                |  no      |

##### Response

|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|status    | string   | Always 'success'                                                                                                  |  no      |

###### Example

```json
{
  "register_sswap_pair": {
    "status": "success"
  }
}
```

### Queries

#### Price
Get asset price according to band protocol or a registered SecretSwap pair

##### Request

|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|symbol      | string |  Asset abbreviation e.g. BTC/ETH/SCRT;                                                                                |  no      |

##### Response

|Name               |Type    |Description                                                                                                     | optional |
|-------------------|--------|----------------------------------------------------------------------------------------------------------------|----------|
|rate               | u128   | The exchange rate of the asset against USD                                                                     |  no      |
|last_updated_base  | u64    | UNIX timestamp of when the base asset price was last updated (0 for SecretSwap pairs)                          |  no      |
|last_updated_quote | u64    | UNIX timestamp of when the quote asset price was last updated (0 for SecretSwap pairs)                         |  no      |

###### Example

```json
{
  "rate": 1470000000000000000,
  "last_updated_base": 1628569146,
  "last_updated_quote": 3377610
}
```

#### Config
Get the current config

#### Prices
Get prices of list of assets
##### Request

|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|symbols      | list |  list of asset symbols e.g. BTC/ETH/SCRT;   |  no      |
##### Response

|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | string   | Contract owner                                                                                                    |  no      |
|band      | Contract | Band contract                                                                                                     |  no      |
|sscrt     | Contract | sSCRT contract                                                                                                    |  no      |

###### Example

Addresses are fictional.

```json
{
  [
    {
      "rate": "1470000000000000000",
      "last_updated_base": 1628569146,
      "last_updated_quote": 3377610
    },
    ...
  ]
}
```

