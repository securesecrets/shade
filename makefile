compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum

build-release=RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
# build-debug=RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --features="debug-print"

# args (no extensions): wasm_name, contract_dir_name
define opt_and_compress = 
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$(2).wasm -o ./$(1).wasm
echo $(md5sum $(1).wasm | cut -f 1 -d " ") >> ${checksum_dir}/$(1).txt
cat ./$(1).wasm | gzip -n -9 > ${compiled_dir}/$(1).wasm.gz
rm ./$(1).wasm
endef

CONTRACTS = \
		airdrop treasury treasury_manager scrt_staking \
		snip20 query_auth admin \
		mock_sienna_pair mock_adapter \
		mock_stkd_derivative basic_staking snip20_migration stkd_scrt \
		snip20_derivative

PACKAGES = shade_protocol contract_harness cosmwasm_math_compat 

release: setup
	${build-release}
	@$(MAKE) compress_all

dao: treasury treasury_manager scrt_staking

compress_all: setup
	@$(MAKE) $(addprefix compress-,$(CONTRACTS))

compress-snip20_staking: setup
	$(call opt_and_compress,snip20_staking,spip_stkd_0)

compress-%: setup
	$(call opt_and_compress,$*,$*)

$(CONTRACTS): setup
	(${build-release} -p $@)
	@$(MAKE) compress-$(@)

$(PACKAGES):
	(cd packages/$@; cargo build)

snip20: setup
	(cd contracts/snip20; ${build-release})
	@$(MAKE) $(addprefix compress-,snip20)

snip20_staking: setup
	(cd contracts/snip20_staking; ${build-release})
	@$(MAKE) $(addprefix compress-,snip20_staking)

test:
	@$(MAKE) $(addprefix test-,$(CONTRACTS))

test-%: %
	(cargo test -p $*)

dao-cov:
	(cargo llvm-cov --html -p treasury -p treasury_manager; xdg-open target/llvm-cov/html/index.html)

cov:
	(cargo llvm-cov --html; xdg-open target/llvm-cov/html/index.html)

setup: $(compiled_dir) $(checksum_dir)

$(compiled_dir) $(checksum_dir):
	mkdir $@

check:
	cargo check

clippy:
	cargo clippy

clean:
	find . -name "Cargo.lock" -delete
	rm -rf target
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
