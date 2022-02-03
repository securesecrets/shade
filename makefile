contracts_dir=contracts
compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum

build-release=RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked
build-debug=RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked --features="debug-print"

# args (no extensions): wasm_name, contract_dir_name
define opt_and_compress = 
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$(2).wasm -o ./$(1).wasm
echo $(md5sum $(1).wasm | cut -f 1 -d " ") >> ${checksum_dir}/$(1).txt
cat ./$(1).wasm | gzip -n -9 > ${compiled_dir}/$(1).wasm.gz
rm ./$(1).wasm
endef

CONTRACTS = airdrop governance staking mint treasury micro_mint oracle mock_band initializer scrt_staking snip20

debug: setup
	(cd ${contracts_dir}; ${build-debug})
	@$(MAKE) compress_all

release: setup
	(cd ${contracts_dir}; ${build-release})
	@$(MAKE) compress_all

compress_all: setup
	@$(MAKE) $(addprefix compress-,$(CONTRACTS))

compress-snip20: setup
	$(call opt_and_compress,snip20,snip20_reference_impl)

compress-%: setup
	$(call opt_and_compress,$*,$*)

$(CONTRACTS): setup
	(cd ${contracts_dir}/$@; ${build-debug})
	@$(MAKE) $(addprefix compress-,$(@))

snip20: setup
	(cd ${contracts_dir}/snip20; ${build-release})
	@$(MAKE) $(addprefix compress-,snip20)

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
