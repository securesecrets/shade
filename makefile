contracts_dir=contracts
compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum

build-release=RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
build-debug=RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --features="debug-print"

# args (no extensions): wasm_name, contract_dir_name
define opt_and_compress = 
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$(2).wasm -o ./$(1).wasm
echo $(md5sum $(1).wasm | cut -f 1 -d " ") >> ${checksum_dir}/$(1).txt
cat ./$(1).wasm | gzip -n -9 > ${compiled_dir}/$(1).wasm.gz
rm ./$(1).wasm
endef

CONTRACTS = \
		airdrop governance shd_staking mint mint_router \
		treasury treasury_manager scrt_staking rewards_emission \
    oracle initializer snip20 \
		mock_band mock_secretswap_pair mock_sienna_pair

debug: setup
	(cd ${contracts_dir}; ${build-debug})
	@$(MAKE) compress_all

release: setup
	(cd ${contracts_dir}; ${build-release})
	@$(MAKE) compress_all

dao: treasury treasury_manager scrt_staking rewards_emission

compress_all: setup
	@$(MAKE) $(addprefix compress-,$(CONTRACTS))

compress-snip20: setup
	$(call opt_and_compress,snip20,snip20_reference_impl)

compress-shd_staking: setup
	$(call opt_and_compress,shd_staking,spip_stkd_0)

compress-%: setup
	$(call opt_and_compress,$*,$*)

$(CONTRACTS): setup
	(cd ${contracts_dir}/$@; ${build-debug})
	@$(MAKE) $(addprefix compress-,$(@))

snip20: setup
	(cd ${contracts_dir}/snip20; ${build-release})
	@$(MAKE) $(addprefix compress-,snip20)


test:
	@$(MAKE) $(addprefix test-,$(CONTRACTS))

test-%:
	(cd ${contracts_dir}/$*; cargo test)

shd_staking: setup
	(cd ${contracts_dir}/shd_staking; ${build-release})
	@$(MAKE) $(addprefix compress-,shd_staking)

setup: $(compiled_dir) $(checksum_dir)

$(compiled_dir) $(checksum_dir):
	mkdir $@

check:
	cargo check

clippy:
	cargo clippy

clean:
	find . -name "Cargo.lock" -delete
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
