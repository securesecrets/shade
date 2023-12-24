SNIP1155 Reference Implementation: Private Multitokens  <!-- omit in toc --> 
==============

This repository contains the [SNIP1155 Base Specifications](#base-specifications) and the [SNIP1155 standard reference implementation](https://github.com/DDT5/snip1155-reference-impl/)

**This is a work in progress**


## Table of contents <!-- omit in toc --> 
- [Abstract](#abstract)
- [Terms](#terms)
- [Base specifications](#base-specifications)
  - [Token hierarchy](#token-hierarchy)
  - [The instantiator](#the-instantiator)
  - [The admin](#the-admin)
  - [The curator(s) and minter(s)](#the-curators-and-minters)
  - [NFT vs fungible tokens](#nft-vs-fungible-tokens)
  - [Handle messages](#handle-messages)
    - [Curate tokenIds](#curate-tokenids)
    - [Mint tokens](#mint-tokens)
    - [Burn tokens](#burn-tokens)
    - [Change metadata](#change-metadata)
    - [Transfer](#transfer)
    - [Send](#send)
    - [Batch transfer and batch send](#batch-transfer-and-batch-send)
    - [Give permission](#give-permission)
    - [Revoke permission](#revoke-permission)
    - [Create viewing key and set viewing key](#create-viewing-key-and-set-viewing-key)
    - [Revoke permit](#revoke-permit)
    - [Add curators and remove curators](#add-curators-and-remove-curators)
    - [Add minters and remove minters](#add-minters-and-remove-minters)
    - [Change admin](#change-admin)
    - [Remove admin](#remove-admin)
  - [Queries](#queries)
    - [Contract info](#contract-info)
    - [TokenId public information](#tokenid-public-information)
    - [Registered code hash](#registered-code-hash)
  - [Authenticated queries](#authenticated-queries)
    - [Balance](#balance)
    - [All balances](#all-balances)
    - [Transaction history](#transaction-history)
    - [Permission](#permission)
    - [All permissions](#all-permissions)
    - [TokenId private information](#tokenid-private-information)
  - [Receiver functions](#receiver-functions)
    - [Register receive](#register-receive)
    - [Snip1155Receive](#snip1155receive)
  - [Miscellaneous](#miscellaneous)
- [Additional specifications](#additional-specifications)
- [Design decisions](#design-decisions)
  - [Reference implementation goals](#reference-implementation-goals)
  - [Starting from a blank slate](#starting-from-a-blank-slate)
  - [Permissionless design](#permissionless-design)
  - [Familiar interface](#familiar-interface)
  - [Modular additional features](#modular-additional-features)


# Abstract

SNIP1155 is a [Secret Network](https://github.com/scrtlabs/SecretNetwork) contract that can create and manage multiple tokens from a single contract instance. Tokens can be a combination of fungible tokens and non-fungible tokens, each with separate attributes, configurations, and metadata. 

This specification writeup ("spec" or "specs") outlines the functionality and interface. The design is loosely based on [CW1155](https://lib.rs/crates/cw1155) which is in turn based on Ethereum's [ERC1155](https://eips.ethereum.org/EIPS/eip-1155), with an additional privacy layer made possible as a Secret Network contract. Fungible and non-fungible tokens are mostly treated equally, but each has a different set of available token configurations which define their possible behaviors. For example, NFTs cannot be configured to be minted more than once. Unlike CW1155 where approvals must cover the entire set of tokens, SNIP1155 contracts allow users to control which tokens fall in scope for a given approval (a feature from [ERC1761](https://eips.ethereum.org/EIPS/eip-1761)), as well as control the type of approval it grants other addresses: token transfer allowances, balance viewership, or private metadata viewership. Also, in SNIP1155, tokens can be configured to allow metadata to be changed.

SNIP1155 introduces distinct and well-defined roles for the instantiator, admin, curators and minters. This allows developers to have granular control over how different parties interact with the contract. Additionally, SNIP1155 allows contracts to be instantiated without an admin. Contracts that were instantiated with admins can break their admin keys any time. These functions are native in SNIP1155 in order to encourage developers to create permissionless applications. 

The ability to hold multiple token types can offer new functionality, improve developer and user experience, and reduce gas fees. For example, users can batch transfer multiple token types in a single transaction, users can view multiple balances from a single viewing key, developers can reduce to use of inter-contract messages and factory contracts, and users may need one approval transaction to cover all tokens for an application.

See [design decisions](#design-decisions) for more details.

Message schemas presented in this specification document are simplified for readability and in order to illustrate functionality. Developers who need detailed API should rely on the [canonical schemas](https://github.com/DDT5/snip1155-reference-impl/tree/master/schema). If there are any discrepancies, the canonical schemas should be used.

Also, [Rust Docs](https://ddt5.github.io/snip1155-doc/snip1155_reference_impl/index.html) are available.


# Terms
*The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).*

This memo uses the terms defined below:
* Message - an on-chain interface. It is triggered by sending a transaction, and receiving an on-chain response which is read by the client. Messages are authenticated both by the blockchain and by the Secret enclave.
* Query - an off-chain interface. Queries are done by returning data that a node has locally, and are not public. Query responses are returned immediately, and do not have to wait for blocks. In addition, queries cannot be authenticated using the standard interfaces. Any contract that wishes to strongly enforce query permissions must implement it themselves.
* Sender, Caller or User - the account found under the sender field in a standard Cosmos SDK message. This is also the signer of the Cosmos message.

# Base specifications

## Token hierarchy
SNIP1155 has a token hierarchy with three layers: (A) SNIP1155 contract > (B) token_id > (C) token(s). 

**(A)** At the highest level, all tokens of a given SNIP1155 contract MUST share:
* admin (if enabled)
* curator(s)

**(B)** A SNIP1155 contract MUST have the ability to hold multiple `tokens_id`s. Each `token_id` MUST have their own:
* token_id (unique identifier within a contract)
* name
* symbol
* `token_config`
* public `metadata`
* private `metadata`


`token_config` MUST be an enum that includes at least these two variants:

```rust
{
  fungible: {
    minters: Vec<HumanAddr>,
    decimals: u8,
    public_total_supply: bool,
    enable_mint: bool,
    enable_burn: bool,
    minter_may_update_metadata: bool,
  }
}
{
  nft: {
    minters: Vec<HumanAddr>,
    public_total_supply: bool,
    owner_is_public: bool,
    enable_burn: bool,
    owner_may_update_metadata: bool,
    minter_may_update_metadata: bool,
  }
}
```

`metadata` MUST be the following struct:
```rust
{
  token_uri: Option<String>,
  extension: Option<Extension>,
}
```

Application developers MAY change `extension` to any `struct` that fits their use case.

**(C)** Each `token_id` can have 1 or more supply of `tokens`, which are indistinguishable from one another (hence fungible). Non-fungible `token_id`s MUST have a total supply of 1, and MUST only be minted once. 

For example, a given SNIP1155 contract may have the following token hierarchy:
```
SNIP1155 contract
├── token_id 0 (total supply = 2)
│   ├── fungible token
│   └── fungible token
├── token_id 1 (total supply = 3)
│   ├── fungible token
│   ├── fungible token
│   └── fungible token
└── token_id 2 (total supply = 1)
    └── non-fungible token
```


The table below gives a summary of these variables: 

| Variable         | Type             | Description                                                      | Optional |
| ---------------- | ---------------- | ---------------------------------------------------------------- | -------- |
| admin            | HumanAddr        | Can to add/remove curators and minters, and break admin key      | Yes      |
| curators         | `Vec<HumanAddr>` | Can curate new token_ids                                         | Yes      |
| minters          | `Vec<HumanAddr>` | Can mint additional fungible tokens and possibly change metadata | Yes      |
| token_id         | String           | A `token_id`'s unique identifier                                 | No       |
| name             | String           | Name of fungible token or NFT. Does not need to be unqiue        | No       |
| symbol           | String           | Symbol of fungible tokens or NFT                                 | No       |
| token_config     | TokenConfig      | Configuration for a specific `token_id`, set by the `curator`    | No       |
| public metadata  | Metadata         | Publicly viewable `uri` and `extension`                          | Yes      |
| private metadata | Metadata         | Non-publicly viewable `uri` and `extension`                      | Yes      |


## The instantiator
The instantiator creates a new instance of a SNIP1155 contract. In that process, the instantiator MUST be able to:
* choose if the contract has an admin, and specify the admin address if applicable
* specify the curator(s) 
* curate and mint initial token balances
* specify the minter(s) for each `token_id` 

If no admin is specified, the instantiator SHOULD be used as the default admin; this set up is for familiarity with SNIP20/721 standards. However, there MUST also be an input field `has_admin: bool` which allows the instantiator to instantiate a no-admin contract. If `has_admin == false`, there MUST be no admin. Any admin input MUST be ignored by the contract.   

The `initial_balances` input SHOULD allow an arbitrary number of `token_id`s and `token`s to be created at instantiation. This design makes it convenient to instantiate permissionless contracts with no admin, curators and minters, as all the required tokens can be minted upon instantiation.


**Instantiation message**
```js
{
  has_admin: boolean,
  admin?: string,
  curators: string[],
  initial_tokens: [{
    token_info: [{
      token_id: string, 
      name: string, 
      symbol: string, 
      token_config: "<token_config>",
      public_metadata?: "<metadata>",
      private_metadata?: "<metadata>",
    }],
    balances: [{
      address: string,
      amount: string,
    }]
  }],
  entropy: string,
} 
```

`token_config` can be one of the two variants below (see [token hierarchy](#token-hierarchy)):
```js
{
  fungible: {
    minters: string[],
    decimals: number,
    public_total_supply: boolean,
    enable_mint: boolean,
    enable_burn: boolean,
    minter_may_update_metadata: boolean,
  }
}
{
  nft: {
    minters: string[],
    public_total_supply: boolean,
    owner_is_public: boolean,
    enable_burn: boolean,
    owner_may_update_metadata: boolean,
    minter_may_update_metadata: boolean,
  }
}
```

`metadata`:
```js
{
  token_uri?: string,
  extension?: "<any object>",
}
```


## The admin
The role of the admin (if exists) is to add and remove curators and minters. 

The admin MUST be able to perform admin-only transactions. Admin-only transactions MUST NOT be callable by non-admins. Admin-only function MUST include `add_curators`, `remove_curators`, `add_minters`, `remove_minters`, `change_admin` and `remove_admin`.

The existence of the `remove_admin` function is enforced in SNIP1155 standards in order to encourage permissionless designs. Users MUST be able to query `contract_info` to get proof on whether an admin role exists.

## The curator(s) and minter(s)
There are two types of roles that can create tokens: 
* `curators` MUST be able to curate new `token_id`s and mints initial balances. They MUST NOT be able to mint additional tokens of existing token_ids, unless they are also minters. 
* `minters` are specific to each token_id. They MUST be able to mint incremental fungible tokens of existing token_ids if the token_id configuration allows this. They MUST NOT be able to mint the initial token balances. Minters of a given token_id can change the public and private metadata if the token_config of the token_id allows this (applies to both fungible tokens and nfts). Minters MUST NOT be able to mint additional non-fungible tokens of existing token_ids. 

## NFT vs fungible tokens
Curators MUST have the option to set public metadata and private metadata for a token_id. Minters of a specific token_id SHOULD be able to change metadata if the configuration allows them to. An NFT owner MUST be able to change the metadata if the token_id configuration allows it to.

Private metadata of an NFT MUST NOT be viewable by any address, other than the owner's address or addresses that have been given permission to[^1]. The base standard implementation allows any owner of a fungible token to view private metadata of the fungible `token_id`, but different rules MAY be implemented in additional specifications.  

[^1]: The base specifications do not have sealed metadata functionality, which is included in the additional specifications.


## Handle messages

### Curate tokenIds
Curators MUST be able to access this function. Other addresses MUST NOT be able to call this function. (Note that admins cannot mint unless it is also a curator). 

A curator MUST be able to create new `token_id`s. The curator MUST be able to configure the token_id and set initial balances. A curator MUST NOT be able to curate a `token_id` that already exists.

`CurateTokenIds` MUST be able to create multiple `token_id`s and set multiple initial balances in a single transaction. Therefore, `BatchCurateTokenIds` is not necesary. 

Message:
```js
{
  curate_token_ids: {
    initial_tokens: [{
      token_info: [{
        token_id: string, 
        name: string, 
        symbol: string, 
        token_config: "<token_config>",
        public_metadata: "<metadata>",
        private_metadata: "<metadata>",
      }],
      balances: [{
        address: string,
        amount: string,
      }]
    }],
    memo?: string,
    padding?: string,
  }
}
```

Response:
```js
{
  curate_token_id: {
    status: "success"
  }
}
```

### Mint tokens
Minters of a given token_id MUST be able to access this function. Other addresses MUST NOT be able to call this function. (Note that admins and curators cannot mint unless they are also minters. Additionally, minters are set by either the admin or the curator that curated the given token_id). 

A minter MUST be able to mint tokens on existing `token_id`s if the configuration allows it to. If a token_id is an NFT, minters MUST NOT be able to mint additional tokens; NFTs SHALL only be minted at most once. The token configuration SHOULD specify whether minters are allowed to mint additional tokens (for fungible tokens).

`MintTokens` MUST be able to mint multiple tokens across multiple `token_id`s in a single transaction. Therefore, `BatchMintTokens` is not necessary.

Message:
```js
{
  mint_tokens: {
    mint_tokens: [{
      token_id: string,
      balances: [{
        address: string,
        amount: string,
      }]
    }],
    memo?: string,
    padding?: string,
  }
}
```

Response:
```js
{
  mint_tokens: {
    status: "success"
  }
}
```

### Burn tokens
Owners of tokens MUST be allowed to burn their tokens only if the `token_id` configuration allows it to. The base specification does not allow any address to burn tokens they do not own, but this feature is OPTIONAL.

`BurnTokens` MUST be able to burn multiple tokens across multiple `token_id`s in a single transaction. Therefore, `BatchBurnTokens` is not necessary.

Message:
```js
{   
  burn_tokens: {
    burn_tokens: [{
      token_id: string,
      balances: [{
        address: string,
        amount: string,
      }]
    }],
    memo?: string,
    padding?: string,
  }
}
```

Response:
```js
{
  burn_tokens: {
    status: "success"
  }
}
```

### Change metadata

Minters (for fungible tokens and NFTs) or owners (for NFTs only) MUST be able to change the token_id's metadata if the configuration allows them to. `null` values can be used for either `public_metadata` or `private_metadata` fields in order to leave the existing metadata unchanged.  

```js
{
  change_metadata: {
      token_id: string,
      public_metadata?: "<metadata>",
      private_metadata?: "<metadata>",
  }
}
```

Response:
```js
{
  change_metadata: {
    status: "success"
  }
}
```

### Transfer
Transfers a specified number of tokens of a single `token_id` from one address to another. If the transaction caller is not the current owner of the token, a successful transaction MUST require that the caller has the required transfer allowances.  

The SNIP1155 `Transfer` interface more closely reflects SNIP721 than SNIP20. SNIP20's `Transfer` and `TransferFrom` functions can both be performed using SNIP1155's `Transfer` function. 

```js
{
  transfer: {
    token_id: string,
    from: string,
    recipient: string,
    amount: string,
    memo?: string,
    padding?: string,
  },
}
```

Response:
```js
{
  transfer: {
    status: "success"
  }
}
```

### Send
Similar to `Transfer`. If the recipient has registered itself with a `RegisterReceive` message, this function MUST also send an inter-contract message ("callback") to the recipient. It is also RECOMMENDED that `Send` includes an optional code hash input, so the recipient contract does not need to have to first `RegisterReceive`.

The SNIP1155 `Send` interface more closely reflects SNIP721 than SNIP20. SNIP20's `Send` and `SendFrom` functions can both be performed using SNIP1155's `Send` function. 

```js
{
  send: {
    token_id: string,
    from: string,
    recipient: string,
    recipient_code_hash?: string,
    amount: string,
    msg?: "<binary>",
    memo?: string,
    padding?: string,
  },
}
```

Response:
```js
{
  send: {
    status: "success"
  }
}
```

### Batch transfer and batch send
These functions perform multiple `Transfer`, or `Send` actions in a single transaction. Multiple `token_id`s and recipients MUST be allowed in a batch, including a mix of NFTs and fungible tokens of the same SNIP1155 contract.

`BatchSend` MUST allow different callback messages to be sent for each `Send` action.

```js
{
  batch_transfer: {
    actions: [{
      token_id: string,
      from: string,
      recipient: string,
      amount: string,
      memo?: string,
    }],
    padding?: string,
  },
}
{
  batch_send: {
    actions: [{
      token_id: string,
      from: string,
      recipient: string,
      recipient_code_hash?: string,
      amount: string,
      msg?: "<binary>",
      memo?: string,
    }],
    padding?: string,
  },
}
```

Response:
```js
{
  batch_transfer: {
    status: "success"
  }
}
{
  batch_send: {
    status: "success"
  }
}
```

### Give permission
`GivePermission` is used by an address to grant other addresses permissions to transfer or view private information of its tokens. This function MUST allow the transaction caller to set the `token_id`s that fall in scope of a given approval (unlike in CW1155, where approvals are global). Permissions MUST include the ability for a token owner to allow another address to:
* view token owner's balance (of specified token_ids. The base specification does not allow giving permission to view all token_id balances)
* view private metadata 
* transfer tokens up to a specified allowance

`GivePermission` can be used to set a specific transfer allowance limit. It is OPTIONAL to additionally include `IncreaseAllowance` and `DecreaseAllowance` messages, as these are familiar SNIP20 interfaces.

```js
{
  give_permission: {
    allowed_address: string,
    token_id: string,
    view_balance?: boolean,
    view_balance_expiry?: "<expiration>",
    view_private_metadata?: boolean,
    view_private_metadata_expiry?: "<expiration>",
    transfer?: string,
    transfer_expiry?: "<expiration>",
    padding?: string,
  },
}
```

`expiration` can be one of the three variants below
```js
{
  at_height: number,
}
{
  at_time: number,
}
{
  never,
}
```

Response:
```js
{
  give_permission: {
    status: "success"
  }
}
```

### Revoke permission
An operator with existing permissions (not to be confused with Query Permits) can use this to revoke (or more accurately, renounce) the permissions it has received. A token owner can also call this function to revoke permissions, although it is recommended that `GivePermission` is used for this purpose instead.  

```js
{
  revoke_permission: {
    token_id: string,
    owner: string,
    allowed_address: string,
    padding?: string,
  },
}
```

Response:
```js
{
  revoke_permission: {
    status: "success"
  }
}
```


### Create viewing key and set viewing key 
These perform the same functions as specified in the SNIP20 standards.

A user can call this function to create or set a viewing key, which is used to perform authenticated queries.

```js
{
  create_viewing_key: {
    entropy: string,
    padding?: string,
  },
}
{
  set_viewing_key: {
    key: string,
    padding?: string,
  },
}
```

Response:
```js
{
  create_viewing_key: {
    status: "success"
  }
}
{
  set_viewing_key: {
    status: "success"
  }
}
```

### Revoke permit
Similar to SNIP20 and SNIP721; allows an address revoke a query permit (not to be confused with permission) that it may have previously created and shared. Query permits are an alternative way to perform authenticated queries without having to first create a viewing key. The ability to revoke permits gives an additional level of access control. 

```js

{
  revoke_permit: {
    permit_name: string,
    padding?: string,
  },
}
```

```js
{
  revoke_permit: {
    status: "success"
  }
}
```


### Add curators and remove curators
The admin MUST be able to access this function. Other addresses MUST NOT be able to call this function. `AddCurators` add one or more curators to the list of curators, while `RemoveCurators` remove one or more curators from the list of curators. Note that a single SNIP1155 contract instance share a consistent list of curators.

Message:
```js
{
  add_curators: {
        add_curators: string[],
        padding?: string,
    },
}
{
  remove_curators: {
        remove_curators: string[],
        padding?: string,
    },
}
```

Response:
```js
{
  add_curators: {
    status: "success"
  }
}
{
  remove_curators: {
    status: "success"
  }
}
```

### Add minters and remove minters
The admin MUST be able to access this function. Other addresses MUST NOT be able to call this function. `AddMinters` add one or more minters to the list of minters for a given token_id, while `RemoveMinters` remove one or more minters from the list of minters for a given token_id.

In additional specifications, it is OPTIONAL to allow the curator of the specific `token_id` to call `AddMinters` and `RemoveMinters`.  

Message:
```js
{
  add_minters: {
        token_id: string,
        add_minters: string[],
        padding: string,
    },
}
{
  remove_minters: {
      token_id: string,
      remove_minters: string[],
      padding: string,
  },
}
```

Response:
```js
{
  add_minters: {
    status: "success"
  }
}
{
  remove_minters: {
    status: "success"
  }
}
```

### Change admin
The admin MUST be able to access this function. Other addresses MUST NOT be able to call this function. This function allows an admin to change the admin address. When this happens, the message caller SHOULD lose its own admin rights. The base specifications allow only one admin at a time. If multiple admins are implemented (possible in additional specifications), a public query MUST be available for anyone to view all the admin addresses. 

```js
{
  change_admin: {
    new_admin: string,
    padding?: string,
  },
}
```

Response:
```js
{
  change_admin: {
    status: "success"
  }
}
```

### Remove admin
The admin MUST be able to access this function. Other addresses MUST NOT be able to call this function. This function allows an admin to revoke its admin rights, without assigning a new admin. Doing this results in a contract that permanently has no admin (ie: "breaking the admin key").

```js
{
  remove_admin: {
    current_admin: string,
    contract_address: string,
    padding?: string,
  },
}
```

Response:
```js
{
  remove_admin: {
    status: "success"
  }
}
```

## Queries

### Contract info

Any user MUST be able to query the contract information, which MUST provide the following information:
* Current `admin`; if there is no admin, it MUST return `null`
* Current list of `curators`
* A list of all token_ids that have been curated

```js
{
  contract_info: {  }
}
```

```js
{
  contract_info: {
    admin?: string,
    curators: string[],
    all_token_ids: string[],
  }
}
```

### TokenId public information

Any user MUST be able to query the public information of a given token_id. In the base specification, the query response json schema is similar to `token_id_private_info`, except that `private_metadata` MUST be `null`. 

Query message:
```js
{
  token_id_public_info: { 
    token_id: string 
  }
}
```

Query response:
```js
{
  token_id_public_info: {
    token_id_info: {
      token_id: string, 
      name: string, 
      symbol: string, 
      token_config: "<token_config>",
      public_metadata: "<metadata>",
      private_metadata: null,
      curator: string
    },
    total_supply?: string,
    owner?: string
  }
}
```

### Registered code hash

Any user MUST be able to query the code hash of a contract that has registered with the SNIP1155 contract.

Query message:
```js
{
  registered_code_hash: {
    contract: string
  }
}
```

Query response:
```js
/// returns None if contract has not registered with SNIP1155 contract
{
  registered_code_hash: {
    code_hash?: string,
  }
}
```

## Authenticated queries
Authenticated queries can be made using viewing keys or query permits. If viewing key is incorrect, an `viewing_key_error` is returned with a custom message:

```js
{
  viewing_key_error: {
    msg: string,
  }
}
```

If incorrect query permit is used, the contract returns `generic_err` with a custom message in most cases. 


### Balance

A user MUST be able to view the balances of its own token_ids. The user may also grant another user to view its balances. 

Query message:
```js
// with viewing key
{
  balance: {
    owner: string,
    viewer: string,
    key: string,
    token_id: string,
  }
}
// with query permit
{
  with_permit: {
    permit: <"permit">,
    query: {
      balance: { 
        owner: string, 
        token_id: string 
      }
    }
  }
}
```

Query reponse:
```js
{
  balance: {
    amount: string,
  },
}
```

### All balances

An owner MUST be able to query all its token_id balances. Note that in the base specification, balance viewership permission only grants another address to query `balance`, not `AllBalances`. Functionally, this query searches through an address's transaction history (effectively calling [transaction_history](#transaction-history)) to produce a list of token_ids, before searching for balances of each unique token_id. In order to avoid situations where query computation is too large, users have the option to specify the page and page size of the `transaction_history` search; this is unlikely to be necessary for straight queries, but may be useful for contract-to-contract interactions. If these two fields are ignored, the query will return the full list of `(token_id, balance)` for the address, where it has some balance currently or at some point in the past. Returns in ascending alphabetical order of `token_id`.

Query message:
```js
// with viewing key
{
  all_balances: {
    owner: string,
    key: string,
    tx_history_page?: number,
    tx_history_page_size?: number,
  }
}
// with query permit
{
  with_permit: {
    permit: <"permit">,
    query: {
      all_balances: { 
        tx_history_page?: number,
        tx_history_page_size?: number,
      }
    }
  }
}
```

Query response:
```js
{
  all_balances: [
    {
    token_id: string,
    amount: string
    }
  ]
}
```

### Transaction history

A user MUST be able to view its transaction history. Transactions include minting (including minting initial balances from `CurateTokenIds`), burning, and transferring (including transfers from `Send` messages).

Query message:
```js
// with viewing key
{
  transaction_history: {
    address: string,
    key: string,
    page?: number,
    page_size: number,
  }
}
// with query permit
{
  with_permit: {
    permit: <"permit">,
    query: {
      transaction_history: {
        page?: number,
        page_size: number,
      }
    }
  }
}
```

Query response:
```js
{
  transaction_history: {
    txs: [{
      tx_id: number,
      block_height: number,
      block_time: number,
      token_id: string,
      action: "<tx_action>",
      memo?: string,
    }],
    total: number,
  }
}
```

`tx_action` can be one of the following variants
```js
  mint: {
    minter: string,
    recipient: string,
    amount: string,
  },
  burn: {
    burner?: string,
    owner: string,
    amount: string,
  },
  transfer: {
    from: string,
    sender?: string,
    recipient: string,
    amount: string,
  },
```

### Permission

The permission granter and grantee can view an existing permission.

Query message:
```js
// with viewing key
{
  permission: {
    owner: string,
    allowed_address: string,
    key: string,
    token_id: string,
  }
}
// with query permit
{
  with_permit: {
    permit: <"permit">,
    query: {
      permission: {
        owner: string,
        allowed_address: string,
        token_id: string,
      }
    }
  }
}
```

Query response:
```js
// returns null if no permission 
{
  permission?: {
    view_balance_perm: boolean,
    view_balance_exp: "<expiration>",
    view_pr_metadata_perm: boolean,
    view_pr_metadata_exp: "<expiration>",
    trfer_allowance_perm: string, 
    trfer_allowance_exp: "<expiration>",
  }
}
```

### All permissions
An address (granter) can view a list of all permissions that it has granted to other addresses. The base specification does not allow an address (grantee) to view all permissions it has been granted, but this is OPTIONAL in the additional specifications. 

Query message:
```js
// with viewing key
{
  all_permissions: {
    address: string,
    key: string,
    page?: number,
    page_size: number,
  }
}
// with query permit
{
  with_permit: {
    permit: <"permit">,
    query: {
      all_permissions: {
        page?: number,
        page_size: number,
      }
    }
  }
}
```

Query response:
```js
{
  all_permissions: {
    permission_keys: [{
      token_id: string,
      allowed_addr: string,
    }],
    permissions: [{
      view_balance_perm: boolean,
      view_balance_exp: "<expiration>",
      view_pr_metadata_perm: boolean,
      view_pr_metadata_exp: "<expiration>",
      trfer_allowance_perm: string, 
      trfer_allowance_exp: "<expiration>",
    }],
    total: number,
  }
}
```

### TokenId private information

A token_id owner or address that has been granted permission MUST be able to query the private information of a given token_id. In the base specification, the query response json schema is similar to `token_id_public_info`, except that the `private_metadata` field MUST include the private metadata if it exists. 


Query message:
```js
// with viewing key
{
  token_id_private_info: { 
    address: string,
    key: string,
    token_id: string,
  }
}
// with query permit
{
  with_permit: {
    permit: <"permit">,
    query: {
      token_id_private_info: { 
        token_id: string,
      }
    }
  }
}
```

Query response:
```js
{
  token_id_private_info: {
    token_id_info: {
      token_id: string, 
      name: string, 
      symbol: string, 
      token_config: "<token_config>",
      public_metadata: "<metadata>",
      private_metadata: "<metadata>",
      curator: string
    },
    total_supply?: string,
    owner?: string
  }
}
```

## Receiver functions

### Register receive
This message is used to pair a code hash with a contract address. The SNIP1155 contract MUST store the `code_hash` sent in this message, and use it when calling the `Snip1155Receive` function.

```js
{
  register_receive: {
    code_hash: string,
    padding?: string,
  },
}
```

Response:
```js
{
  register_receive: {
    status: "success"
  }
}
```

### Snip1155Receive
When a the `Send` or `BatchSend` function is called, the SNIP1155 sends a callback to a registered address. When doing so, the SNIP1155 contract MUST call the `Snip1155Receive` handle function of the recipient contract. The callback message is in the following format:

```js
{
  snip1155_receive: {
    sender: "<HumanAddr that called the transaction>",
    token_id: "<String representing unique token_id being sent>",
    from: "<HumanAddr of the current owner of the tokens>",
    amount: "<Amount of tokens sent in Uint128>",
    memo?: "<optional String>",
    msg?: "<optional message in Binary>"
  }
}
```

## Miscellaneous

### Padding <!-- omit in toc --> 
Users may want to enforce constant length messages to avoid leaking data. To support this functionality, SNIP1155 tokens MUST support the option to include a padding field in every message. This optional padding field may be sent with ANY of the messages in this spec. Contracts and Clients MUST ignore this field if sent, either in the request or response fields.

### Requests <!-- omit in toc --> 
Requests SHOULD be sent as base64 encoded JSON. Future versions of Secret Network may add support for other formats as well, but at this time we recommend usage of JSON only. For this reason the parameter descriptions specify the JSON type which must be used. In addition, request parameters will include in parentheses a CosmWasm (or other) underlying type that this value must conform to. E.g. a recipient address is sent as a string, but must also be parsed to a bech32 address.

### Queries <!-- omit in toc --> 
Queries are off-chain requests, that are not cryptographically validated. This means that contracts that wish to validate the caller of a query MUST implement some sort of authentication. SNIP1155 uses an "API key" scheme, which validates a (viewing key, account) pair, as well as query permits (which utilizes digital signatures).

Authentication MUST happen on each query that reveals private account-specific information. Authentication MUST be a resource intensive operation, that takes a significant amount of time to compute. This is because such queries are open to offline brute-force attacks, which can be parallelized to scale linearly with the resources of a motivated attacker. If viewing keys are used, the authentication MUST perform the same computation even if the user does not have a viewing key set; and the authentication response MUST be indistinguishable for both the case of a wrong viewing key and the case of a non-existent viewing key.

### Responses <!-- omit in toc --> 
Unless specified otherwise, all message & query responses will be JSON encoded in the data field of the Cosmos response, rather than in the logs. This is meant to reduce the potential for data-leakage through side-channel attacks. In addition, since all keys will be encrypted, it is not possible to use the log events for event triggering.

### Success status <!-- omit in toc --> 
Some of the messages detailed in this document contain a "status" field. This field MUST hold one of two values: "success" or "failure".

While errors during execution of contract functions should usually result in a proper and detailed error response, the "failure" status is reserved for cases where a contract might choose to obfuscate the exact cause of failure, or otherwise indicate that while nothing failed to happen, the operation itself could not be completed for some valid reason.

### Balances and amounts <!-- omit in toc --> 
Note that all amounts are represented as numerical strings (the Uint128 type). Handling decimals is left to the UI.



# Additional specifications

Additional specifications include:
* Royalty for NFTs
* Ability for owners to give other addresses permission to burn their tokens 
* Ability to view nft ownership history, including configuration on whether this should be public. In the base reference implementation, only the current owner may be viewable, but the history of ownership is actually being saved, although not accessible through queries.
* Ability for an address (grantee) to view list of all permissions that it has been granted by others (currently only granter can view comprehensive list of its permissions)
* Sealed metadata and Reveal functionality that mirrors SNIP721
* Ability for admin to restrict certain types of transactions (as seen in SNIP20 and SNIP721). A design decision was made on SNIP1155 NOT to include this functionality in the base specifications, in order to encourage more permissionless contract designs.
* Ability for an owner to give another address batch permission that covers all its token_ids. 
* Expand ability for query permits to selectively allow access to specific query functions (in the base specifications, users can grant selective viewership permissions for balances or private metadata)


# Design decisions

## Reference implementation goals
The SNIP1155 standards and [reference implementation](https://github.com/DDT5/snip1155-reference-impl/) aims to provide developers with an additional option to use as a base contract for creating Secret Network tokens. The aim is to complement the existing SNIP20 and SNIP721 standards by offering a lean contract whose core architecture focuses on the ability to create and manage multiple token types.

Non-goals include:
* providing a feature-rich base contract that encompasses both SNIP20 and SNIP721 functionality
* superseding previous token standards
* backward compatibility with previous token standards, although [familiar](#familiar-interface) interfaces are used where possible

## Starting from a blank slate
The current Secret Network token standard reference implementations at the time of writing are feature-rich and implement many useful features beyond their base standards. This makes it possible to use one of these contracts as a starting point and add functionality in order to create the SNIP1155 reference implementation.

However, the decision was to build the SNIP1155 reference implementation mostly from a blank slate. This creates a leaner contract with several advantages from a systems design point of view:
* Mitigates code bloat. The SNIP1155 standard implementation generally aims to avoid implementing non-core features, and starting from a blank slate avoids adopting features from another standard that are not critical for SNIP1155.
* Reduces surface area of attack and potential for bugs.
* Offers developers an additional base contract option that is meaningfully different to existing templates, so developers can choose the architecture that most closely fits their requirements.  
* Allows SNIP1155 to follow an independent update cycle and its own development path, making it more responsive to feature requirements or changes to the network infrastructure that matter most to SNIP1155 use cases.

The disadvantages of this design decision are:
* No (full) backward compatibility with SNIP20 or SNIP721
* Required more developer hours to create

## Permissionless design
The standard implementations of both SNIP20 and SNIP721 (at the time of writing) are designed to have an admin with special rights. This can be a useful feature depending on the required use case, but has the downside of not being permissionless.    

With this in mind, the SNIP1155 base standard implementation natively provides the ability to break admin keys at any point, as well as for contract instantiators to create no-admin contract instances. 

## Familiar interface
The interface reflects a few of the message schemas in [SNIP20](https://github.com/scrtlabs/snip20-reference-impl) and [SNIP721](https://github.com/baedrik/snip721-reference-impl), in order to have a familiar interface across Secret Network. However, SNIP1155 does not aim to have backward compatibility with SNIP20 and SNIP721.

## Modular additional features
The reference implementation aims to maintain a base implementation that is lean (with no additional features). The aspiration is to eventually offer template code for some additional feature(s) in a modular fashion. Keeping the base implementation and additional features separate avoids the situation where developers are forced to adopt all additional features even if their use case does not require them.
