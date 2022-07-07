# Shade Swap LP Providing and Bonding
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
|viewing_key | String  | Viewing Key to be set for any relevant SNIP-20
|token_a   | Contract  |  One token to be provided to the pool
|token_b   | Contract  |  Other token to be provided to the pool
|pool      | Contract  |  Pool contract to provide LP to
|bonding   | Contract  |  Contract to bond LP for rewards

## Interface

### Messages
#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
|config    | Config    |  contract designated to receive all outgoing funds

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
