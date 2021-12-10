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
|admin     | string   |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |
|sscrt     | Contract |  sSCRT snip20 token contract |  no      |
|band      | Contract |  Band protocol contract   |  no      |

##User

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
```json
{
  "update_config": {
    "status": "success"
  }
}
```

### Queries

#### Price
Get asset price
##### Request
|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|symbol      | string |  asset abbreviation e.g. BTC/ETH/SCRT;   |  no      |
##### Response
```json
{
  {
    "rate": "1470000000000000000",
    "last_updated_base": 1628569146,
    "last_updated_quote": 3377610
  }
}
```
#### Prices
Get prices of list of assets
##### Request
|Name        |Type    |Description                                                                                                            | optional |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------|----------|
|symbols      | list |  list of asset symbols e.g. BTC/ETH/SCRT;   |  no      |
##### Response
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
