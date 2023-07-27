# sSCRT Staking Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [DAO Adapter](/packages/shade_protocol/src/DAO_ADAPTER.md)
    * [Interface](#Interface)
        * Messages
            * [Receive](#Receive)
            * [UpdateConfig](#UpdateConfig)
        * Queries
            * [Config](#Config)
            * [Delegations](#Delegations)

# Introduction
The sSCRT Staking contract receives sSCRT, redeems it for SCRT, then stakes it with a validator that falls within the criteria it has been configured with. The configured `treasury` will receive all funds from claiming rewards/unbonding.

# Sections

## Init
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|admin     | Addr |  contract owner/admin; a valid bech32 address;
|treasury  | Addr |  contract designated to receive all outgoing funds
|sscrt     | Contract  |  sSCRT Snip-20 contract to accept for redemption/staking, all other funds will error
|validator_bounds | ValidatorBounds | criteria defining an acceptable validator to stake with
|viewing_key      | String  | Viewing Key to be set for any relevant SNIP-20

## Interface

### Messages
#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|owner     | Addr |  contract owner/admin; a valid bech32 address;
|treasury  | Addr |  contract designated to receive all outgoing funds
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

#### Config
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
