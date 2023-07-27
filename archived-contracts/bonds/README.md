
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
        * Queries
            * [Config](#Config)
            * [BondOpportunities](#BondOpportunities)
            * [Account](#Account)
            * [DepositAddresses](#DepositAddresses)
            * [BondInfo](#BondInfo)
            * [PriceCheck](#PriceCheck)
            * [CheckAllowance](#CheckAllowance)
            * [CheckBalance](#CheckBalance)

# Introduction
Generic contract responsible for protocol and treasury bond opportunities
# Sections

## Init
##### Request
| Name                              | Type      | Description                                                                | optional |
|-----------------------------------|-----------|----------------------------------------------------------------------------|----------|
| limit_admin                       | Addr | Limit Assembly/Admin; SHOULD be a valid bech32 address                     | no       |
| global_issuance_limit             | Uint128   | Total number of tokens this contract can issue before limit reset          | no       |
| global_minimum_bonding_period     | u64       | Minimum amount of time before any pending bonds can be claimed.            | no       |
| global_maximum_discount           | Uint128   | Maximum allowed discount for any bond opportunities                        | no       |
| admin                             | Addr | Bonds Assembly/Admin; SHOULD be a valid bech32 address                     | no       |
| oracle                            | Contract  | Oracle contract                                                            | no       |
| treasury                          | Addr | Treasury address for allowance and deposit assets                       | no       |
| issued_asset                      | Contract  | Issued asset for this bonds contract                                       | no       |
| activated                         | bool      | Turns entering opportunities contract-wide on/off                          | no       |
| bond_issuance_limit               | Uint128   | Default issuance limit for new bond opportunities                          | no       |
| bonding_period                    | u64       | Default time for new opportunity before its pending bonds can be claimed   | no       |
| discount                          | Uint128   | Default percent discount on issued asset for new bond opportunities        | no       |
| global_min_accepted_issued_price  | Uint128   | Min price for issued asset. Opps will never issue at lower price than this | no       |
| global_err_issued_price           | Uint128   | Asset price that will fail transaction due to risk                         | no       |
| allowance_key                     | String    | Entropy for generating snip20 viewing key for issued asset. Arbitrary.     | no       |
| airdrop                           | Contract  | Airdrop contract for completing bond task and unlocking % of drop          | yes      |

## Admin

### Messages

#### UpdateConfig
Updates the given values
##### Request
| Name                              | Type      | Description                                                                                   | optional  |
|-----------------------------------|-----------|-----------------------------------------------------------------------------------------------|-----------|
| admin                             | Addr | New contract admin; SHOULD be a valid bech32 address                                          | yes       |
| oracle                            | Contract  | Oracle address                                                                                | yes       |
| treasury                          | Addr | Treasury address                                                                              | yes       |
| issued_asset                      | Contract  | The asset this bond contract will issue to users                                              | yes       |
| activated                         | bool      | If true, bond opportunities can be entered into                                               | yes       |
| minting_bond                      | bool      | If true, bond is minting issued asset. If false, bond is spending on allowance from treasury  | yes       |
| bond_issuance_limit               | Uint128   | Default issuance limit for any new opportunities                                              | yes       |
| bonding_period                    | Uint128   | Default bonding period in UNIX time for any new opportunities                                 | yes       |
| discount                          | Uint128   | Default discount % for any new opportunities                                                  | yes       |
| global_min_accepted_issued_price  | Uint128   | SMin price for issued asset. Opps will never issue at lower price than this                   | yes       |
| global_err_issued_price           | Uint128   | Asset price that will fail transaction due to risk                                            | yes       |
| airdrop                           | Contract  | Airdrop contract for completing bond task and unlocking % of drop                             | yes      |

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
| Name                          | Type      | Description                                       | optional  |
|-------------------------------|-----------|---------------------------------------------------|-----------|
| deposit_asset                 | Contract  | Contract for deposit asset                        | no        |
| start_time                    | u64       | When the opportunity opens in UNIX time           | no        |
| end_time                      | u64       | When the opportunity closes in UNIX time          | no        |
| bond_issuance_limit           | Uint128   | Issuance limit for this opportunity               | yes       |
| bonding_period                | u64       | Bonding period for this opportunity in UNIX time  | yes       |
| discount                      | Uint128   | Discount % for this opportunity                   | yes       |
| max_accepted_deposit_price    | Uint128   | Maximum accepted price for deposit asset       | no        |
| err_deposit_price             | Uint128   | Price for deposit asset that causes error      | no        |
| minting_bond                  | bool      | True for minting from snip20, false for allowance | no        |
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
    "max_accepted_deposit_price": "maximum price accepted for deposit asset Uint128",
    "err_deposit_price": "error-causing price limit for deposit asset Uint128",
    "minting_bond": "bool whether bond opp is a minting bond or not"
  }
}
```

#### CloseBond
Closes bond opportunity for a given asset

##### Request
| Name             | Type     | Description                   | optional  |
|------------------|----------|-------------------------------|-----------|
| deposit_asset    | Contract | Contract for deposit asset    | no        |

##### Response
```json
{
  "close_bond": {
    "status": "success",
    "deposit_asset": "contract for asset who's opportunity was just closed"
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
| limit_admin                   | Addr | New contract limit admin; SHOULD be a valid bech32 address  | yes       |
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
| account      | Addr  | Accounts address            | yes      |
| permit       | Permit     | User's signed permit        | no       |

##### Response
```json
{
  "account": {
    "pending_bonds": "List of pending bonds Vec<PendingBond>",
  }
}
```

#### DepositAddresses
Get the list of addresses for currently recognized deposit addresses, correlated to the open Bond Opportunities

##### Response
```json
{
  "deposit_addresses": {
    "deposit_addresses": "List of deposit addresses Vec<Addr>",
  }
}
```

#### BondInfo
Gets this contracts issuance and claimed totals, as well as the issued asset

##### Response
```json
{
  "bond_info": {
    "global_total_issued": "global total issued Uint128",
    "global_total_claimed": "global total claimed Uint128",
    "issued_asset": "native/issued asset Snip20Asset",
    "global_min_accepted_issued_price": "global minimum accepted price for issued asset Uint128",
    "global_err_issued_price": "global error limit price for issued asset Uint128"
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

#### CheckAllowance
Views this bond contract's allowance from its current Treasury address

##### Response
```json
{
  "check_allowance": {
    "allowance": "current queried allowance Uint128"
  }
}
```

#### CheckBalance
Views this bond contract's current balance for its issued asset

##### Response
```json
{
  "check_balance": {
    "check_balance": "current balance Uint128"
  }
}
```

## Account
User account, stores address

### Structure
| Name            | Type              | Description                                                           | optional  |
|-----------------|-------------------|-----------------------------------------------------------------------|---------- |
| address         | Addr         | User address                                                          | no        |
| pending_bonds   | Vec<PendingBond>  | Bond opportunities purchased by user that are unclaimed and maturing  | no        |


## PendingBond
Stored within user's pending_bonds vector.

NOTE: The parameters must be in order
### Structure
| Name            | Type        | Description                                                                             | optional  |
|-----------------|-------------|-----------------------------------------------------------------------------------------|---------- |
| deposit_denom   | Snip20Asset | Snip20 information for issued asset                                                     | no        |
| end_time        | u64         | Time that bond will be matured and claimable in UNIX time                               | no        |                                 
| deposit_amount  | Uint128     | Amount of issued asset when opportunity was purchased                                   | no        |
| deposit_price   | Uint128     | Price of deposit asset when opportunity was purchased                                | no        |
| claim_amount    | Uint128     | Amount of issued asset set to be claimed                                                | no        |
| claim_price     | Uint128     | Price of issued asset when opportunity was purchased                                    | no        |
| discount        | Uint128     | Discount of issued asset when opportunity was purchased                                 | no        |
| discount_price  | Uint128     | Price of issued asset after discount was applied when opportunity was purchased         | no        |


## BondOpportunity
Stores information for bond opportunity

NOTE: The parameters must be in order
### Structure
| Name                          | Type        | Description                                                           | optional  |
|-------------------------------|-------------|-----------------------------------------------------------------------|---------- |
| issuance_limit                | Uint128     | Issuance limit for this bond opportunity                              | no        |
| amount_issued                 | Uint128     | Amount of issued asset when opportunity was purchased                 | no        |
| deposit_denom                 | Snip20Asset | Snip20 information for issued asset                                   | no        |
| start_time                    | u64         | Time that bond opportunity will be open in UNIX time                  | no        |                                 
| end_time                      | u64         | Time that bond opportunity will be closed in UNIX time                | no        |                                 
| bonding_period                | u64         | Time that users that enter the opportunity must wait before claiming  | no        |
| discount                      | Uint128     | Discount of issued asset when opportunity was purchased               | no        |
| max_accepted_deposit_price    | Uint128     | Maximum accepted price for deposit asset                              | no        |
| err_deposit_price             | Uint128     | Error-causing limit price for deposit                                 | no        |
| minting_bond                  | bool        | True for minting from snip20, false for allowance                     | no        |

## SlipMsg
Stores the user's slippage limit when entering bond opportunities

```json
{
  "slip_msg": {
    "minimum_expected_amount": "minimum expected amount to be issued Uint128"
  }
}
```

## AccountProofMsg
The information inside permits that validate account ownership

NOTE: The parameters must be in order
### Structure
| Name      | Type         | Description                                             | optional |
|-----------|--------------|---------------------------------------------------------|----------|
| contracts | Vec<String>  | Bond contracts the permit is good for                   | no       |
| key       | String       | Some permit key                                         | no       |


## PermitSignature
The signature that proves the validity of the data

NOTE: The parameters must be in order
### Structure
| Name      | Type   | Description               | optional |
|-----------|--------|---------------------------|----------|
| pub_key   | pubkey | Signer's public key       | no       |
| signature | String | Base 64 encoded signature | no       |

## Pubkey
Public key

NOTE: The parameters must be in order
### Structure
| Name  | Type   | Description                        | optional |
|-------|--------|------------------------------------|----------|
| type  | String | Must be tendermint/PubKeySecp256k1 | no       |
| value | String | The base 64 key                    | no       |