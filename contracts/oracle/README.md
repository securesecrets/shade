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
|owner     | string   |  New contract owner; SHOULD be a valid bech32 address, but contracts may use a different naming scheme as well    |  yes     |

##User

### Queries

#### GetScrtPrice
Get SCRT's price according to band protocol.
##### Response
```json
{
  "get_scrt_price": {
  }
}
```