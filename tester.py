#!/usr/bin/env python3 
import copy
import json
import random
import argparse
from contractlib.contractlib import PreInstantiatedContract
from contractlib.contractlib import Contract
from contractlib.secretlib import secretlib
from contractlib.snip20lib import SNIP20
from contractlib.mintlib import Mint
from contractlib.micro_mintlib import MicroMint
from contractlib.oraclelib import Oracle
from contractlib.utils import gen_label, to_base64


def test_send(burn_asset, burn_asset_password, mint_asset, mint_asset_password, mint, slipage, amount, account, account_key):
    # Save token symbol
    burn_asset_symbol = burn_asset.get_token_info()["token_info"]["symbol"]
    mint_asset_symbol = mint_asset.get_token_info()["token_info"]["symbol"]

    # Get all the token amounts before sending
    user_burn_asset_before = burn_asset.get_balance(account, burn_asset_password)
    user_mint_asset_before = mint_asset.get_balance(account, mint_asset_password)
    burn_asset_supply_before = burn_asset.get_token_info()["token_info"]["total_supply"]
    queried_burn_asset_supply_before = mint.get_asset(burn_asset)["asset"]["burned"]
    mint_asset_supply_before = mint_asset.get_token_info()["token_info"]["total_supply"]

    # Get all the token amounts after sending
    msg = to_base64({"minimum_expected_amount": str(slipage), "to_mint": mint_asset.address})
    send_response = burn_asset.send(account_key, mint.address, amount, msg)

    if send_response["output_error"] != {}:
        print(f"Mint error: {send_response['output_error']}")

    user_burn_asset_after = burn_asset.get_balance(account, burn_asset_password)
    user_mint_asset_after = mint_asset.get_balance(account, mint_asset_password)
    burn_asset_supply_after = burn_asset.get_token_info()["token_info"]["total_supply"]
    queried_burn_asset_supply_after = mint.get_asset(burn_asset)["asset"]["burned"]
    mint_asset_supply_after = mint_asset.get_token_info()["token_info"]["total_supply"]
    print(f"Sending:    {amount} u{burn_asset_symbol} to receive u{mint_asset_symbol}\n"
          f"Sent:       {int(user_burn_asset_before) - int(user_burn_asset_after)} u{burn_asset_symbol}\n"
          f"Burned      {int(burn_asset_supply_before) - int(burn_asset_supply_after)} u{burn_asset_symbol}\n"
          f"Burn Query: {int(queried_burn_asset_supply_after) - int(queried_burn_asset_supply_before)} u{burn_asset_symbol}\n"
          f"Received:   {int(user_mint_asset_after) - int(user_mint_asset_before)} u{mint_asset_symbol}\n"
          f"Mint:       {int(mint_asset_supply_after) - int(mint_asset_supply_before)} u{mint_asset_symbol}\n")


parser = argparse.ArgumentParser(description='Automated smart contract tester')
parser.add_argument("--testnet", choices=["private", "public"], default="private", type=str, required=False,
                    help="Specify which deploy scenario to run")

args = parser.parse_args()
password = "password"

