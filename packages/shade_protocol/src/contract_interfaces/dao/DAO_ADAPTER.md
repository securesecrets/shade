# DAO Adapter Interface
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Interface](#Interface)
        * Messages
            * [Unbond](#Unbond)
            * [Claim](#Claim)
            * [Update](#Update)
        * Queries
            * [Balance](#Balance)
            * [Unbonding](#Unbonding)
            * [Claimable](#Claimable)
            * [Unbondable](#Unbondable)

# Introduction
This is an interface for dapps to follow to integrate with the DAO, to receive funding fromthe treasury and later unbond those funds back to treasury when needed. 
NOTE: Because of how the contract implements this, all messages will be enclosed as:
```
{
  "adapter": {
    <msg>
  }
}
```

# Sections

### Messages
#### Unbond
Begin unbonding of a given amount from a given asset

##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset     | Addr |  SNIP-20 asset to unbond

##### Response
```json
{
  "unbond": {
    "amount": "100"
    "status": "success"
  }
}
```

#### Claim
Claim a given amount from completed unbonding of a given asset

##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset     | Addr |  SNIP-20 asset to unbond

##### Response
```json
{
  "claim": {
    "amount": "100"
    "status": "success"
  }
}
```

#### Update
Update a given asset on the adapter, to perform regular maintenance tasks if needed
Examples:
 - `scrt_staking` - Claim rewards and restake
 - `treasury` - Rebalance funds

##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset     | Addr |  SNIP-20 asset to unbond

##### Response
```json
{
  "update": {
    "status": "success"
  }
}
```

### Queries

#### Balance
Get the balance of a given asset, Error if unrecognized

##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset     | Addr |  SNIP-20 asset to query

##### Response
```json
{
  "balance": {
    "amount": "100000",
  }
}
```

#### Unbonding
Get the current unbonding amount of a given asset, Error if unrecognized

##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset     | Addr |  SNIP-20 asset to query

##### Response
```json
{
  "unbonding": {
    "amount": "100000",
  }
}
```

#### Claimable
Get the current claimable amount of a given asset, Error if unrecognized

##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset     | Addr | SNIP-20 asset to query

##### Response
```json
{
  "claimable": {
    "amount": "100000",
  }
}
```

#### Unbondable
Get the current unbondable amount of a given asset, Error if unrecognized

##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|asset     | Addr | SNIP-20 asset to query

##### Response
```json
{
  "unbondable": {
    "amount": "100000",
  }
}
```
