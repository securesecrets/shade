# Shade Protocol Core Contracts
| Contract                    | Reference                         | Description                           |
| --------------------------- | --------------------------------- | ------------------------------------- |
| [`governance`](./contracts/governance)  | [doc](./contracts/governance/README.md) | Protocol's governance module |
| [`shade_staking`](./contracts/staking)  | [doc](./contracts/staking/README.md) | Snip20 staker |
| [`scrt_staking`](./contracts/scrt_staking)  | [doc](./contracts/scrt_staking/README.md) | SCRT staker |
| [`treasury`](./contracts/treasury)  | [doc](./contracts/treasury/README.md) | Protocol's asset manager |
| [`mint`](./contracts/micro_mint)  | [doc](./contracts/micro_mint/README.md) | Asset burner and minter |
| [`oracle`](./contracts/oracle)  | [doc](./contracts/oracle/README.md) | Asset price querier |
| [`airdrop`](./contracts/airdrop)  | [doc](./contracts/airdrop/README.md) | Task based, multichain snip20 airdropper  |

## Development

## Development Environment
Install docker for local environment

Source from [testnet](https://build.scrt.network/dev/quickstart.html#setup-the-local-developer-testnet)

```
docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v $(pwd):/root/code --name secretdev enigmampc/secret-network-sw-dev

docker exec -it secretdev /bin/bash
```
#### Testing the environment
First go inside the repo and build all of the contracts:
```
make
```
Then inside the container run:
```
cargo test -- --nocapture --test-threads=1
```

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker
- binaryen

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

4. To compile the contracts install binaryen
```sh
apt install binaryen
```

### Unit Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
```

### Compiling

Run this script to run all of the contract's unit / integration tests and then prepare the contracts for production in /compiled:

```sh
make
```

### Testing

You can optionally run extended tests using the [tester](packages/network_integration)

For the private testnet you can run ```cargo test -- --nocapture --test-threads=1```
