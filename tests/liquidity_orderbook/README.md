# Contract Tests

!! **Contracts must be compiled and placed in the `wasm` directory.**

Run tests with

```sh
npx ts-node integration.ts
```

## LBFactory

### Minimum Viable Tests

- [x] Set LBPair implementation
- [x] Add presets and quote assets
- [x] Be able to create an LBPair (message will come from Router)
- [x] Get pool information for a specific pair

### Basic Message Tests

- [x] Instantiate
- [ ] Execute
  - [x] SetLBPairImplementationMsg\*
  - [x] CreateLBPairMsg\*
  - [ ] SetLBPairIgnoredMsg
  - [x] SetPresetMsg\*
  - [ ] SetPresetOpenStateMsg
  - [ ] RemovePresetMsg
  - [ ] SetFeeParametersOnPairMsg
  - [ ] SetFeeRecipientMsg
  - [ ] SetFlashLoanFeeMsg
  - [x] AddQuoteAssetMsg\*
  - [ ] RemoveQuoteAssetMsg
  - [ ] ForceDecayMsg
- [ ] Query
  - [ ] GetMinBinStepQuery
  - [ ] GetFeeRecipientQuery
  - [ ] GetMaxFlashLoanFeeQuery
  - [ ] GetFlashLoanFeeQuery
  - [ ] GetLBPairImplementationQuery
  - [ ] GetNumberOfLBPairsQuery
  - [ ] GetLBPairAtIndexQuery
  - [ ] GetNumberOfQuoteAssetsQuery
  - [ ] GetQuoteAssetAtIndexQuery
  - [ ] IsQuoteAssetQuery
  - [x] GetLBPairInformationQuery\*
  - [ ] GetPresetQuery
  - [ ] GetAllBinStepsQuery
  - [ ] GetOpenBinStepsQuery
  - [ ] GetAllLBPairsQuery

### Edge cases

TBD

---

## LBPair

### Minimum Viable Tests

- [x] Be able to mint LP tokens
- [ ] Be able to burn LP tokens
- [ ] Be able to swap
- [x] Be able to estimate swaps

### Basic Message Tests

- [x] Instantiate
- [ ] Execute
  - [ ] Swap\*
  - [ ] FlashLoan
  - [x] Mint\*
  - [ ] Burn\*
  - [ ] CollectProtocolFees
  - [ ] IncreaseOracleLength
  - [ ] SetStaticFeeParameters
  - [ ] ForceDecay
- [ ] Query
  - [ ] GetFactory
  - [ ] GetTokenX
  - [ ] GetTokenY
  - [ ] GetBinStep
  - [x] GetReserves\*
  - [x] GetActiveId\*
  - [ ] GetBin
  - [ ] GetNextNonEmptyBin
  - [ ] GetProtocolFees
  - [x] GetStaticFeeParameters
  - [x] GetVariableFeeParameters
  - [ ] GetOracleParameters
  - [ ] GetOracleSampleAt
  - [x] GetPriceFromId\*
  - [ ] GetIdFromPrice\*
  - [x] GetSwapIn\*
  - [x] GetSwapOut\*

### Edge cases

TBD

---

## LBQuoter

### Basic Message Tests

- [ ] Instantiate
- [ ] Execute
  - [ ] TBD
- [ ] Query
  - [ ] TBD

### Edge cases

TBD

---

## LBRouter

### Minimum Viable Tests

- [x] Create an LBPair
- [x] Add liquidity
- [ ] Remove liquidity
- [ ] Swap exact tokens for tokens
- [ ] Get price of a given bin id

### Basic Message Tests

- [ ] Instantiate
- [ ] Execute
  - [x] CreateLBPair\*
  - [x] AddLiquidity\*
  - [ ] AddLiquidityNATIVE\*
  - [ ] RemoveLiquidity\*
  - [ ] RemoveLiquidityNATIVE\*
  - [ ] SwapExactTokensForTokens\*
  - [ ] SwapExactTokensForNATIVE
  - [ ] SwapExactNATIVEForTokens
  - [ ] SwapTokensForExactTokens
  - [ ] SwapTokensForExactNATIVE
  - [ ] SwapNATIVEForExactTokens
  - [ ] SwapExactTokensForTokensSupportingFeeOnTransferTokens
  - [ ] SwapExactTokensForNATIVESupportingFeeOnTransferTokens
  - [ ] SwapExactNATIVEForTokensSupportingFeeOnTransferTokens
  - [ ] Sweep
  - [ ] SweepLBToken
- [ ] Query
  - [ ] GetFactory
  - [ ] GetWNATIVE
  - [ ] GetIdFromPrice
  - [ ] GetPriceFromId\*
  - [ ] GetSwapIn
  - [ ] GetSwapOut

### Edge cases

TBD

---

## LBToken

### Minimum Viable Tests

- [ ] Mint tokens with different IDs
- [ ] Burn tokens based on ID
- [ ] Transfer tokens based on ID

### Basic Message Tests

- [x] Instantiate
- [ ] Execute
  - [ ] ApproveForAll
  - [ ] BatchTransferFrom
  - [ ] Mint
  - [ ] Burn
- [ ] Query
  - [x] Name
  - [x] Symbol
  - [x] Decimals
  - [ ] TotalSupply
  - [ ] BalanceOf
  - [ ] BalanceOfBatch
  - [ ] IsApprovedForAll

### Edge cases

#### ApproveForAll

1. tbd

#### BatchTransferFrom

1. tbd

#### Mint

1. tbd

#### Burn

1. tbd
