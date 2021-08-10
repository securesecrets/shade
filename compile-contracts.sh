#!/bin/bash

root_dir=$(git rev-parse --show-toplevel)
contracts_dir="${root_dir}/contracts"
compiled_dir="${contracts_dir}/compiled"
checksum_dir="${compiled_dir}/checksum"

compile_contract() {
  # Run tests
  (cd ${contracts_dir}/$1; cargo unit-test)
  #(cd ${contracts_dir}/$1; cargo integration-test)
  (cd ${compiled_dir}; rm $1.wasm.gz)
  (cd ${contracts_dir}; cargo build --release --target wasm32-unknown-unknown --locked)
  wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$1.wasm -o ./$1.wasm
  echo $(md5sum $1.wasm | cut -f 1 -d " ") >> ${checksum_dir}/$1.txt
  cat ./$1.wasm | gzip -n -9 > ${compiled_dir}/$1.wasm.gz
  rm -f ./$1.wasm
}

# There should be a cleaner way to do this
rm -r ${checksum_dir}
mkdir ${checksum_dir}
compile_contract "mint"
compile_contract "oracle"
compile_contract "mock_band"
