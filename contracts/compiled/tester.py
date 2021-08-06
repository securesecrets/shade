import copy
import json
import base64
import random
import argparse
from contractlib.contractlib import PreInstantiatedContract
from contractlib.secretlib import secretlib
from contractlib.snip20lib import SNIP20
from contractlib.mintlib import Mint
from contractlib.oraclelib import Oracle
from contractlib.utils import gen_label

parser = argparse.ArgumentParser(description='Automated smart contract tester')
parser.add_argument("--testnet", choices=["private", "public"], default="private", type=str, required=False,
                    help="Specify which deploy scenario to run")

args=parser.parse_args()

if args.testnet == "private":
    account_key = 'a'
    account = secretlib.run_command(['secretcli', 'keys', 'show', '-a', account_key]).rstrip()

    print("Configuring sSCRT")
    sscrt = SNIP20(gen_label(8), decimals=6, public_total_supply=True, enable_deposit=True)
    sscrt_password = sscrt.set_view_key(account_key, "password")

    sscrt_mint_amount = '100000000000000'
    print(f"\tDepositing {sscrt_mint_amount} uSCRT")
    sscrt.deposit(account, sscrt_mint_amount + "uscrt")
    sscrt_minted = sscrt.get_balance(account, sscrt_password)
    print(f"\tReceived {sscrt_minted} usSCRT")
    assert sscrt_mint_amount == sscrt_minted, f"Minted {sscrt_minted}; expected {sscrt_mint_amount}"

    print("Configuring silk")
    silk = SNIP20(gen_label(8), decimals=6, public_total_supply=True, enable_mint=True)
    silk_password = silk.set_view_key(account_key, "password")

    print('Configuring Oracle')
    oracle = Oracle(gen_label(8))
    price = int(oracle.get_scrt_price()["rate"])
    print(price / (10**18))

    print("Configuring Mint contract")
    mint = Mint(gen_label(8), silk, oracle)
    silk.set_minters([mint.address])
    mint.register_asset(sscrt)
    assets = mint.get_supported_assets()['supported_assets']['assets'][0]
    assert sscrt.address == assets, f"Got {assets}; expected {sscrt.address}"

    print("Sending to mint contract")

    total_amount = int(sscrt_mint_amount)
    minimum_amount = 1000
    total_tests = 5

    total_sent = 0

    for i in range(total_tests):
        send_amount = random.randint(minimum_amount, int(total_amount/total_tests)-1)
        total_sent += send_amount

        print(f"\tSending {send_amount} usSCRT")
        # {"snip_msg_hook": {
        #     "minimum_expected_amount": "1",
        #     "mint_type": {"mint_silk": {}}}}
        mint_option = "eyJtaW5pbXVtX2V4cGVjdGVkX2Ftb3VudCI6ICIxIiwgIm1pbnRfdHlwZSI6IHsibWludF9zaWxrIjoge319fQ=="
        # This one will fail because mint will never exceed its expected amount
        #mint_option = "eyJtaW5pbXVtX2V4cGVjdGVkX2Ftb3VudCI6ICIxNTkzNzA1MTUzMzg1NjAwMDAiLCAibWludF90eXBlIjogeyJtaW50X3NpbGsiOiB7fX19"
        print(sscrt.send(account_key, mint.address, send_amount, mint_option))
        silk_minted = silk.get_balance(account, silk_password)
        #assert total_sent == int(silk_minted), f"Total minted {silk_minted}; expected {total_sent}"

        print(f"\tSilk balance: {silk_minted} uSILK")
        burned_amount = mint.get_asset(sscrt)["asset"]["asset"]["burned_tokens"]
        print(f"\tTotal burned: {burned_amount} usSCRT\n")
        #assert total_sent == int(burned_amount), f"Burnt {burned_amount}; expected {total_sent}"

    print("Testing migration")
    new_mint = mint.migrate(gen_label(8), int(mint.contract_id), mint.code_hash)
    assert mint.get_supported_assets() == new_mint.get_supported_assets(), "Contracts are not the same"

if args.testnet == "public":
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

    # Save json data
    with open('testnet-contracts.json', 'w', encoding='utf-8') as json_file:
        json.dump(contracts_config, json_file, ensure_ascii=False, indent=4)