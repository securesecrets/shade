import copy
import json
import random
from contractlib.contractlib import PreInstantiatedContract
from contractlib.secretlib import secretlib
from contractlib.snip20lib import SNIP20
from contractlib.mintlib import Mint
from contractlib.oraclelib import Oracle
from contractlib.utils import gen_label

account_key = 'admin'
account = secretlib.run_command(['secretcli', 'keys', 'show', '-a', account_key]).rstrip()

with open("testnet-contracts.json", "r") as json_file:
    contracts_config = json.load(json_file)

print("Configuring silk")
silk_updated = False
silk_instantiated_contract = PreInstantiatedContract(
    contract_id=contracts_config["silk"]["contract_id"],
    address=contracts_config["silk"]["address"],
    code_hash=contracts_config["silk"]["code_hash"])
silk = SNIP20(contracts_config["silk"]["label"], "silk", "SLK", decimals=6, public_total_supply=True, enable_mint=True,
              admin=account, uploader=account, backend=None, instantiated_contract=silk_instantiated_contract)
silk_password = silk.set_view_key(account_key, "password")
silk.print()

print("Configuring sSCRT")
sscrt = copy.deepcopy(silk)
sscrt.label = contracts_config["sscrt"]["label"]
sscrt.address = contracts_config["sscrt"]["address"]
sscrt.code_hash = contracts_config["sscrt"]["code_hash"]
sscrt_password = sscrt.set_view_key(account_key, "password")
sscrt.print()

print("Configuring Oracle")
oracle_updated = False
with open("checksum/oracle.txt", 'r') as oracle_checksum:
    if oracle_checksum.readline().strip() == contracts_config["oracle"]["checksum"].strip():
        oracle_instantiated_contract = PreInstantiatedContract(
            contract_id=contracts_config["oracle"]["contract_id"],
            address=contracts_config["oracle"]["address"],
            code_hash=contracts_config["oracle"]["code_hash"])
        oracle = Oracle(contracts_config["oracle"]["label"], admin=account, uploader=account, backend=None,
                        instantiated_contract=oracle_instantiated_contract)
    else:
        print("Instantiating Oracle")
        oracle_updated = True
        contracts_config["oracle"]["label"] = f"oracle-{gen_label(8)}"
        oracle = Oracle(contracts_config["oracle"]["label"], admin=account, uploader=account, backend=None)
        contracts_config["oracle"]["contract_id"] = oracle.contract_id
        contracts_config["oracle"]["address"] = oracle.address
        contracts_config["oracle"]["code_hash"] = oracle.code_hash

print(oracle.get_silk_price())
oracle.print()

print("Configuring Mint")
mint_updated = False
with open("checksum/mint.txt", 'r') as mint_checksum:
    mint_instantiated_contract = PreInstantiatedContract(
        contract_id=contracts_config["mint"]["contract_id"],
        address=contracts_config["mint"]["address"],
        code_hash=contracts_config["mint"]["code_hash"])
    mint = Mint(contracts_config["mint"]["label"], silk, oracle, admin=account, uploader=account, backend=None,
                    instantiated_contract=mint_instantiated_contract)

    if mint_checksum.readline().strip() != contracts_config["mint"]["checksum"].strip():
        print("Instantiating Mint")
        mint_updated = True
        label = f"mint-{gen_label(8)}"
        # TODO: upload and get codehash + id of the contract without instantiating to call the mint.migrate
        new_mint = Mint(label, silk, oracle, admin=account, uploader=account, backend=None)
        # mint.migrate()
        mint = copy.deepcopy(new_mint)
        contracts_config["mint"]["label"] = label
        contracts_config["mint"]["contract_id"] = mint.contract_id
        contracts_config["mint"]["address"] = mint.address
        contracts_config["mint"]["code_hash"] = mint.code_hash

if silk_updated or oracle_updated:
    mint.update_config(silk=silk, oracle=oracle)

if silk_updated or mint_updated:
    # TODO: reset minters if mint updated
    silk.set_minters([mint.address])

if mint_updated:
    mint.register_asset(sscrt)

assets = mint.get_supported_assets()['supported_assets']['assets'][0]
assert sscrt.address == assets, f"Got {assets}; expected {sscrt.address}"
mint.print()