if args.testnet == "private":
    account_key = 'a'
    account = secretlib.run_command(['secretcli', 'keys', 'show', '-a', account_key]).rstrip()

    print("Configuring sSCRT")
    sscrt = SNIP20(gen_label(8), name="sSCRT", symbol="SSCRT", decimals=6, public_total_supply=True,
                   enable_deposit=True)
    sscrt.set_view_key(account_key, password)

    sscrt_mint_amount = '100000000000000'
    print(f"\tDepositing {sscrt_mint_amount} uSCRT")
    sscrt.deposit(account, sscrt_mint_amount + "uscrt")
    sscrt_minted = sscrt.get_balance(account, password)
    print(f"\tReceived {sscrt_minted} usSCRT")
    assert sscrt_mint_amount == sscrt_minted, f"Minted {sscrt_minted}; expected {sscrt_mint_amount}"

    print("Configuring Silk")
    silk = SNIP20(gen_label(8), name="Silk", symbol="SILK", decimals=6, public_total_supply=True, enable_mint=True,
                  enable_burn=True)
    silk.set_view_key(account_key, password)

    print("Configuring Shade")
    shade = SNIP20(gen_label(8), name="Shade", symbol="SHD", decimals=6, public_total_supply=True, enable_mint=True,
                   enable_burn=True)
    shade.set_view_key(account_key, password)

    print('Mocking Band')
    band = Contract('mock_band.wasm.gz', '{}', gen_label(8))

    print('Configuring Oracle')
    oracle = Oracle(gen_label(8), band)
    price = int(oracle.get_price('SCRT')["rate"])
    print(price / (10 ** 18))

    print("Configuring Mint contract")
    mint = Mint(gen_label(8), oracle=oracle)
    mint.register_asset(silk, burnable=True)
    mint.register_asset(shade, burnable=True)
    mint.register_asset(sscrt, name="SCRT")
    assets = mint.get_supported_assets()['supported_assets']['assets']
    assert 3 == len(assets), f"Got {len(assets)}; expected {3}"

    print("Configuring Silk-Mint Contract")
    silk_mint = MicroMint(gen_label(8), native_asset=silk, oracle=oracle)
    silk_mint.register_asset(sscrt, False)
    silk_mint.register_asset(shade, True)

    print("Configuring Shade-Mint Contract")
    shade_mint = MicroMint(gen_label(8), native_asset=shade, oracle=oracle)
    shade_mint.register_asset(sscrt, False)
    shade_mint.register_asset(silk, True)

    print("Setting minters")
    silk.set_minters([mint.address, silk_mint.address])
    shade.set_minters([mint.address, shade_mint.address])

    print("Sending to mint contract")

    total_amount = int(sscrt_mint_amount)
    minimum_amount = 1000
    total_tests = 1

    total_sent = 0

    for i in range(total_tests):
        send_amount = random.randint(minimum_amount, int(total_amount / total_tests / 2) - 1)
        total_sent += send_amount

        test_send(sscrt, password, silk, password, silk_mint, 1, send_amount, account, account_key)
        test_send(sscrt, password, shade, password, shade_mint, 1, send_amount, account, account_key)

    send_amount = 1_000_000_000
    test_send(silk, password, shade, password, shade_mint, 1, send_amount, account, account_key)

    send_amount = 10_000_000
    test_send(shade, password, silk, password, silk_mint, 1, send_amount, account, account_key)

    print("Testing migration")
    new_mint = mint.migrate(gen_label(8), int(mint.code_id), mint.code_hash)
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
            code_hash=contracts_config["silk"]["code_hash"],
            code_id=contracts_config["silk"]["code_id"])

    silk = SNIP20(contracts_config["silk"]["label"], "silk", "SLK", decimals=6, public_total_supply=True,
                  enable_mint=True, enable_burn=True, admin=account, uploader=account, backend=None,
                  instantiated_contract=silk_instantiated_contract, code_id=contracts_config["silk"]["code_id"])

    contracts_config["silk"]["address"] = silk.address
    contracts_config["silk"]["code_hash"] = silk.code_hash
    silk.set_view_key(account_key, password)
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
            code_hash=contracts_config["shade"]["code_hash"],
            code_id=contracts_config["shade"]["code_id"])

    shade = SNIP20(contracts_config["shade"]["label"], "shade", "SHD", decimals=6, public_total_supply=True,
                   enable_mint=True, enable_burn=True, admin=account, uploader=account, backend=None,
                   instantiated_contract=shade_instantiated_contract, code_id=contracts_config["shade"]["code_id"])

    contracts_config["shade"]["address"] = shade.address
    contracts_config["shade"]["code_hash"] = shade.code_hash
    contracts_config["shade"]["code_id"] = shade.code_id
    shade.set_view_key(account_key, password)
    shade.print()

    print("Configuring sSCRT")
    sscrt = copy.deepcopy(silk)
    sscrt.label = contracts_config["sscrt"]["label"]
    sscrt.address = contracts_config["sscrt"]["address"]
    sscrt.code_hash = contracts_config["sscrt"]["code_hash"]
    sscrt.set_view_key(account_key, password)
    sscrt.print()

    print("Configuring Oracle")
    oracle_updated = False
    band_contract = PreInstantiatedContract("secret1p0jtg47hhwuwgp4cjpc46m7qq6vyjhdsvy2nph",
                                            "77c854ea110315d5103a42b88d3e7b296ca245d8b095e668c69997b265a75ac5")
    with open("checksum/oracle.txt", 'r') as oracle_checksum:
        if oracle_checksum.readline().strip() == contracts_config["oracle"]["checksum"].strip():
            oracle_instantiated_contract = PreInstantiatedContract(
                address=contracts_config["oracle"]["address"],
                code_hash=contracts_config["oracle"]["code_hash"],
                code_id=contracts_config["oracle"]["code_id"]
            )
            oracle = Oracle(contracts_config["oracle"]["label"], band_contract, admin=account, uploader=account,
                            backend=None,
                            instantiated_contract=oracle_instantiated_contract,
                            code_id=contracts_config["oracle"]["code_id"])
        else:
            print("Instantiating Oracle")
            oracle_updated = True
            contracts_config["oracle"]["label"] = f"oracle-{gen_label(8)}"
            oracle = Oracle(contracts_config["oracle"]["label"], band_contract, admin=account, uploader=account,
                            backend=None)
            contracts_config["oracle"]["code_id"] = oracle.code_id
            contracts_config["oracle"]["address"] = oracle.address
            contracts_config["oracle"]["code_hash"] = oracle.code_hash

    print(oracle.get_price('SCRT'))
    oracle.print()

    print("Configuring Mint")
    mint_updated = False
    with open("checksum/mint.txt", 'r') as mint_checksum:
        mint_instantiated_contract = PreInstantiatedContract(
            address=contracts_config["mint"]["address"],
            code_hash=contracts_config["mint"]["code_hash"],
            code_id=contracts_config["mint"]["code_id"])
        mint = Mint(contracts_config["mint"]["label"], silk, shade, oracle, admin=account, uploader=account,
                    backend=None,
                    instantiated_contract=mint_instantiated_contract, code_id=contracts_config["mint"]["code_id"])

        if mint_checksum.readline().strip() != contracts_config["mint"]["checksum"].strip():
            print("Instantiating Mint")
            mint_updated = True
            label = f"mint-{gen_label(8)}"
            # TODO: upload and get codehash + id of the contract without instantiating to call the mint.migrate
            new_mint = Mint(label, silk, shade, oracle, admin=account, uploader=account, backend=None)
            # mint.migrate()
            mint = copy.deepcopy(new_mint)
            contracts_config["mint"]["label"] = label
            contracts_config["mint"]["code_id"] = mint.code_id
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
