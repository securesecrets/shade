# Shade Protocol 

## Overview
Shade Protocol is a decentralized, cross-chain, asset management protocol. It is designed to be a modular, composable, and extensible protocol that is the host to variety of asset management tools. The protocol is built on the [Secret Network](https://scrt.network/) and is designed to be interoperable with other blockchains within the [Cosmos Network](https://cosmos.network/) using [Inter Blockchain Protocol (IBC)](https://ibc.cosmos.network/)

## Protocol Modules

1. Shade Swap - A decentralized, private, cross-chain, AMM-based DEX, built on the Secret Network
    1. Stable Swap - A decentralized, private, cross-chain, stablecoin swap protocol
    2. Derivative Swap - A decentralized, private, cross-chain, derivative swap protocol
2. Shade Lending - A decentralized, private, cross-chain, lending protocol
    1. SILK - A decentralized, private, cross-chain, stablecoin.
3. Shade Staking - A decentralized, private, cross-chain, staking protocol
4. Shade Treasury - A decentralized, private, cross-chain, asset management protocol
5. Shade Mint - A decentralized, private, cross-chain, asset minting and burning protocol
6. Shade Oracle - A decentralized, private, cross-chain, asset price oracle



## Core Contracts
| Contract                    | Reference                         | Description                           |
| --------------------------- | --------------------------------- | ------------------------------------- |
| [`governance`](./contracts/governance)  | [doc](./contracts/governance/README.md) | Protocol's governance module |
| [`shade_staking`](./contracts/staking)  | [doc](./contracts/staking/README.md) | Snip20 staker |
| [`scrt_staking`](./contracts/scrt_staking)  | [doc](./contracts/scrt_staking/README.md) | SCRT staker |
| [`treasury`](./contracts/treasury)  | [doc](./contracts/treasury/README.md) | Protocol's asset manager |
| [`mint`](./contracts/mint)  | [doc](./contracts/mint/README.md) | Asset burner and minter |
| [`oracle`](./contracts/oracle)  | [doc](./contracts/oracle/README.md) | Asset price querier |
| [`airdrop`](./contracts/airdrop)  | [doc](./contracts/airdrop/README.md) | Task based, multichain snip20 airdropper  |

## Development Environment

### Environment Setup

1. Make sure [Docker](https://www.docker.com/) is installed

2. Pull the SN-testnet image
```shell
make server-download
```

3. Open a terminal inside this repo and run:
```shell
make server-start
```

4. Inside another terminal run:
```shell
make server-connect
```

#### Testing the environment
Inside the container, go to /root/code and compile all the smart contracts:
```
make
```
Then test run all the Protocol unit-tests and integration tests using the [tester](packages/network_integration):
```shell
make integration-tests
```

### Unit Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
```

## Security
For further details about the security of the protocol, please refer to the [Security document](./SECURITY.md).

## License
The Shade Protocol Core Contracts are licensed under the [Apache License 2.0](./LICENSE).