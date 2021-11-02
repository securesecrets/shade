
# Mint Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateConfig](#UpdateConfig)
            * [AddTasks](#AddTasks)
    * [Task_Admin](#Task_Admin)
        * Messages
            * [CompleteTask](#CompleteTask)
    * [User](#User)
        * Messages
            * [Claim](#Claim)
        * Queries
            * [GetConfig](#GetConfig)
            * [GetDates](#GetDates)
            * [GetEligibility](#GetEligibility)
    
# Introduction
Contract responsible to handle snip20 airdrop

# Sections 

## Init
##### Request
|Name          |Type           |Description                                           | optional |
|--------------|---------------|------------------------------------------------------|----------|
|admin         | String        | New contract owner; SHOULD be a valid bech32 address |  yes     |
|airdrop_token | Contract      | The token that will be airdropped                    |  no      |
|start_time    | u64           | When the airdrop starts in UNIX time                 |  yes     |
|end_time      | u64           | When the airdrop ends in UNIX time                   |  yes     |
|rewards       | Rewards       | The total rewards for all the addresses              |  no      |
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
|start_time    | u64        | When the airdrop starts in UNIX time                  |  yes     |
|end_time      | u64        | When the airdrop ends in UNIX time                    |  yes     |

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
Get the contracts airdrop timeframe
```json
{
  "dates": {
    "start": "Airdrop start",
    "end": "Airdrop end"
  }
}
```

## GetEligibility
Get the contract's eligibility per user
##### Request
|Name    |Type    |Description                          | optional |
|--------|--------|-------------------------------------|----------|
|address | String | The address to check eligibility of | no       |
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