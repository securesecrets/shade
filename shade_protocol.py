#!/usr/bin/env python3 
import random
#import click
import json
from sys import exit, argv
from collections import defaultdict

from contractlib.contractlib import PreInstantiatedContract
from contractlib.contractlib import Contract
from contractlib.snip20lib import SNIP20
from contractlib.micro_mintlib import MicroMint
from contractlib.oraclelib import Oracle
from contractlib.treasurylib import Treasury
from contractlib.utils import gen_label

from contractlib.secretlib.secretlib import GAS_METRICS, run_command, execute_contract, query_contract

SHADE_FILE_NAME = 'shade_protocol.json'

# Burns for core_assets assets
entry_assets = [
    ('secretSCRT', 'SSCRT'),
]

core_assets = [
    # Core
    ('Shade', 'SHD'),
    ('Silk', 'SILK'),
]

# These will be generated into pairs of Synthetic/Stabilizer
synthetic_assets = [
    # Metals
    ('Gold', 'XAU'),
    ('Silver', 'XAG'),

    # Fiat
    #('Euro', 'EUR'),
    #('Japanese yen', 'JPY'),
    #('Yuan', 'CNY'),

    # Crypto
    #('Ethereum', 'ETH'),
    #('Bitcoin', 'BTC'),
    #('secret', 'SCRT'),
    #('Dogecoin', 'DOGE'),
    #('Uniswap', 'UNI'),
    #('Stellar', 'XLM'),
    #('PancakeSwap', 'CAKE'),
    #('Band Protocol', 'BAND'),
    #('Terra', 'LUNA'),
    #('Cosmos', 'ATOM'),

    # TODO: Oracle: add these sources
    # Stocks 
    # ('Tesla', 'TSLA'),
    # ('Apple', 'AAPL'),
    # ('Google', 'GOOG'),
]

BURN_MAP = {
    # minted: burners
    'SHD': ['SILK', 'SSCRT'],
    'SILK': ['SHD', 'SSCRT']
}

CAPTURE = {
    'SSCRT': 1, # 100%
    'SILK': .01, # 1%
    'SHD': .01, # 1%
}

# normalize capture values
CAPTURE = {
    k: int(v * 10000) for k, v in CAPTURE.items()
}

# Synthetics and ARB's burn for eachother
# SILK burns for Synthetic
for _, s in synthetic_assets:
    BURN_MAP[f'S{s}'] = [f'A{s}', 'SILK']
    BURN_MAP[f'A{s}'] = [f'S{s}', 'SILK']

# synthetic and stabilizer of each asset
synthetic_assets = [
    ('Synthetic ' + name, 'S' + symbol)
    for name, symbol in synthetic_assets
] + [
    ('Stabilizer ' + name, 'A' + symbol)
    for name, symbol in synthetic_assets
]

TESTNET_BAND = {
    "address": "secret1p0jtg47hhwuwgp4cjpc46m7qq6vyjhdsvy2nph",
    "code_hash": "77c854ea110315d5103a42b88d3e7b296ca245d8b095e668c69997b265a75ac5"
}

TESTNET_SSCRT = {
    'address': 'secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx',
    'code_hash': 'cd400fb73f5c99edbc6aab22c2593332b8c9f2ea806bf9b42e3a523f3ad06f62'
}

# viewing_key = '4b734d7a2e71fb277a9e3355f5c56d347f1012e1a9533eb7fdbb3ceceedad5fc'
viewing_key = 'password'

chain_config = run_command(['secretcli', 'config'])

chain_config = {
    key.strip('" '): val.strip('" ')
    for key, val in 
    (
        line.split('=') 
        for line in chain_config.split('\n')
        if line
    )
}
account_key = 'drpresident' if chain_config['chain-id'] == 'holodeck-2' else 'a'
backend = None if chain_config['chain-id'] == 'holodeck-2' else 'test'
account = run_command(['secretcli', 'keys', 'show', '-a', account_key]).rstrip()

def execute(contract: dict, msg: dict):
    return execute_contract(contract['address'], json.dumps(msg), user=account_key, backend=backend)

