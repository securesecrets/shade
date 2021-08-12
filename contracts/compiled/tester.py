#!/usr/bin/env python3 
import copy
import json
import base64
import random
import argparse
from contractlib.contractlib import PreInstantiatedContract
from contractlib.contractlib import Contract
from contractlib.secretlib import secretlib
from contractlib.snip20lib import SNIP20
from contractlib.mintlib import Mint
from contractlib.oraclelib import Oracle
from contractlib.utils import gen_label

parser = argparse.ArgumentParser(description='Automated smart contract tester')
parser.add_argument("--testnet", choices=["private", "public"], default="private", type=str, required=False,
                    help="Specify which deploy scenario to run")

args = parser.parse_args()

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

    print("Configuring Silk")
    silk = SNIP20(gen_label(8), decimals=6, public_total_supply=True, enable_mint=True, enable_burn=True)
    silk_password = silk.set_view_key(account_key, "password")

    print("Configuring Shade")
    shade = SNIP20(gen_label(8), decimals=6, public_total_supply=True, enable_mint=True, enable_burn=True)
    shade_password = shade.set_view_key(account_key, "password")

    print('Mocking Band')
    band = Contract('mock_band.wasm.gz', '{}', gen_label(8))

    print('Configuring Oracle')
    oracle = Oracle(gen_label(8), band)
    price = int(oracle.get_scrt_price()["rate"])
    print(price / (10 ** 18))

    print("Configuring Mint contract")
    mint = Mint(gen_label(8), silk, shade, oracle)
    silk.set_minters([mint.address])
    shade.set_minters([mint.address])
    mint.register_asset(sscrt)
    assets = mint.get_supported_assets()['supported_assets']['assets'][0]
    assert sscrt.address == assets, f"Got {assets}; expected {sscrt.address}"

    print("Sending to mint contract")

    total_amount = int(sscrt_mint_amount)
    minimum_amount = 1000
    total_tests = 1

    total_sent = 0

    for i in range(total_tests):
        send_amount = random.randint(minimum_amount, int(total_amount / total_tests / 2) - 1)
        total_sent += send_amount

        print(f"\tSending {send_amount} usSCRT")

        # {"minimum_expected_amount": "1", "mint_type": {"coin_to_silk": {}}}
        mint_silk = "eyJtaW5pbXVtX2V4cGVjdGVkX2Ftb3VudCI6ICIxIiwgIm1pbnRfdHlwZSI6IHsiY29pbl90b19zaWxrIjoge319fQ=="
        mint_silk_response = sscrt.send(account_key, mint.address, send_amount, mint_silk)
        if mint_silk_response["output_error"] != {}:
            print(f"Silk mint error: {mint_silk_response['output_error']}")
        silk_minted = silk.get_balance(account, silk_password)
        print(f"\tSilk balance: {silk_minted} uSILK")

        # { "minimum_expected_amount": "1", "mint_type": {"coin_to_shade": {}}}
        mint_shade = "eyAibWluaW11bV9leHBlY3RlZF9hbW91bnQiOiAiMSIsICJtaW50X3R5cGUiOiB7ImNvaW5fdG9fc2hhZGUiOiB7fX19"
        mint_shade_response = sscrt.send(account_key, mint.address, send_amount, mint_shade)
        if mint_shade_response["output_error"] != {}:
            print(f"Shade mint error: {mint_shade_response['output_error']}")
        shade_minted = shade.get_balance(account, shade_password)
        print(f"\tShade balance: {shade_minted} uSHD")

        burned_amount = mint.get_asset(sscrt)["asset"]["asset"]["burned_tokens"]
        print(f"\tTotal burned: {burned_amount} usSCRT\n")

    send_amount = 1_000_000_000
    print(f"Converting {send_amount} uSilk into Shade")
    user_silk_before = silk.get_balance(account, silk_password)
    user_shade_before = shade.get_balance(account, shade_password)
    silk_supply_before = silk.get_token_info()["token_info"]["total_supply"]
    shade_supply_before = shade.get_token_info()["token_info"]["total_supply"]
    # { "minimum_expected_amount": "1", "mint_type": {"convert_to_shade": {}}}
    msg = "eyAKICAgICJtaW5pbXVtX2V4cGVjdGVkX2Ftb3VudCI6ICIxIiwgCiAgICAibWludF90eXBlIjogewogICAgICAgICAgICAgICAgICAgICJjb252ZXJ0X3RvX3NoYWRlIjoge30KICAgICAgICAgICAgICAgICB9Cn0="
    silk.send(account_key, mint.address, send_amount, msg)
    user_silk_after = silk.get_balance(account, silk_password)
    user_shade_after = shade.get_balance(account, shade_password)
    silk_supply_after = silk.get_token_info()["token_info"]["total_supply"]
    shade_supply_after = shade.get_token_info()["token_info"]["total_supply"]
    print(f"Sent {int(user_silk_before) - int(user_silk_after)} uSilk and received {int(user_shade_after) - int(user_shade_before)} uSHD")
    print(f"Burned {int(silk_supply_before) - int(silk_supply_after)} uSilk and minted {int(shade_supply_after) - int(shade_supply_before)} uSHD")

    send_amount = 10_000_000
    print(f"Converting {send_amount} uSHD into Silk")
    user_silk_before = silk.get_balance(account, silk_password)
    user_shade_before = shade.get_balance(account, shade_password)
    silk_supply_before = silk.get_token_info()["token_info"]["total_supply"]
    shade_supply_before = shade.get_token_info()["token_info"]["total_supply"]
    # { "minimum_expected_amount": "1", "mint_type": {"convert_to_silk": {}}}
    msg = "eyAibWluaW11bV9leHBlY3RlZF9hbW91bnQiOiAiMSIsICJtaW50X3R5cGUiOiB7ImNvbnZlcnRfdG9fc2lsayI6IHt9fX0="
    shade.send(account_key, mint.address, send_amount, msg)
    user_silk_after = silk.get_balance(account, silk_password)
    user_shade_after = shade.get_balance(account, shade_password)
    silk_supply_after = silk.get_token_info()["token_info"]["total_supply"]
    shade_supply_after = shade.get_token_info()["token_info"]["total_supply"]
    print(f"Sent {int(user_shade_before) - int(user_shade_after)} uSHD and received {int(user_silk_after) - int(user_silk_before)} uSilk")
    print(f"Burned {int(shade_supply_before) - int(shade_supply_after)} uSHD and minted {int(silk_supply_after) - int(silk_supply_before)} uSilk")

    print("Testing migration")
    new_mint = mint.migrate(gen_label(8), int(mint.contract_id), mint.code_hash)
    assert mint.get_supported_assets() == new_mint.get_supported_assets(), "Contracts are not the same"

