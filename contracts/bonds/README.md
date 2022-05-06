
# Bonds Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [OpenBond](#OpenBond)
            * [CloseBond](#CloseBond)
    * [Limit_Admin](#Limit_Admin)
        * Messages
            * [UpdateLimitConfig](#UpdateLimitConfig)
    * [User](#User)
        * Messages
            * [Receive](#Receive)
            * [Claim](#Claim)
            * [SetViewingKey](#SetViewingKey)
        * Queries
            * [Config](#Config)
            * [BondOpportunities](#BondOpportunities)
            * [Account](#Account)
            * [CollateralAddresses](#CollateralAddresses)
            * [BondInfo](#BondInfo)
            * [PriceCheck](#PriceCheck)

# Introduction
Contract responsible to handle snip20 airdrop

# Sections

## Init
##### Request
| Name                            | Type      | Description                                                                | optional |
|---------------------------------|-----------|----------------------------------------------------------------------------|----------|
| limit_admin                     | HumanAddr | New contract owner; SHOULD be a valid bech32 address                       | no       |
| global_issuance_limit           | Uint128   | Where the decay amount will be sent                                        | no       |
| global_minimum_bonding_period   | u64       | The token that will be airdropped                                          | no       |
| global_maximum_discount         | Uint128   | Total airdrop amount to be claimed                                         | no       |
| admin                           | HumanAddr | When the airdrop starts in UNIX time                                       | mo       |
| oracle                          | Contract  | When the airdrop ends in UNIX time                                         | no       |
| treasury                        | HumanAddr | When the airdrop decay starts in UNIX time                                 | no       |
| issued_asset                    | Contract  | Base 64 encoded merkle root of the airdrop data tree                       | no       |
| activated                       | bool      | Total accounts in airdrop (needed for merkle proof)                        | no       |
| minting_bond                    | bool      | Used to limit the user permit amounts (lowers exploit possibility)         | no       |
| bond_issuance_limit             | Uint128   | The default amount to be gifted regardless of tasks                        | no       |
| bonding_period                  | u64       | The amounts per tasks to gift                                              | no       |
| discount                        | Uint128   | To prevent leaking information, total claimed is rounded off to this value | no       |
| global_minimum_issued_price     | Uint128   | To prevent leaking information, total claimed is rounded off to this value | no       |
| allowance_key                   | String    | To prevent leaking information, total claimed is rounded off to this value | yes      |

## Admin

### Messages

#### UpdateConfig
Updates the given values
##### Request
| Name                        | Type      | Description                                                                                   | optional  |
|-----------------------------|-----------|-----------------------------------------------------------------------------------------------|-----------|
| admin                       | HumanAddr | New contract admin; SHOULD be a valid bech32 address                                          | yes       |
| oracle                      | Contract  | Oracle address                                                                                | yes       |
| treasury                    | HumanAddr | Treasury address                                                                              | yes       |
| issued_asset                | Contract  | The asset this bond contract will issue to users                                              | yes       |
| activated                   | bool      | If true, bond opportunities can be entered into                                               | yes       |
| minting_bond                | bool      | If true, bond is minting issued asset. If false, bond is spending on allowance from treasury  | yes       |
| bond_issuance_limit         | Uint128   | Default issuance limit for any new opportunities                                              | yes       |
| bonding_period              | Uint128   | Default bonding period in UNIX time for any new opportunities                                 | yes       |
| discount                    | Uint128   | Default discount % for any new opportunities                                                  | yes       |
| global_minimum_issued_price | Uint128   | Sets the floor price the issued asset can be at before all bond opportunities lock            | yes       |

##### Response
``` json
{
  "update_config": {
    "status": "success"
  }
}
```

#### OpenBond
Opens new bond opportunity for a unique asset

##### Request
| Name                  | Type      | Description                                       | optional  |
|-----------------------|-----------|---------------------------------------------------|-----------|
| collateral_asset      | Contract  | Contract for collateral asset                     | no        |
| start_time            | u64       | When the opportunity opens in UNIX time           | yes       |
| end_time              | u64       | When the opportunity closes in UNIX time          | yes       |
| bond_issuance_limit   | Uint128   | Issuance limit for this opportunity               | yes       |
| bonding_period        | u64       | Bonding period for this opportunity in UNIX time  | yes       |
| discount              | Uint128   | Discount % for this opportunity                   | yes       |
| max_collateral_price  | Uint128   | Maximum accepted price for collateral asset       | yes       |

##### Response
```json
{
  "open_bond": {
    "status": "success",
    "deposit_contract": "Contract",
    "start_time": "u64 start in UNIX time",
    "end_time": "u64 end in UNIX time",
    "bond_issuance_limit": "opportunity limit Uint128",
    "bonding_period": "u64 bonding period in UNIX time",
    "discount": "opportunity discount percentage Uint128",
    "max_collateral_price": "maximum price accepted for collateral asset Uint128",
  }
}
```

#### CloseBond
Closes bond opportunity for a given asset

##### Request
| Name             | Type     | Description                   | optional  |
|------------------|----------|-------------------------------|-----------|
| collateral_asset | Contract | Contract for collateral asset | no        |

##### Response
```json
{
  "close_bond": {
    "status": "success"
  }
}
```

## Limit Admin

### Messages

#### UpdateLimitConfig
Update the given limit config values
##### Request
| Name                          | Type      | Description                                                 | optional  |
|-------------------------------|-----------|-------------------------------------------------------------|-----------|
| limit_admin                   | HumanAddr | New contract limit admin; SHOULD be a valid bech32 address  | yes       |
| global_isuance_limit          | Uint128   | asset issuance limit, cumulative across all opportunities   | yes       |
| global_minimum_bonding_period | u64       | minimum bonding time for all opportunities, in UNIX time    | yes       |
| global_maximum_discount       | Uint128   | maximum percent discount for all new opportunities          | yes       |
| reset_total_issued            | bool      | if true, resets global_total_issued to 0                    | yes       |
| reset_total_claimed           | bool      | if true, resets global_total_claimed to 0                   | yes       |

##### Response
```json
{
  "update_limit_config": {
    "status": "success"
  }
}
```

## User

### Messages

#### Receive
To mint the user must use a supported asset's send function and send the amount over to the contract's address. The contract will take care of the rest.
In the msg field of a snip20 send command you must send a base64 encoded json like this one
```json
{"minimum_expected_amount": "Uint128" }
```

##### Response
```json
{
  "deposit": {
    "status": "success",
    "deposit_amount": "Deposit amount Uint128",
    "pending_claim_amount": "Claim amount Uint128",
    "end_date": "u64 end time of bonding period in UNIX time",
  }
}
```

#### Claim
The user doesn't need to pass any parameters to claim. Claiming redeems all of a user's Pending Bonds.

##### Response
```json
{
  "claim": {
    "status": "success",
    "amount": "claim amount Uint128",
  }
}
```

#### SetViewingKey
Set's the user's viewing key for their account to whatever string  is passed.

##### Request
| Name         | Type    | Description                                | optional |
|--------------|---------|--------------------------------------------|----------|
| key          | String  | Proof that the user owns those addresses   | no       |

##### Response
```json
{
  "account": {
    "status": "success",
    "total": "Total airdrop amount",
    "claimed": "Claimed amount",
    "unclaimed": "Amount available to claim",
    "finished_tasks": "All of the finished tasks",
    "addresses": ["claimed addresses"]
  }
}
```

### Queries

#### Config
Gets the contract's config

##### Response
```json
{
  "config": {
    "config": "Contract's config"
  }
}
```

#### BondOpportunities
Get the vector of bond opportunities currently available

##### Response
```json
{
  "bond_opportunities": {
    "bond_opportunities": "List of opportunities Vec<BondOpportunity",
  }
}
```

#### Account
Get the account's pending bonds using a viewing key

##### Request
| Name         | Type       | Description                 | optional |
|--------------|------------|-----------------------------|----------|
| account      | HumanAddr  | Accounts address            | yes      |
| key          | String     | Address's viewing key       | no       |

##### Response
```json
{
  "account": {
    "pending_bond": "List of pending bonds Vec<PendingBond>",
  }
}
```

#### CollateralAddresses
Get the list of addresses for currently recognized collateral addresses, correlated to the open Bond Opportunities

##### Response
```json
{
  "collateral_addresses": {
    "collateral_addresses": "List of collateral addresses Vec<HumanAddr>",
  }
}
```

#### BondInfo
Gets this contracts issuance and claimed totals, as well as the issued asseet

##### Response
```json
{
  "bond_info": {
    "global_total_issued": "global total issued Uint128",
    "global_total_claimed": "global_total_claimed Uint128",
    "issued_asset": "native/issued asset Snip20Asset",
  }
}
```

#### PriceCheck
Gets the price for the passed asset by querying the oracle registered in the config

##### Response
```json
{
  "price_check": {
    "price": "price of passed asset in dollars Uint128",
  }
}
```

## Account
User account, stores address

### Structure
| Name            | Type              | Description                                                           | optional  |
|-----------------|-------------------|-----------------------------------------------------------------------|---------- |
| address         | HumanAddr         | User address                                                          | no        |
| pending_bonds   | Vec<PendingBond>  | Bond opportunities purchased by user that are unclaimed and maturing  | no        |


## PendingBond
Stored within user's pending_bonds vector.

NOTE: The parameters must be in order
### Structure
| Name            | Type        | Description                                                                             | optional  |
|-----------------|-------------|-----------------------------------------------------------------------------------------|---------- |
| deposit_denom   | Snip20Asset | Snip20 information for issued asset                                                     | no        |
| end             | u64         | Time that bond will be matured and claimable in UNIX time                               | no        |                                 
| deposit_amount  | Uint128     | Amount of issued asset when opportunity was purchased                                   | no        |
| deposit_price   | Uint128     | Price of collateral asset when opportunity was purchased                                | no        |
| claim_amount    | Uint128     | Amount of issued asset set to be claimed                                                | no        |
| claim_price     | Uint128     | Price of issued asset when opportunity was purchased                                    | no        |
| discount        | Uint128     | Discount of issued asset when opportunity was purchased                                 | no        |
| discount_price  | Uint128     | Price of issued asset after discount was applied when opportunity was purchased         | no        |


## BondOpportunity
Stores information for bond opportunity

NOTE: The parameters must be in order
### Structure
| Name                  | Type        | Description                                                           | optional  |
|-----------------------|-------------|-----------------------------------------------------------------------|---------- |
| issuance_limit        | Uint128     | Issuance limit for this bond opportunity                              | no        |
| amount_issued         | Uint128     | Amount of issued asset when opportunity was purchased                 | no        |
| deposit_denom         | Snip20Asset | Snip20 information for issued asset                                   | no        |
| start_time            | u64         | Time that bond opportunity will be open in UNIX time                  | no        |                                 
| end_time              | u64         | Time that bond opportunity will be closed in UNIX time                | no        |                                 
| bonding_period        | u64         | Time that users that enter the opportunity must wait before claiming  | no        |
| discount              | Uint128     | Discount of issued asset when opportunity was purchased               | no        |
| max_collateral_price  | Uint128     | Maximum accepted price for collateral asset                           | no        |
