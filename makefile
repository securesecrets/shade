contracts_dir=contracts
compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum

# Compresses the wasm file, args: compressed_file_name, built_file_name
define compress_wasm =
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$(2).wasm -o ./$(1).wasm
echo $(md5sum $(1).wasm | cut -f 1 -d " ") >> ${checksum_dir}/$(1).txt
cat ./$(1).wasm | gzip -n -9 > ${compiled_dir}/$(1).wasm.gz
rm ./$(1).wasm
endef

CONTRACTS = airdrop governance staking mint treasury micro_mint oracle mock_band initializer scrt_staking

release: build_release compress

debug: build_debug compress

build_release:
	(cd ${contracts_dir}; RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked)

build_debug:
	(cd ${contracts_dir}; RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked --features="debug-print")

compress: setup $(CONTRACTS); $(call compress_wasm,snip20,snip20_reference_impl)

$(CONTRACTS):
	$(call compress_wasm,$@,$@)

setup: $(compiled_dir) $(checksum_dir)

$(compiled_dir) $(checksum_dir):
	mkdir $@

check:
	cargo check

clippy:
	cargo clippy

clean:
	rm -r $(compiled_dir)

format:
	cargo fmt

# Downloads the docker server
server-download:
	docker pull securesecrets/sn-testnet:v0.2

# Starts the docker server / private testnet
server-start:
	docker run -it --rm \
	 -p 26657:26657 -p 26656:26656 -p 1337:1337 \
	 -v $$(pwd):/root/code --name shade-testnet securesecrets/sn-testnet:v0.2

# Connects to the docker server
server-connect:
	docker exec -it shade-testnet /bin/bash

# Runs integration tests
integration-tests:
	cargo test -- --nocapture --test-threads=1