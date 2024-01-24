# Token contract

This contract is used for keeping track of CTokens.
A market contract instantiates this for tracking the collateral.

## Overview

The `lend-token` contract has a few main functionalities:

* Modified cw20-token
  * Some amount of tokens may be locked
  * Transfer and burn limits are controlled by external logic (market contract)
  * Maintains a global "multiplier" of how many "base" tokens each token represents
* ERC2222-inspired distribution contract
  * Efficiently distributes rewards among all token holders
  * Each holder can withdraw his/her share manually

## Snip20 Token

The initialization can config queries are very similar to cw20-base. The currently miss
some fields like `MarketingInfo`, which could be added.

It supports `transfer`, `transfer_from`, `send`.
`mint` and `burn` are reserved for the controller.
Allowances are not implemented as deemed not very important.

# Token Smart Contract Documentation - ExecuteMsg and QueryMsg

### ExecuteMsg

- **Transfer**
  - Moves tokens to another account without triggering actions.
  - Requires `ControllerQuery::CanTransfer` call for transfer validation.

- **TransferFrom**
  - Orders transfer of tokens from sender to recipient.
  - Restricted to the controller.
  - Requires `ControllerQuery::CanTransfer` validation.

- **TransferBaseFrom**
  - Similar to `TransferFrom`, but uses base token amount.
  - Reserved for controller use.

- **Send**
  - Transfers tokens to a contract, triggering an action on the recipient contract.
  - Requires `ControllerQuery::CanTransfer` validation.

- **Mint**
  - Creates new tokens for a recipient.
  - Controller-only function.

- **MintBase**
  - Similar to `Mint`, but specifies amount in base tokens.
  - Reserved for controller.

- **BurnFrom**
  - Destroys tokens from the specified owner's balance.
  - Controller-only function.

- **BurnBaseFrom**
  - Similar to `BurnFrom`, but in base token amounts.
  - Reserved for controller.

- **Rebase**
  - Adjusts the global multiplier.
  - Controller-only function.

- **Distribute**
  - Distributes tokens using the cw2222 mechanism.
  - Includes tokens sent previously but not yet distributed.

- **WithdrawFunds**
  - Withdraws previously distributed tokens.

### QueryMsg

- **Balance**
  - Retrieves the current balance of an address.
  - Requires authentication.

- **BaseBalance**
  - Retrieves balance in base tokens.
  - Requires authentication.

- **TokenInfo**
  - Provides metadata on the contract (name, supply, etc.).

- **Multiplier**
  - Returns the global multiplier factor.

- **DistributedFunds**
  - Shows funds distributed by the contract.

- **UndistributedFunds**
  - Displays funds sent but not yet distributed.

- **WithdrawableFunds**
  - Queries funds distributed but not withdrawn by the owner.
  - Requires authentication.

### Base functions

The "_base" functions in the smart contract (TransferBaseFrom, MintBase, and BurnBaseFrom) are designed to operate with amounts specified in the base token amount rather than the amount of the lending token itself. This distinction is crucial in a lending/borrowing protocol, where there can be different denominations or representations of value.

- TransferBaseFrom
 - This function facilitates the transfer of tokens based on their base token amount, not the lending token's amount. This is particularly useful when dealing with fractionalized tokens or when operating in a context where the base token (underlying asset) needs to be referenced directly, bypassing the lending token's valuation.

- MintBase
 - The MintBase function creates new tokens based on the base token amount. It's used for situations where the issuance of new lending tokens needs to be aligned with the base token's quantity, ensuring consistency between the underlying asset and the lending token's supply.

- BurnBaseFrom
 - Similar to MintBase, BurnBaseFrom destroys lending tokens based on the base token amount. This function is essential for managing the token supply in relation to the underlying asset, allowing for the adjustment of the lending token's supply to accurately reflect changes in the base asset.
The purpose of these "_base" functions is to align the lending token's operations with the base token's metrics. This alignment is crucial for maintaining the integrity and balance of the lending/borrowing protocol, ensuring that the lending tokens accurately represent the underlying assets' value and quantity.
