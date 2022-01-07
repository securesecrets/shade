contracts_dir=contracts
compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum

define build_wasm =
(cd $(contracts_dir)/$(1); cargo unit-test)
(cd ${contracts_dir}; cargo build --release --target wasm32-unknown-unknown --locked)
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$(2).wasm -o ./$(1).wasm
echo $(md5sum $(1).wasm | cut -f 1 -d " ") >> ${checksum_dir}/$(1).txt
cat ./$(1).wasm | gzip -n -9 > ${compiled_dir}/$(1).wasm.gz
rm ./$(1).wasm
endef

CONTRACTS = airdrop governance staking mint treasury micro_mint oracle mock_band initializer scrt_staking

COMPILED = ${CONTRACTS:=.wasm.gz}

all: setup $(CONTRACTS); $(call build_wasm,snip20,snip20_reference_impl)

$(CONTRACTS):
	$(call build_wasm,$@,$@)

setup: $(compiled_dir) $(checksum_dir)

$(compiled_dir) $(checksum_dir):
	mkdir $@

clean:
	rm -r $(CONTRACTS)

format:
	cargo fmt