if args.testnet == "public":
    account_key = 'admin'
    account = secretlib.run_command(['secretcli', 'keys', 'show', '-a', account_key]).rstrip()

    with open("testnet-contracts.json", "r") as json_file:
        contracts_config = json.load(json_file)

    print("Configuring Silk")
    silk_updated = False
    if contracts_config["silk"]["address"] == "":
        print("Instantiating Silk")
        contracts_config["silk"]["label"] = f"silk-{gen_label(8)}"
        silk_instantiated_contract = None
        silk_updated = True
    else:
        silk_instantiated_contract = PreInstantiatedContract(
            address=contracts_config["silk"]["address"],
            code_hash=contracts_config["silk"]["code_hash"])

    silk = SNIP20(contracts_config["silk"]["label"], "silk", "SLK", decimals=6, public_total_supply=True,
                  enable_mint=True, enable_burn=True, admin=account, uploader=account, backend=None,
                  instantiated_contract=silk_instantiated_contract, code_id=contracts_config["silk"]["contract_id"])

    contracts_config["silk"]["address"] = silk.address
    contracts_config["silk"]["code_hash"] = silk.code_hash
    silk_password = silk.set_view_key(account_key, "password")
    silk.print()

    print("Configuring shade")
    shade_updated = False
    if contracts_config["shade"]["address"] == "":
        print("Instantiating Shade")
        contracts_config["shade"]["label"] = f"shade-{gen_label(8)}"
        shade_instantiated_contract = None
        shade_updated = True
    else:
        shade_instantiated_contract = PreInstantiatedContract(
            address=contracts_config["shade"]["address"],
            code_hash=contracts_config["shade"]["code_hash"])

    shade = SNIP20(contracts_config["shade"]["label"], "shade", "SHD", decimals=6, public_total_supply=True,
                  enable_mint=True, enable_burn=True, admin=account, uploader=account, backend=None,
                  instantiated_contract=shade_instantiated_contract, code_id=contracts_config["shade"]["contract_id"])

    contracts_config["shade"]["address"] = shade.address
    contracts_config["shade"]["code_hash"] = shade.code_hash
    shade_password = shade.set_view_key(account_key, "password")
    shade.print()

    print("Configuring sSCRT")
    sscrt = copy.deepcopy(silk)
    sscrt.label = contracts_config["sscrt"]["label"]
    sscrt.address = contracts_config["sscrt"]["address"]
    sscrt.code_hash = contracts_config["sscrt"]["code_hash"]
    sscrt_password = sscrt.set_view_key(account_key, "password")
    sscrt.print()

    print("Configuring Oracle")
    oracle_updated = False
    band_contract = PreInstantiatedContract("secret1p0jtg47hhwuwgp4cjpc46m7qq6vyjhdsvy2nph", "77c854ea110315d5103a42b88d3e7b296ca245d8b095e668c69997b265a75ac5")
    with open("checksum/oracle.txt", 'r') as oracle_checksum:
        if oracle_checksum.readline().strip() == contracts_config["oracle"]["checksum"].strip():
            oracle_instantiated_contract = PreInstantiatedContract(
                address=contracts_config["oracle"]["address"],
                code_hash=contracts_config["oracle"]["code_hash"])
            oracle = Oracle(contracts_config["oracle"]["label"], band_contract, admin=account, uploader=account, backend=None,
                            instantiated_contract=oracle_instantiated_contract,
                            code_id=contracts_config["oracle"]["contract_id"])
        else:
            print("Instantiating Oracle")
            oracle_updated = True
            contracts_config["oracle"]["label"] = f"oracle-{gen_label(8)}"
            oracle = Oracle(contracts_config["oracle"]["label"], band_contract, admin=account, uploader=account, backend=None)
            contracts_config["oracle"]["contract_id"] = oracle.contract_id
            contracts_config["oracle"]["address"] = oracle.address
            contracts_config["oracle"]["code_hash"] = oracle.code_hash

    print(oracle.get_scrt_price())
    oracle.print()

    print("Configuring Mint")
    mint_updated = False
    with open("checksum/mint.txt", 'r') as mint_checksum:
        mint_instantiated_contract = PreInstantiatedContract(
            address=contracts_config["mint"]["address"],
            code_hash=contracts_config["mint"]["code_hash"])
        mint = Mint(contracts_config["mint"]["label"], silk, shade, oracle, admin=account, uploader=account, backend=None,
                    instantiated_contract=mint_instantiated_contract, code_id=contracts_config["mint"]["contract_id"])

        if mint_checksum.readline().strip() != contracts_config["mint"]["checksum"].strip():
            print("Instantiating Mint")
            mint_updated = True
            label = f"mint-{gen_label(8)}"
            # TODO: upload and get codehash + id of the contract without instantiating to call the mint.migrate
            new_mint = Mint(label, silk, shade, oracle, admin=account, uploader=account, backend=None)
            # mint.migrate()
            mint = copy.deepcopy(new_mint)
            contracts_config["mint"]["label"] = label
            contracts_config["mint"]["contract_id"] = mint.contract_id
            contracts_config["mint"]["address"] = mint.address
            contracts_config["mint"]["code_hash"] = mint.code_hash

    if silk_updated or oracle_updated or shade_updated:
        mint.update_config(silk=silk, oracle=oracle)

    if silk_updated or mint_updated:
        silk.set_minters([mint.address])

    if mint_updated:
        mint.register_asset(sscrt)

    assets = mint.get_supported_assets()['supported_assets']['assets'][0]
    assert sscrt.address == assets, f"Got {assets}; expected {sscrt.address}"
    mint.print()

    # Save json data
    with open('testnet-contracts.json', 'w', encoding='utf-8') as json_file:
        json.dump(contracts_config, json_file, ensure_ascii=False, indent=4)
