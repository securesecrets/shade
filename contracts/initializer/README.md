
# Mint Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
      * Messages
        * [SetAdmin](#SetAdmin)
        * [InitSilk](#InitSilk)
      * [User](#User)
        * Queries
          * [Config](#Config)
          * [Contracts](#Contracts)
    
# Introduction
Contract responsible to initialize the snip20s and keeping the their initial states public

# Sections

## Init
##### Request
|Name             |Type                |Description                   | optional |
|-----------------|--------------------|------------------------------|----------|
|admin            | String             | Contract;s admin             |  yes     |
|snip20_id        | u64                | The uploaded contract's ID   |  no      |
|snip20_code_hash | String             | The uploaded contract's hash |  no      |
|shade            | Snip20ContractInfo | Initial state for the Snip20 |  no      |

## Admin

### Messages

#### SetAdmin
Sets the contract admin
##### Request
|Name  |Type   |Description                | optional |
|------|-------|---------------------------|----------|
|admin            | String             | Contract;s admin             | no       |


##### Response
```json
{
  "set_admin": {
    "status": "success"
  }
}
```

#### InitSilk
Initializes silk
##### Request
| Name     | Type               | Description                  | optional |
|----------|--------------------|------------------------------|----------|
| shade    | Snip20ContractInfo | Initial state for the Snip20 | no       |
| ticker   | String             | Silk ticker                  | no       |
| decimals | u8                 | Silk decimal places          | no       |

##### Response
```json
{
  "init_silk": {
    "status": "success"
  }
}
```

## User

### Queries

#### Config
Gets the contract's config
#### Response
```json
{
  "config": {
    "config": {
      "admin": "Contract admin",
      "snip20_id": "Snip20 id to allow contract init",
      "snip20_code_hash": "Snip20 code hash needed for the init"
    }
  }
}
```

#### Contracts
Gets the contract's initialized snip20s and their initial balances
#### Response
```json
{
  "contracts": {
    "shade": "Init History",
    "silk": "Init History"
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