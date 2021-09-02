contracts_dir=contracts
compiled_dir=compiled
checksum_dir=${compiled_dir}/checksum

CONTRACTS = mint snip20 treasury micro_mint oracle mock_band initializer

COMPILED = ${CONTRACTS:=.wasm.gz}

all: setup $(CONTRACTS)

$(CONTRACTS):
	(cd $(contracts_dir)/$@; cargo unit-test)
	(cd ${contracts_dir}; cargo build --release --target wasm32-unknown-unknown --locked)
	wasm-opt -Oz ./target/wasm32-unknown-unknown/release/$@.wasm -o ./$@.wasm
	echo $(md5sum $@.wasm | cut -f 1 -d " ") >> ${checksum_dir}/$@.txt
	cat ./$@.wasm | gzip -n -9 > ${compiled_dir}/$@.wasm.gz
	rm ./$@.wasm

setup: $(compiled_dir) $(checksum_dir)

$(compiled_dir) $(checksum_dir):
	mkdir $@
