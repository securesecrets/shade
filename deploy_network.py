#!/usr/bin/env python3 
import random
import click
import json
from sys import exit
from collections import defaultdict


from contractlib.contractlib import PreInstantiatedContract
from contractlib.contractlib import Contract
from contractlib.secretlib import secretlib
from contractlib.snip20lib import SNIP20
from contractlib.micro_mintlib import MicroMint
from contractlib.oraclelib import Oracle
from contractlib.treasurylib import Treasury
from contractlib.utils import gen_label

@click.command()
@click.option('--no_prime', is_flag=True)


def deploy_network(no_prime: bool):
    if no_prime:
        print('Deploying without priming')
    else:
        print('Will deploy then prime the network with funds, pass --no_prime to skip')

    # Burns for core_assets assets
    entry_assets = [
        ('secretSCRT', 'SSCRT'),
    ]

    core_assets = [
        # Core
        ('Shade', 'SHD'),
        ('Silk', 'SILK'),
        # Synthetic assets stablecoin
        #('Synthesis', 'SYN'),
    ]

    # burn core assets for these, and back
    synthetic_assets = [
        # Metals
        ('Synthetic Gold', 'XAU'),
        ('Synthetic Silver', 'XAG'),

        # Fiat
        #('Synthetic Euro', 'EUR'),
        #('Synthetic Japanese yen', 'JPY'),
        #('Synthetic Yuan', 'CNY'),

        # Crypto
        #('Synthetic Ethereum', 'ETH'),
        #('Synthetic Bitcoin', 'BTC'),
        #('Synthetic secret', 'SCRT'),
        #('Synthetic Dogecoin', 'DOGE'),
        #('Synthetic Uniswap', 'UNI'),
        #('Synthetic Stellar', 'XLM'),
        #('Synthetic PancakeSwap', 'CAKE'),
        #('Synthetic Band Protocol', 'BAND'),
        #('Synthetic Terra', 'LUNA'),
        #('Synthetic Cosmos', 'ATOM'),

        # TODO: Oracle: add these sources
        # Stocks 
        # ('Synthetic Tesla', 'TSLA'),
        # ('Synthetic Apple', 'AAPL'),
        # ('Synthetic Google', 'GOOG'),
    ]

    TESTNET_BAND = {
        "address": "secret1p0jtg47hhwuwgp4cjpc46m7qq6vyjhdsvy2nph",
        "code_hash": "77c854ea110315d5103a42b88d3e7b296ca245d8b095e668c69997b265a75ac5"
    }

    TESTNET_SSCRT = {
        'address': 'secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx',
        'code_hash': 'cd400fb73f5c99edbc6aab22c2593332b8c9f2ea806bf9b42e3a523f3ad06f62'
    }
    shade_network = defaultdict(dict)

    # 1%
    commission = int(.01 * 10000)
    # viewing_key = '4b734d7a2e71fb277a9e3355f5c56d347f1012e1a9533eb7fdbb3ceceedad5fc'
    viewing_key = 'passsword'

    chain_config = secretlib.run_command(['secretcli', 'config'])

    chain_config = {
        key.strip('" '): val.strip('" ')
        for key, val in 
        (
            line.split('=') 
            for line in chain_config.split('\n')
            if line
        )
    }

    if chain_config['chain-id'] == 'holodeck-2':
        account_key = 'drpresident'
        backend = None
        print('Setting testnet SSCRT')
        sscrt = PreInstantiatedContract(TESTNET_SSCRT['address'], TESTNET_SSCRT['code_hash'])
        sscrt.symbol = 'SSCRT'
        print(sscrt.address)

        print('Setting testnet band')
        band = PreInstantiatedContract(TESTNET_BAND['address'], TESTNET_BAND['code_hash'])
        print(TESTNET_BAND['address'])

    elif chain_config['chain-id'] == 'enigma-pub-testnet-3':
        account_key = 'a'
        backend = 'test'
        print('Configuring SSCRT')
        sscrt = SNIP20(gen_label(8), name='secretSCRT', symbol='SSCRT', decimals=6, public_total_supply=True, enable_deposit=True, enable_burn=True,
                    admin=account_key, uploader=account_key, backend=backend)
        print(sscrt.address)
        sscrt.set_view_key(account_key, viewing_key)

        print('Mocking Band for local chain')
        band = Contract('mock_band.wasm.gz', '{}', gen_label(8))
        print(band.address)

    else:
        print('Failed to determine chain', chain_config['chain-id'])
        exit(1)



    account = secretlib.run_command(['secretcli', 'keys', 'show', '-a', account_key]).rstrip()

    print(json.loads(secretlib.run_command(['secretcli', 'q', 'account', account]))['value']['coins'])

    print('Configuring Oracle')
    oracle = Oracle(gen_label(8), band, 
                    admin=account_key, uploader=account_key, backend=backend)
    shade_network['oracle'] = {
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
    shade_network['treasury'] = {
        'address': treasury.address,
        'code_hash': treasury.code_hash,
    }

    print('Registering SSCRT with treasury')
    print(treasury.register_asset(sscrt))

    snip20_id = None
    mint_id = None

    shade_network['assets'] = {}
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

        shade_network['assets'][snip.symbol]['snip20'] = {
            'address': snip.address,
            'code_hash': snip.code_hash,
        }

        print(f'Registering {snip.symbol} with treasury')
        treasury.register_asset(snip)

        print('Set Viewing Key')
        snip.set_view_key(account_key, viewing_key)
        print(snip.symbol, snip.address)
        core_snips.append(snip)

    print('\nSynthetic Snips')

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
        shade_network['assets'][snip.symbol]['snip20'] = {
            'address': snip.address,
            'code_hash': snip.code_hash,
        }
        print(f'Registering {snip.symbol} with treasury')
        treasury.register_asset(snip)
        print('Set Viewing Key')
        snip.set_view_key(account_key, viewing_key)
        print(snip.symbol, snip.address)

        synthetic_snips.append(snip)

    print('Core Mints')
    # (snip, mint)
    core_pairs = []
    for snip in core_snips:
        initial_assets = [
                c
                for c in core_snips
                if c is not snip 
            ] + synthetic_snips + [ sscrt ]
        print('Configuring', snip.symbol, 'Mint')
        mint = MicroMint(gen_label(8), snip, oracle, treasury, commission,
                        # initial_assets=initial_assets,
                        admin=account_key, uploader=account_key, backend=backend,
                        code_id=mint_id)
        if not mint_id:
            mint_id = mint.code_id

        shade_network['assets'][snip.symbol]['mint'] = {
            'address': mint.address,
            'code_hash': mint.code_hash,
        }
        print(mint.address)

        print(f'Linking Snip/Mint {snip.symbol}')
        snip.set_minters([mint.address])

        for i in initial_assets:
            print(f'Registering {i.symbol} with {snip.symbol}')
            mint.register_asset(i)

        core_pairs.append((snip, mint))

    print('Synthetic Mints')
    for snip in synthetic_snips:
        initial_assets = [
                s
                for s in synthetic_snips
                if s is not snip
            ] + core_snips
        print('Configuring', snip.symbol, 'Mint')
        mint = MicroMint(gen_label(8), snip, oracle, treasury, commission,
                        # initial_assets=initial_assets,
                        admin=account_key, uploader=account_key, backend=backend,
                        code_id=mint_id)

        shade_network['assets'][snip.symbol]['mint'] = {
            'address': mint.address,
            'code_hash': mint.code_hash,
        }
        print(mint.address)

        for i in initial_assets:
            print(f'Registering {i.symbol} with {snip.symbol}')
            mint.register_asset(i)

        print(f'Linking Snip/Mint {snip.symbol}')
        snip.set_minters([mint.address])

        synthetics.append((snip, mint))

    print(secretlib.run_command(['secretcli', 'q', 'account', account]))

    if not no_prime:

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

            mint_response = sscrt.send(account_key, mint.address, send_amount)
            # print(mint_response) 
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

                mint_response = core.send(account_key, mint.address, send_amount)
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

                mint_response = core.send(account_key, mint.address, send_amount)
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

                mint_response = synthetic.send(account_key, mint.address, send_amount)
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

    print(json.dumps(shade_network, indent=2))
    suffix = 'local' if account_key == 'a' else 'testnet'

    print(len(secretlib.GAS_METRICS), 'times gas was used')
    open(f'logs/gas_metrics_{suffix}.json', 'w+').write(json.dumps(secretlib.GAS_METRICS, indent=2))

    shade_network['gas_wanted'] = sum(int(g['want']) for g in secretlib.GAS_METRICS)
    shade_network['gas_used'] = sum(int(g['used']) for g in secretlib.GAS_METRICS)
    open(f'logs/shade_{suffix}.json', 'w+').write(json.dumps(shade_network, indent=2))
    return shade_network

if __name__ == '__main__':
    shade_network = deploy_network()

