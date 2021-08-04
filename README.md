# Shade Protocol Core Contracts
| Contract                    | Reference                         | Description                           |
| --------------------------- | --------------------------------- | ------------------------------------- |
| [`mint`](./contracts/mint)  | [doc](./contracts/mint/README.md) | Handles asset burning and silk minting|

## Development

## Development Environment
Instlal docker for local envirnment

Source from [testner](https://build.scrt.network/dev/quickstart.html#setup-the-local-developer-testnet)

```
docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v $(pwd):/root/code --name secretdev enigmampc/secret-network-sw-dev

docker exec -it secretdev /bin/bash

```
#### Testing the environment
Inside the container:
```
run python3 contract_tester.py
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

### Unit / Integration Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
cargo integration-test
```

### Compiling

Run this script to run all of the contract's unit / integration tests and then prepare the contracts for production in /contracts/compiled:

```sh
bash ./compile-contracts.sh
```

### Testing

You can optionally run extended tests inside a private testnet using the [contract tester](contracts/compiled/tester.py)