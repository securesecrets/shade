# Shade Protocol Core Contracts
| Contract                    | Reference                         | Description                           |
| --------------------------- | --------------------------------- | ------------------------------------- |
| [`admin`](./contracts/admin)  | [doc](./contracts/admin/README.md) | Contract admin manager |
| [`governance`](./contracts/governance)  | N/A | Protocol's governance module |
| [`shade_staking`](./contracts/basic_staking)  | [doc](./contracts/basic_staking/README.md) | Snip20 staker |
| [`query_auth`](./contracts/query_auth)  | N/A | authentication manager for validation for permits and viewing keys |
| [`dao_contracts`](./contracts/dao)  |  [doc](./contracts/query_auth/README.md) | collection of dao contracts |
| [`bonds`](./archived-contracts/bonds)  | [doc](./archived-contracts/bonds/README.md)| snip20 bonds |
| [`peg_stability`](./contracts/peg_stability)  | N/A | peg stability |
| [`snip20_migration`](./contracts/snip20_migration)  | N/A |  migrate snip20 tokens into a newer version |
| [`sky_arbitrage`](./contracts/sky)  | N/A |  protocol arbitrage contract |
| [`airdrop`](./contracts/airdrop)  | [doc](./contracts/airdrop/README.md) | Task based, multichain snip20 airdropper  |
| [`mock_contracts`](./contracts/mock)  | N/A | testing contracts that mock mainnet contracts |
| [`snip20_derivative`](./contracts/snip20_derivative)  | [doc](./contracts/snip20_derivative/README.md) | snip20 staking derivative token  |

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
