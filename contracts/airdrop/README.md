
# Airdrop Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [AddTasks](#AddTasks)
            * [ClaimDecay](#ClaimDecay)
    * [Task_Admin](#Task_Admin)
        * Messages
            * [CompleteTask](#CompleteTask)
    * [User](#User)
        * Messages
            * [CreateAccount](#CreateAccount)
            * [UpdateAccount](#UpdateAccount)
            * [Claim](#Claim)
        * Queries
            * [GetConfig](#GetConfig)
            * [GetDates](#GetDates)
            * [GetAccount](#GetAccount)
    
# Introduction
Contract responsible to handle snip20 airdrop

# Sections 

## Init
##### Request
|Name          |Type           |Description                                           | optional |
|--------------|---------------|------------------------------------------------------|----------|
|admin         | String        | New contract owner; SHOULD be a valid bech32 address |  yes     |
|dump_address  | String        | Where the decay amount will be sent                  |  yes     |
|airdrop_token | Contract      | The token that will be airdropped                    |  no      |
|airdrop_amount | String      | Total airdrop amount to be claimed                   |  no      |
|start_date    | u64           | When the airdrop starts in UNIX time                 |  yes     |
|end_date      | u64           | When the airdrop ends in UNIX time                   |  yes     |
|decay_start   | u64          | When the airdrop decay starts in UNIX time           | yes|
|merkle_root  | String        | Base 64 encoded merkle root of the airdrop data tree |  no      |
|total_accounts | String      | Total accounts in airdrop (needed for merkle proof)  |  no      |
|max_amount   | String        | Used to limit the user permit amounts (lowers exploit possibility) |  no|
|default_claim | String        | The default amount to be gifted regardless of tasks  |  no      |
|task_claim    | RequiredTasks | The amounts per tasks to gift                        |  no      |

##Admin

### Messages

#### UpdateConfig
Updates the given values
##### Request
|Name          |Type        |Description                                            | optional |
|--------------|------------|-------------------------------------------------------|----------|
|admin         | string     |  New contract admin; SHOULD be a valid bech32 address |  yes     |
| dump_address | string     | Sets the dump address if there isnt any               |  yes     |
|start_date    | u64        | When the airdrop starts in UNIX time                  |  yes     |
|end_date      | u64        | When the airdrop ends in UNIX time                    |  yes     |
|decay_start   | u64        | When the airdrop decay starts in UNIX time            |  yes |

#### AddTasks
Adds more tasks to complete
##### Request
|Name  |Type   |Description                | optional |
|------|-------|---------------------------|----------|
|Tasks | Tasks | The new tasks to be added | no       |

##### Response
```json
{
  "add_tasks": {
    "status": "success"
  }
}
```

#### ClaimDecay
Drains the decayed amount of airdrop into a dump address

##### Response
```json
{
  "claim_decay": {
    "status": "success"
  }
}
```

##Task Admin

### Messages

#### CompleteTask
Complete that address' tasks for a given user
##### Request
|Name    |Type    |Description                          | optional |
|--------|--------|-------------------------------------|----------|
|address | String | The address that completed the task | no       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

##User

### Messages

### CreateAccount
Creates an account from which the user will claim all of his given addresses' rewards
##### Request
| Name      | Type                             | Description                              | optional |
|-----------|----------------------------------|------------------------------------------|----------|
| addresses | Array of [AddressProofPermit](#AddressProofPermit) | Proof that the user owns those addresses | no       |
| partial_tree | Array of string                  | An array of nodes that serve as a proof for the addresses | no |

##### Response
```json
{
  "create_account": {
    "status": "success"
  }
}
```

### UpdateAccount
Updates a users accounts with more addresses
##### Request
| Name      | Type                        | Description                              | optional |
|-----------|-----------------------------|------------------------------------------|----------|
| addresses | Array of [AddressProofPermit](#AddressProofPermit) | Proof that the user owns those addresses | no       |
| partial_tree | Array of string          | An array of nodes that serve as a proof for the addresses | no |

##### Response
```json
{
  "update_account": {
    "status": "success"
  }
}
```

#### Claim
Claim the user's available claimable amount

##### Response
```json
{
  "claim": {
    "status": "success"
  }
}
```

### Queries

#### GetConfig
Gets the contract's config
#### Response
```json
{
  "config": {
    "config": "Contract's config"
  }
}
```

## GetDates
Get the contracts airdrop timeframe, can calculate the decay factor if a time is given
##### Request
|Name    |Type    |Description                          | optional |
|--------|--------|-------------------------------------|----------|
|current_date | u64 | The current time in UNIX format | yes       |
```json
{
  "dates": {
    "start": "Airdrop start",
    "end": "Airdrop end",
    "decay_start": "Airdrop start of decay",
    "decay_factor": "Decay percentage"
  }
}
```

## GetAccount
Get the account's information
##### Request
|Name    |Type    |Description                          | optional |
|--------|--------|-------------------------------------|----------|
|permit  | [AccountProofPermit](#AccountProofMsg)|Address's permit | no |
|current_date | u64 | Current time in UNIT format       | yes      |
```json
{
  "eligibility": {
    "total": "Total airdrop amount",
    "claimed": "Claimed amount",
    "unclaimed": "Amount available to claim",
    "finished_tasks": "All of the finished tasks"
  }
}
```

## AddressProofPermit
This is a structure used to prove that the user has permission to query that address's information (when querying account info).
This is also used to prove that the user owns that address (when creating/updating accounts) and the given amount is in the airdrop.

NOTE: The parameters must be in order

[How to sign](https://github.com/securesecrets/shade/blob/77abdc70bc645d97aee7de5eb9a2347d22da425f/packages/shade_protocol/src/signature/mod.rs#L100)
#### Structure
| Name       | Type            | Description                                        | optional |
|------------|-----------------|----------------------------------------------------|----------|
| params     | AddressProofMsg | Information relevant to the airdrop information    | no       |
| chain_id   | String          | Chain ID of the network this proof will be used in | no       |
| signature  | PermitSignature | Signature of the permit                            | no       |

## AccountProofMsg
The information inside permits that validate account ownership

NOTE: The parameters must be in order
### Structure
| Name     | Type    | Description                                             | optional |
|----------|---------|---------------------------------------------------------|----------|
| contract | String  | Airdrop contract                                        | no       |
| key      | String  | Some permit key                                         | no       |


## AddressProofMsg
The information inside permits that validate the airdrop eligibility and validate the account holder's key.

NOTE: The parameters must be in order
### Structure
| Name     | Type    | Description                                             | optional |
|----------|---------|---------------------------------------------------------|----------|
| address  | String  | Address of the signer (might be redundant)              | no       |
| amount   | String  | Airdrop amount                                          | no       |                                 
| contract | String  | Airdrop contract                                        | no       |
| index    | Integer | Index of airdrop data in reference to the original tree | no       |
| key      | String  | Some permit key                                         | no       |

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