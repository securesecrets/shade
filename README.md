# Shade Protocol Core Contracts
| Contract                                            | Reference                                              | Description                                                                                                                        |
| --------------------------------------------------- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| [Placeholder]()  | [Placeholder]() | [Placeholder]                                              |
| [Placeholder]() | [Placeholder]() | [Placeholder]                                                                                             |
| [Placeholder]()     | [Placeholder]()  | [Placeholder]                                                        |
| [Placeholder]()            | [Placeholder]()      | [Placeholder] |
| [Placeholder]()        | [Placeholder]()      | [Placeholder]                                                                                  |
|[Placeholder]()        | [Placeholder]()   | [Placeholder]                                                                 |
| [Placeholder]()     | [Placeholder]()   |[Placeholder]                                                                           |                                                                                    |

## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
cargo integration-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.2
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.