def query(contract: dict, msg: dict):
    return query_contract(contract['address'], json.dumps(msg))

def deploy():

    sscrt = None
    band = None

    shade_protocol = defaultdict(dict)

    shade_protocol['assets'] = defaultdict(dict)
    shade_protocol['burn_map'] = BURN_MAP

    if chain_config['chain-id'] == 'holodeck-2':
        print('Setting testnet SSCRT')
        sscrt = PreInstantiatedContract(TESTNET_SSCRT['address'], TESTNET_SSCRT['code_hash'])
        sscrt.symbol = 'SSCRT'
        print(sscrt.address)

        print('Setting testnet band')
        band = PreInstantiatedContract(TESTNET_BAND['address'], TESTNET_BAND['code_hash'])
        print(TESTNET_BAND['address'])

    elif chain_config['chain-id'] == 'enigma-pub-testnet-3':
        print('Configuring SSCRT')
        sscrt = SNIP20(gen_label(8), name='secretSCRT', symbol='SSCRT', decimals=6, public_total_supply=True, enable_deposit=True, enable_burn=True,
                    admin=account_key, uploader=account_key, backend=backend)
        print(sscrt.address)
        sscrt.set_view_key(account_key, viewing_key)

        print('Mocking Band for local chain')
        band = Contract('mock_band.wasm.gz', '{}', gen_label(8))
        print(band.address)


    print('Configuring Oracle')
    oracle = Oracle(gen_label(8), band, sscrt,
                    admin=account_key, uploader=account_key, backend=backend)
    shade_protocol['oracle'] = {
        'address': oracle.address,
        'code_hash': oracle.code_hash,
    }
    print(oracle.address)
    scrt_price = int(oracle.get_price('SCRT')['rate'])
    print('SCRT', scrt_price / (10 ** 18))
    sscrt_price = int(oracle.get_price('SSCRT')['rate'])
    print('SSCRT', scrt_price / (10 ** 18))
    for _, symbol in core_assets + synthetic_assets:
        print(symbol, int(oracle.get_price(symbol)['rate']) / (10 ** 18))

    print('Configuring Treasury')
    treasury = Treasury(gen_label(8), 
                admin=account_key, uploader=account_key, backend=backend)
    print(treasury.address)
    shade_protocol['treasury'] = {
        'address': treasury.address,
        'code_hash': treasury.code_hash,
    }

    print('Registering SSCRT with treasury')
    treasury.register_asset(sscrt)

    snip20_id = None
    mint_id = None

    tokens = { 'SSCRT': sscrt }

    print('\nCore Snips')
    core_snips = []

    for name, symbol in core_assets:

        print('\nConfiguring', name, symbol)
        snip = SNIP20(gen_label(8), name=name, symbol=symbol, decimals=6, 
                public_total_supply=True, enable_mint=True, enable_burn=True, 
                admin=account_key, uploader=account_key, backend=backend,
                code_id=snip20_id)

        if not snip20_id:
            snip20_id = snip.code_id

        shade_protocol['assets'][snip.symbol]['snip20'] = {
            'address': snip.address,
            'code_hash': snip.code_hash,
        }

        print(f'Registering {snip.symbol} with treasury')
        treasury.register_asset(snip)

        print('Set Viewing Key')
        snip.set_view_key(account_key, viewing_key)
        print(snip.symbol, snip.address)
        core_snips.append(snip)
        tokens[symbol] = snip

    print('Core Mints')
    # (snip, mint)
    core_pairs = []
    for snip in core_snips:
        print('Configuring', snip.symbol, 'Mint')
        mint = MicroMint(gen_label(8), snip, oracle, treasury,
                        # initial_assets=initial_assets,
                        admin=account_key, uploader=account_key, backend=backend,
                        code_id=mint_id)

        if not mint_id:
            mint_id = mint.code_id

        shade_protocol['assets'][snip.symbol]['mint'] = {
            'address': mint.address,
            'code_hash': mint.code_hash,
        }
        print(mint.address)

        print(f'Linking Snip/Mint {snip.symbol}')
        snip.set_minters([mint.address])

        for burn_symbol in BURN_MAP[snip.symbol]:
            capture = CAPTURE.get(burn_symbol)
            print(f'Registering {burn_symbol} with {snip.symbol}' + f' {capture} captured' if capture else '')
            mint.register_asset(tokens[burn_symbol], capture)

        core_pairs.append((snip, mint))

    print('\nSynthetics')

    synthetic_snips = []
    # (snip, mint)
    synthetics = []

    for name, symbol in synthetic_assets:
        print('\nConfiguring', name, symbol)
        snip = SNIP20(gen_label(8), name=name, symbol=symbol, 
                            decimals=6, public_total_supply=True, 
                            enable_mint=True, enable_burn=True,
                            admin=account_key, uploader=account_key, backend=backend,
                            code_id=snip20_id)

        shade_protocol['assets'][snip.symbol]['snip20'] = {
            'address': snip.address,
            'code_hash': snip.code_hash,
        }

        print(f'Registering {snip.symbol} with treasury')
        treasury.register_asset(snip)
        print('Set Viewing Key')
        snip.set_view_key(account_key, viewing_key)
        print(snip.symbol, snip.address)

        synthetic_snips.append(snip)
        tokens[symbol] = snip

    print('Synthetic Mints')

    for snip in synthetic_snips:

        print('Configuring', snip.symbol, 'Mint')
        mint = MicroMint(gen_label(8), snip, oracle, treasury,
                        # initial_assets=initial_assets,
                        admin=account_key, uploader=account_key, backend=backend,
                        code_id=mint_id)

        shade_protocol['assets'][snip.symbol]['mint'] = {
            'address': mint.address,
            'code_hash': mint.code_hash,
        }
        print(mint.address)

        for burn_symbol in BURN_MAP[snip.symbol]:
            capture = CAPTURE.get(burn_symbol)
            print(f'Registering {burn_symbol} with {snip.symbol}' + f' {capture} capture' if capture else '')
            mint.register_asset(tokens[burn_symbol], CAPTURE.get(burn_symbol))

        print(f'Linking Snip/Mint {snip.symbol}')
        snip.set_minters([mint.address])

        synthetics.append((snip, mint))

    print(run_command(['secretcli', 'q', 'account', account]))

    if 'prime' in argv:

        sscrt_mint_amount = '100000000000000'
        print(f'\tDeposit  {sscrt_mint_amount} uSCRT')
        sscrt.deposit(account, sscrt_mint_amount + 'uscrt')
        sscrt_minted = sscrt.get_balance(account, viewing_key)
        print(f'\tReceived {sscrt_minted} usSCRT')
        assert sscrt_mint_amount == sscrt_minted, f'Minted {sscrt_minted}; expected {sscrt_mint_amount}'
        total_amount = 100000000000000
        minimum_amount = 1000
        send_amount = random.randint(1000, int(total_amount / len(core_pairs + synthetics)) - 1)

        # Initializing balances with sscrt
        print('\nEntry minting with SSCRT')
        for core, mint in core_pairs:
            print(f'Burning {send_amount} usSCRT for', core.symbol)

            mint_response = sscrt.send(account_key, mint.address, send_amount, {'minimum_expected_amount': '0'})
            if mint_response.get('output_error'):
                print(f'Mint error: {mint_response["output_error"]}')

        print('Wallet Balance')
        for snip in [sscrt] + core_snips + synthetic_snips:
            print('\t', snip.get_balance(account, viewing_key), snip.symbol)

        print('Treasury Balance')
        print('\t', treasury.get_balance(sscrt), sscrt.symbol)
        for snip in core_snips + synthetic_snips:
            print('\t', treasury.get_balance(snip), snip.symbol)

        print('\nBurning core assets for eachother')

        for core, _ in core_pairs:
            send_amount = int(
                    int(core.get_balance(account, viewing_key)) 
                    / 10)
            for c, mint in core_pairs:
                if c is core:
                    continue
                print(f'\nBurning {send_amount}', core.symbol, 'for', c.symbol)

                mint_response = core.send(account_key, mint.address, send_amount, {'minimum_expected_amount': '0'})
                # print(mint_response) 
                if mint_response.get('output_error'):
                    print(f'Mint error: {mint_response["output_error"]}')


        print('\nBurning core assets for Synthetics')
        #send_amount = int(send_amount / 3)
        for core, _ in core_pairs:
            send_amount = int(
                    int(core.get_balance(account, viewing_key)) 
                    / (len(synthetics) + 1))

            for synthetic, mint in synthetics:
                print(f'\nBurning {send_amount}', core.symbol, 'for', synthetic.symbol)

                mint_response = core.send(account_key, mint.address, send_amount, {'minimum_expected_amount': '0'})
                # print(mint_response) 
                if mint_response.get('output_error'):
                    print(f'Mint error: {mint_response["output_error"]}')

        print('\nBurning Synthetics amongst eachother')

        for snip in synthetic_snips:
            send_amount = int(
                    int(snip.get_balance(account, viewing_key)) 
                    / (len(synthetic_snips) + 1))

            for s, mint in synthetics:
                if s is snip:
                    continue
                print(f'\nBurning {send_amount}', synthetic.symbol, 'for', s.symbol)

                mint_response = synthetic.send(account_key, mint.address, send_amount, {'minimum_expected_amount': '0'})
                print(mint_response) 
                if mint_response.get('output_error'):
                    print(f'Mint error: {mint_response["output_error"]}')


    print('Wallet Balance')
    for snip in [sscrt] + core_snips + synthetic_snips:
        print('\t', snip.get_balance(account, viewing_key), snip.symbol)

    print('Treasury Balance')
    print('\t', treasury.get_balance(sscrt), sscrt.symbol)
    for snip in core_snips + synthetic_snips:
        print('\t', treasury.get_balance(snip), snip.symbol)

    print(json.dumps(shade_protocol, indent=2))
    suffix = 'local' if account_key == 'a' else 'testnet'

    print(len(GAS_METRICS), 'times gas was used')
    open(f'logs/gas_metrics_{suffix}.json', 'w+').write(json.dumps(GAS_METRICS, indent=2))

    shade_protocol['gas_wanted'] = sum(int(g['want']) for g in GAS_METRICS)
    shade_protocol['gas_used'] = sum(int(g['used']) for g in GAS_METRICS)


    return shade_protocol

