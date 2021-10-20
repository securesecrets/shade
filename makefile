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

CONTRACTS = initializer airdrop governance staking mint micro_mint treasury oracle mock_band

COMPILED = ${CONTRACTS:=.wasm.gz}

all: setup $(CONTRACTS); $(call build_wasm,snip20,snip20_reference_impl)

$(CONTRACTS):
	$(call build_wasm,$@,$@)


setup: $(compiled_dir) $(checksum_dir)

$(compiled_dir) $(checksum_dir):
	mkdir $@
