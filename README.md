# Shade Protocol Core Contracts
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