'''
Reads in the shade_protocol.json and verifies each contract
'''
def verify(shade_protocol):

    print('ORACLE')
    print(shade_protocol['oracle'])

    print('Treasury')
    print(shade_protocol['treasury'])

    print('ASSETS')
    for asset in shade_protocol['assets']:
        print(asset)

'''
Primes networks with user funds
'''
def prime(shade_protocol):

    '''
    sscrt_mint_amount = '100000000000000'
    print(f'\tDeposit  {sscrt_mint_amount} uSCRT')
    sscrt.deposit(account, sscrt_mint_amount + 'uscrt')
    sscrt_minted = sscrt.get_balance(account, viewing_key)
    print(f'\tReceived {sscrt_minted} usSCRT')
    assert sscrt_mint_amount == sscrt_minted, f'Minted {sscrt_minted}; expected {sscrt_mint_amount}'
    total_amount = 100000000000000
    minimum_amount = 1000
    send_amount = random.randint(1000, int(total_amount / len(core_pairs + synthetics)) - 1)

    # Initializing balances with sscrt
    print('\nEntry minting with SSCRT')
    for core, mint in core_pairs:
        print(f'Burning {send_amount} usSCRT for', core.symbol)

        mint_response = sscrt.send(account_key, mint.address, send_amount, {'minimum_expected_amount': '0'})
        if mint_response.get('output_error'):
            print(f'Mint error: {mint_response["output_error"]}')

    print('Wallet Balance')
    for snip in core_snips + synthetic_snips:
        print('\t', snip.get_balance(account, viewing_key), snip.symbol)

    print('Treasury Balance')
    print('\t', treasury.get_balance(sscrt), sscrt.symbol)
    for snip in core_snips + synthetic_snips:
        print('\t', treasury.get_balance(snip), snip.symbol)

    print('\nBurning core assets for eachother')

    for core, _ in core_pairs:
        send_amount = int(
                int(core.get_balance(account, viewing_key)) 
                / 10)
        for c, mint in core_pairs:
            if c is core:
                continue
            print(f'\nBurning {send_amount}', core.symbol, 'for', c.symbol)

            mint_response = core.send(account_key, mint.address, send_amount, {'minimum_expected_amount': '0'})
            # print(mint_response) 
            if mint_response.get('output_error'):
                print(f'Mint error: {mint_response["output_error"]}')


    print('\nBurning core assets for Synthetics')
    #send_amount = int(send_amount / 3)
    for core, _ in core_pairs:
        send_amount = int(
                int(core.get_balance(account, viewing_key)) 
                / (len(synthetics) + 1))

        for synthetic, mint in synthetics:
            print(f'\nBurning {send_amount}', core.symbol, 'for', synthetic.symbol)

            mint_response = core.send(account_key, mint.address, send_amount, {'minimum_expected_amount': '0'})
            # print(mint_response) 
            if mint_response.get('output_error'):
                print(f'Mint error: {mint_response["output_error"]}')

    print('\nBurning Synthetics amongst eachother')

    for snip in synthetic_snips:
        send_amount = int(
                int(snip.get_balance(account, viewing_key)) 
                / (len(synthetic_snips) + 1))

        for s, mint in synthetics:
            if s is snip:
                continue
            print(f'\nBurning {send_amount}', synthetic.symbol, 'for', s.symbol)

            mint_response = synthetic.send(account_key, mint.address, send_amount, {'minimum_expected_amount': '0'})
            print(mint_response) 
            if mint_response.get('output_error'):
                print(f'Mint error: {mint_response["output_error"]}')


    print('Wallet Balance')
    for snip in core_snips + synthetic_snips:
        print('\t', snip.get_balance(account, viewing_key), snip.symbol)

    print('Treasury Balance')
    print('\t', treasury.get_balance(sscrt), sscrt.symbol)
    for snip in core_snips + synthetic_snips:
        print('\t', treasury.get_balance(snip), snip.symbol)

    print(json.dumps(shade_protocol, indent=2))
    suffix = 'local' if account_key == 'a' else 'testnet'

    print(len(GAS_METRICS), 'times gas was used')
    open(f'logs/gas_metrics_{suffix}.json', 'w+').write(json.dumps(GAS_METRICS, indent=2))

    shade_protocol['gas_wanted'] = sum(int(g['want']) for g in GAS_METRICS)
    shade_protocol['gas_used'] = sum(int(g['used']) for g in GAS_METRICS)


    return shade_protocol
    '''


