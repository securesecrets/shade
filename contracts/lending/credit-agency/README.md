# Credit Agency

The credit agency manages all the Lending Pools and authorizes loans, transfer of collateral, and, if necessary, liquidations. It depends on Price Oracles to provide accurate price feeds. It also requires a Liquidator to sell off assets. This could be swapping on an AMM or directly selling to anyone claiming the liquidation event. Those are detailed in other sections. Here, we describe how credit is calculated.

## Collateral Ratio

Different base assets have different quality and volatility. These characteristics determine how much collateral is needed relative to a loan. Thus, when adding a Lending Pool to the Credit Agency, we also need to set a collateral ratio. In a future version, this can be changed by governance, for now it is a constant.
If I have `S` value in cTokens, I can borrow `S * collateral_ratio`.

## Common Currency

In order to compare all of these Lending Pools, we need to price all base assets to some common currency. The price oracles do this, but require a highly liquid market to provide accurate pricing (and time average it).
This is actually an old and soon no longer up to date concept - deployed on Shade, the common denomination will be in USD stable coin.

## Entering Markets

While the above logic is correct, it is also quite expensive to execute if we have a few dozen different Lending Pools, while a given account only uses 2 or 3. In order to speed this up, a user can "enter" a market by declaring their intent to use it. This market is then used to calculate their Credit Line and Total Debt. Note that you can only borrow from markets you have added.

## Liquidation

Liquidation is the act of selling collateral to cover the undercollateralized debt positions. Undercollateralized means the collateral ratio is not maintained. There may still be 150% more value in the collateral than in the borrowed assets, but this margin is important to protect the lenders when the market changes quickly.
We can provide multiple liquidation strategies in the future, but the initial one is as follows:
We provide a liquidate method that can be called by anyone on any account that has negative available credit. They will pay a base asset to repay debt for the user, and in return receive the collateral from the user. In order to incentivise bots to monitor the situation and liquidate, they get the liquidated assets at an 8% discount (fixed number, configured in init).
If the borrower has multiple cTokens as collateral, the one with the lowest collateral_ratio must be returned first, as this is the quickest way to get to a healthy ratio. You can only pay back one asset at a time, and get the equivalent amount + 8% in cTokens belonging to the user, determined by the above ratio:

## Contract workflows

This section provides detailed information about the contract's workflows and interactions for each market entry point, aiming to enhance understanding of the protocol's behavior.

### Instantiate

TODO

### Execute

![CreateMarket](assets/create-market.png "create market")

![ExitMarket](assets/exit-market.png "exit market")

![Liquidate](assets/liquidate.png "liquidate")

![RepayWithCollateral](assets/repay-with-collateral.png "repay with collateral")

## ExecuteMsg Functions
- **CreateMarket**
  - To instantiate new lending market contracts.
  - Takes `MarketConfig` as an argument to set up a new market.

- **EnterMarket**
  - Ensures that an account has entered a specific market.
  - Called by a market contract. Registers an account's participation in a market.

- **ExitMarket**
  - Allows an account to exit a market under certain conditions.
  - Exits are allowed if the account has no debt and no C Tokens in the market, or if the collateral provided by owned C Tokens does not affect the account's liquidity.

- **Receive**
  - Handles the logic involving receiving Snip20 tokens.
  - Processes incoming Snip20 tokens as per contract's logic.

- **AdjustMarketId**
  - Adjusts the identification number of a market.
  - Can only be called by the Governance Contract to update market identification.

- **AdjustTokenId**
  - Changes the identification number of a token.
  - Restricted to the Governance Contract, used for updating token identification.

- **AdjustCommonToken**
  - Sets a common token parameter in the configuration.
  - Updates the common token setting and notifies all affiliated markets. Restricted to the Governance Contract.

## ReceiveMsg Functions
- **Liquidate**
  - Facilitates the liquidation of an account using collateral's denomination.
  - The Snip20 tokens sent with this message define the debt market. This function is used to initiate liquidation on an account based on its collateral.

## QueryMsg Functions
- **Configuration**
  - Provides the current configuration settings of the Credit Agency contract.

- **Market**
  - Queries a market address by its market token.
  - Identifies and returns the address of a market based on the provided market token.

- **ListMarkets**
  - Lists all base assets and the markets handling them.
  - Provides a paginated list of markets along with their associated base assets.

- **TotalCreditLine**
  - Queries the total credit line available to a specific account across all markets.
  - Aggregates credit lines from various markets for a given account, returning the sum.

- **ListEnteredMarkets**
  - Lists all markets that a particular account has entered.
  - Provides a list of markets in which the specified account is involved, useful for tracking participation and verifying market engagement.

- **IsOnMarket**
  - Checks if an account is a member of a particular market.
  - Determines and confirms an account's membership in a specified market.

- **Liquidation**
  - Checks if a given account is liquidatable and provides necessary information for liquidation.
  - Assesses an account's position and returns details relevant to potential liquidation processes.


## Functionality Overview
The Credit Agency smart contract is a central piece in the lending market ecosystem. It manages the creation and adjustment of markets, ensuring proper interaction and compliance among them. Key functions include market creation and management, Snip20 token handling, and adjustments of market and token parameters, typically controlled by the Governance Contract. The `Configuration` query provides insight into the contract's settings, crucial for understanding the operational parameters of the Credit Agency.
