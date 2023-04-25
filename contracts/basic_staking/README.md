# Snip20 Staking
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Interface](#Interface)
        * Messages
            * [Receive](#Receive)
            * [UpdateConfig](#UpdateConfig)
            * [RegisterRewards](#RegisterRewards)
            * [Unbond](#Unbond)
            * [Withdraw](#Withdraw)
            * [Claim](#Claim)
            * [Compound](#Compound)
            * [CancelRewardPool](#CancelRewardPool)
            * [TransferStake](#TransferStake)
        * Queries
            * [Config](#Config)
            * [StakeToken](#StakeToken)
            * [StakingInfo](#StakingInfo)
            * [TotalStaked](#TotalStaked)
            * [RewardTokens](#RewardTokens)
            * [RewardPools](#RewardPools)
            * [Balance](#Balance)
            * [Staked](#Staked)
            * [Rewards](#Rewards)
            * [Unbonding](#Unbonding)

# Introduction
This contract allows users to lock up their 'stake_token', with a configurable unbonding period. Staking users will earn rewards from all active reward pools based on their stake amount / total staked.
Rewards will be initialized by sending in an amount of tokens to be emitted, with start/end timestamps for the rewards period.
Reward pools can be initialized with any registered reward token (admin-only registration). Admins can always init a reward pool (known as 'official'), there is also a configurable 'max_user_pools' that determines how many pools are allowed at 1 time that can be initialized permissionlessly (by any user)

# Sections

## Init
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
| admin_auth | Contract | shade admin authentication contract
| query_auth | Contract | shade query authentication contract
| stake_token | Contract  | token that will be deposited for staking 
| unbond_period | Uint128 | How long it takes to unbond funds in seconds
| max_user_pools | Uint128 | How many permissionless pools are allowed
| reward_cancel_threshold | Uint128 | Percentage of rewards that must be claimed for a reward pool to be cancelled without 'force'
| viewing_key | String | Contract viewing key for snip20's

## Interface

### Messages
#### UpdateConfig
Updates the given values
##### Request
|Name      |Type      |Description                                                                                                        | optional |
|----------|----------|-------------------------------------------------------------------------------------------------------------------|----------|
| admin_auth | Contract | shade admin authentication contract
| query_auth | Contract | shade query authentication contract
| unbond_period | Uint128 | How long it takes to unbond funds in seconds
| max_user_pools | Uint128 | How many permissionless pools are allowed
| reward_cancel_threshold | Uint128 | Percentage of rewards that must be claimed for a reward pool to be cancelled without 'force'

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
