#!/bin/bash

root_dir=$(git rev-parse --show-toplevel)
contracts_dir="${root_dir}/contracts"
compiled_dir="${contracts_dir}/compiled"

compile_contract() {
  (cd ${contracts_dir}; cargo build --release --target wasm32-unknown-unknown --locked)
  wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$1.wasm -o ./$1.wasm
  cat ./$1.wasm | gzip -n -9 > ${compiled_dir}/$1.wasm.gz
  rm -f ./$1.wasm
}

compile_contract "mint"