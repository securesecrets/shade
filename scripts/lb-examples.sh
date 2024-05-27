#!/bin/bash

export RUSTFLAGS="--cfg cargo_supress_warning"
cargo run -p lb-factory --bin secretcli
cargo run -p lb-pair --bin secretcli
cargo run -p lb-router --bin secretcli
cargo run -p lb-staking --bin secretcli
