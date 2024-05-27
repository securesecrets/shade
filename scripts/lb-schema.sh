#!/bin/bash

export RUSTFLAGS="--cfg cargo_supress_warning"
cargo run -p lb-factory --bin schema
cargo run -p lb-pair --bin schema
cargo run -p lb-router --bin schema
cargo run -p lb-staking --bin schema
cargo run -p lb-token --bin schema