'''
Query contracts for more detailed information such as
token_info, token_config & get_config
code_id
treasury balance, snip20 balances
'''
def enrich(shade_protocol):

    print('Enriching')
    print('Oracle')
    shade_protocol['oracle'].update(query(shade_protocol['oracle'], {'get_config': {}}))

    print('Treasury')
    shade_protocol['treasury'].update(query(shade_protocol['treasury'], {'get_config': {}}))

    for symbol, pair in shade_protocol['assets'].items():
        print(symbol)
        pair['snip20'].update(query(pair['snip20'], {'token_info': {}}))
        pair['snip20'].update(query(pair['snip20'], {'token_config': {}}))
        pair['mint'].update(query(pair['mint'], {'get_config': {}}))

    return shade_protocol

if __name__ == '__main__':

    shade_protocol = None

    for s in argv:

        if s == 'load':
            print('Loading')
            shade_protocol = json.loads(open(SHADE_FILE_NAME).read())

        elif s == 'deploy':
            print('Deploying')
            shade_protocol = deploy()

        elif s == 'verify':
            print('Verifying')
            verify(shade_protocol)

        elif s == 'enrich':
            print('Enriching')
            shade_protocol = enrich(shade_protocol)

        elif s == 'prime':
            print('Priming')
            shade_protocol = prime(shade_protocol)

        elif s == 'save':
            print('Saving')
            open(SHADE_FILE_NAME, 'w+').write(json.dumps(shade_protocol, indent=2))

    if shade_protocol:
        print(json.dumps(shade_protocol, indent=2))
    else:
        print('nothing to do')
