# Router Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [RecoverFunds](#RecoverFunds)
    * [User](#User)
        * Messages
            * [Receive](#Receive)
            * [SwapTokensForExact](#SwapTokensForExact)
            * [RegisterSNIP20Token](#RegisterSNIP20Token)
        * Queries
            * [SwapSimulation](#SwapSimulation)            
    * [Hooks](#Hooks)
        * Messages
            ** [SwapCallBack](#SwapCallBack)
    * [Invoke](#Invoke)
        * Messages
            * [SwapTokensForExact](#SwapTokensForExact)

# Introduction
The Router is stateless between transactions and can be replaced safely except for view keys specific to the SNIP20 to be traded. Before swapping the router contract, make sure all SNIP20s to be traded in the new contract are registered. This is to ensure upgradability of functionality with minimal impact.
Stateful data is stored within the factory.

# Sections
## Init

Initialize a Router Contract

|Name|Type|Description|Optional|
|-|-|-|-|
|prng_seed|Binary|Seed used for generated viewing key|no|
|entropy|Binary|Entropy used for generated viewing key|no|
|admin_auth|Contract|The Contract used for admin authentication|no|

## Admin
### Messages

#### RecoverFunds

It is used to recover by sending some funds to a specific address.
##### Request
|Name|Type|Description|Optional|
|-|-|-|-|
| token | TokenType | Token type information| no|
| amount | Unit128 | The amount to recover | no       |
| to | HumanAddr | The address to send the amount to         | no       |
| msg | Binary | Message to pass in the send         | yes       |


##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

## User
### Messages


#### RegisterSNIP20Token

Register the router's viewing key with SNIP20 contract. This is required to verify the amount of tokens that the router contract receives on each step of a swap.
##### Request
|Name|Type|Description|Optional|
|-|-|-|-|
|token_addr|String|Register the viewing key for the router to the SNIP20 Token Contract|no|
|token_code_hash|String|Token code hash used to verify the contract that is being registered|no|

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### Receive

Extension of the SNIP20 receive callback used when receiving SNIP20 tokens used for trades.

##### Request
|Name|Type|Description|Optional|
|-|-|-|-|
| from | HumanAddr | who invokes the callback                  | no      |
| amount | Uint128 | amount sent               | no       |
| msg | Binary | Message to Invoke in Pair Contract                  | yes       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```
#### SwapTokensForExact

Used to trade the native token. Calls to this interface directly sending a SNIP20 token will not work, instead use the SNIP20 send with a embedded invoke.
##### Request
|Name|Type|Description|Optional|
|-|-|-|-|
|offer|TokenAmount|The native token amount sent into the start of the router trade|no|
|expected_return|Binary|When given, the minimum amount of tokens that need to come out of the router trade|yes|
|path|Vec(Hop)|The pair addresses in a array used for each leg of the trade|no|
|recipient|String|Specify a recepient besides the sender of the native token|yes|


##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```
### Queries


#### SwapSimulation
Simulates the execution of a swap and returns the estimated values.
##### Request
|Name|Type|Description|Optional|
|-|-|-|-|
|offer|TokenAmount|The native token amount sent into the start of the router trade|no|
|path|Vec(Hop)|The pair addresses in a array used for each leg of the trade|no|

##### Response
```json
{
  "total_fee_amount": "Uint128",
  "lp_fee_amount": "Uint128",
  "shade_dao_fee_amount": "Uint128",
  "result": "SwapResult",
  "price": "String"  
}
```
#### GetConfig
Gets the fonfiguration of a router.
##### Request
|Name|Type|Description|Optional|
|-|-|-|-|
|||||


##### Response
```json
{
}
```

## Hooks
### Messages
#### SwapCallBack

Swap callback is called by the pair contract after completing a trade to initialize the next step in the trade.
##### Request
|Name|Type|Description|Optional|
|-|-|-|-|
|last_token_out|TokenAmount|The token coming out from the pair contract trade|no|
|signature|Binary|Signature to verify correct contract is calling back|no|

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```
<!-- 
## Invoke
### Messages
#### SwapTokensForExact

Used with SNIP20 Send message to initiate router swap.
##### Request
|Name|Type|Description|Optional|
|-|-|-|-|
|expected_return|Binary|When given, the minimum amount of tokens that need to come out of the router trade|yes|
|path|Vec(Hop)|The pair addresses in a array used for each leg of the trade|no|
|recipient|String|Specify a recepient besides the sender of the SNIP20 token|no|

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
``` -->

## Best Path
Best path is calculated within the client, when invoking a swap that path is then provided to the router.

