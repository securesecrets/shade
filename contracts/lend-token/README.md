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

## CW20 Token

The initialization can config queries are very similar to cw20-base. The currently miss
some fields like `MarketingInfo`, which could be added.

It supports `transfer`, `transfer_from`, `send`.
`mint` and `burn` are reserved for the controller.
Allowances are not implemented as deemed not very important.
