# sSCRT Staking Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [Receive](#Receive)
            * [Unbond](#Unbond)
            * [Claim](#Claim)
        * Queries
            * [GetConfig](#GetConfig)
            * [Delegations](#Delegations)
            * [Delegation](#Delegation)
# Introduction
The sSCRT Staking contract receives sSCRT, redeems it for SCRT, then stakes it with a validator that falls within the criteria it has been configured with. The configured `treasury` will receive all funds from claiming rewards/unbonding.

# Sections

## Init
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | Addr |  contract owner/admin; a valid bech32 address;
|treasury  | Addre |  contract designated to receive all outgoing funds
|sscrt     | Contract |  sSCRT Snip-20 contract to accept for redemption/staking, all other funds will error
|validator_bounds | ValidatorBounds | criteria defining an acceptable validator to stake with

## Admin

### Messages
#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | Addr |  contract owner/admin; a valid bech32 address;
|treasury  | Addre |  contract designated to receive all outgoing funds
|sscrt     | Contract |  sSCRT Snip-20 contract to accept for redemption/staking, all other funds will error
|validator_bounds | ValidatorBounds | criteria defining an acceptable validator to stake with

##### Response
```json
{
  "update_config": {
    "status": "success"
  }
}
```


### Queries

#### GetConfig
Gets the contract's configuration variables
##### Response
```json
{
  "config": {
    "config": {
      "owner": "Owner address",
    }
  }
}
```
