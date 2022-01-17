
# Mint Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [User](#User)
        * Queries
            * [GetContracts](#GetContracts)
    
# Introduction
Contract responsible to initialize the snip20s and keeping the their initial states public

# Sections

## Init
##### Request
|Name             |Type                |Description                   | optional |
|-----------------|--------------------|------------------------------|----------|
|snip20_id        | u64                | The uploaded contract's ID   |  no      |
|snip20_code_hash | String             | The uploaded contract's hash |  no      |
|shade            | Snip20ContractInfo | Initial state for the Snip20 |  no      |
|silk             | Snip20ContractInfo | Initial state for the Snip20 |  no      |

##User

### Queries

#### GetContracts
Gets the contract's initialized snip20s and their initial balances
#### Response
```json
{
  "contracts": {
    "contracts": ["Init History"]
  }
}
```

## Snip20ContractInfo
Type used to init the snip20s
```json
{
  "snip20_contract_info": {
    "label": "Initialized label",
    "admin": "Optional admin",
    "prng_seed": "Randomizer seed",
    "initial_balances": "Initial snip20 balances"
  }
}
```