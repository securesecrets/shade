# Query Authentication
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [SetAdmin](#SetAdmin)
            * [SetRunState](#SetRunState)
    * [User](#User)
        * Messages
            * [SetViewingKey](#SetViewingKey)
            * [CreateViewingKey](#CreateViewingKey)
            * [BlockPermitKey](#BlockPermitKey)
        * Queries
            * [Config](#Config)
            * [ValidateViewingKey](#ValidateViewingKey)
            * [ValidatePermit](#ValidatePermit)

# Introduction
User authentication manager that allows for validation for permits and viewing keys, making all smart contracts 
share one viewing key.
# Sections

## Init
##### Request
| Name      | Type      | Description                                    | optional |
|-----------|-----------|------------------------------------------------|----------|
| admin     | Addr | Contract admin                                 | yes      |
| prng_seed | Binary    | Randomness seed for the viewing key generation | no       |

## Admin

### Messages

#### SetAdmin
Changes the current admin
##### Request
| Name    | Type      | Description                                          | optional |
|---------|-----------|------------------------------------------------------|----------|
| admin   | Addr | New contract admin; SHOULD be a valid bech32 address | no       |
| padding | String    | Randomly generated data to pad the message           | yes      |


##### Response
``` json
{
  "update_config": {
    "status": "success"
  }
}
```

#### SetRunState
Limits the smart contract's run state
##### Request
| Name    | Type           | Description                                                       | optional |
|---------|----------------|-------------------------------------------------------------------|----------|
| state   | ContractStatus | Limits what queries / handlemsgs can be triggered in the contract | no       |
| padding | String         | Randomly generated data to pad the message                        | yes      |

#### ContractStatus
* Default
* DisablePermit
* DisableVK
* DisableAll

##### Response
``` json
{
  "update_config": {
    "status": "success"
  }
}
```

## User

### Messages

#### SetViewingKey
Sets the signers viewing key
##### Request
| Name    | Type   | Description                                | optional |
|---------|--------|--------------------------------------------|----------|
| key     | String | The new viewing key                        | no       |
| padding | String | Randomly generated data to pad the message | yes      |

##### Response
``` json
{
  "update_config": {
    "status": "success"
  }
}
```

#### CreateViewingKey
Generated the signers viewing key with the given entropy
##### Request
| Name    | Type   | Description                                | optional |
|---------|--------|--------------------------------------------|----------|
| entropy | String | The entropy used for VK generation         | no       |
| padding | String | Randomly generated data to pad the message | yes      |

##### Response
``` json
{
  "update_config": {
    "key": "new VK"
  }
}
```

#### BlockPermitKey
Blocks a permit key, whenever a permit with that key is queried then it will return that its not valid
##### Request
| Name    | Type   | Description                                | optional |
|---------|--------|--------------------------------------------|----------|
| key     | String | Permit key to block                        | no       |
| padding | String | Randomly generated data to pad the message | yes      |

##### Response
``` json
{
  "update_config": {
    "status": "success"
  }
}
```

### Queries

#### Config
Get the contracts config

##### Response
```json
{
  "config": {
    "admin": "address",
    "state": "contract state"
  }
}
```

#### ValidateViewingKey
Validates the users viewing key

##### Request
| Name | Type      | Description        | optional |
|------|-----------|--------------------|----------|
| user | Addr | User to verify     | no       |
| key  | String    | User's viewing key | no       |

##### Response
```json
{
  "validate_viewing_key": {
    "is_valid": true
  }
}
```

#### ValidatePermit
Validates the users permit

##### Request
| Name         | Type       | Description                 | optional |
|--------------|------------|-----------------------------|----------|
| permit       | Permit     | User's signed permit        | no       |

#### Permit
```json
{
  "params": {
    "data": "base64 data specific to the contract",
    "key": "permit key"
  },
  "signature": {
    "pub_key": {
      "type": "tendermint/PubKeySecp256k1",
      "value": "Secp256k1 PubKey"
    },
    "signature": "base64 signature of permit"
  },
  "account_number": "optional account number",
  "chain_id": "optional chain id",
  "sequence": "optional sequence",
  "memo": "Optional memo"
}
```

##### Response
NOTE: is revoked refers to if the permit's key has been blocked
```json
{
  "validate_permit": {
    "user": "Signer's address",
    "is_revoked": false
  }
}
```