#!/usr/bin/env python3
import json
from contractlib.secretlib.secretlib import query_contract, run_command
from contractlib.contractlib import Contract, PreInstantiatedContract
from contractlib.oraclelib import Oracle
from contractlib.snip20lib import SNIP20
from contractlib.utils import gen_label

'''
sscrt = PreInstantiatedContract(
        'secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg', 
        '9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813',
        0,
)
'''

band_prices = {
    # symbol: price
    'USD': 1,
    'SCRT': 4.8,
    # 'SHD': 1,
}

index_basket = {
    # symbol: weight
    'USD': .2,
    'SCRT': .5,
    'SHD': .1,
}

ACCOUNT_KEY = 'a'
ACCOUNT = run_command(['secretd', 'keys', 'show', '-a', ACCOUNT_KEY]).rstrip()

sscrt = SNIP20(gen_label(8), name='secretSCRT', symbol='SSCRT', 
               decimals=6, public_total_supply=True, 
               enable_deposit=True, enable_burn=True, 
               initial_balances=[
                   {
                       'address': ACCOUNT, 
                       'amount': '1000000000',
                   },
               ],
               admin=ACCOUNT_KEY, uploader=ACCOUNT_KEY, 
               backend='test')

print('sSCRT', sscrt.address)

shade = SNIP20(gen_label(8), name='Shade', symbol='SHD', 
               decimals=8, public_total_supply=True, 
               enable_deposit=False, enable_burn=True, 
               initial_balances=[
                   {
                       'address': ACCOUNT, 
                       'amount': '100000000000',
                   },
               ],
               admin=ACCOUNT_KEY, uploader=ACCOUNT_KEY, 
               backend='test')

print('SHD', shade.address)

print('Mocking sswap Pair')
sswap_pair = Contract('mock_secretswap_pair.wasm.gz', '{}', gen_label(8))
print(sswap_pair.address)

print(sswap_pair.execute({
    'mock_pool': {
        'token_a': shade.as_dict(),
        'amount_a': '606443379564',
        'token_b': sscrt.as_dict(),
        'amount_b': '69598078371',
    }
}))

'''
print(sswap_pair.query({'pool': {}}))
print(sswap_pair.query({'pair': {}}))
'''

print('Mocking sienna pair')
sienna_pair = Contract('mock_sienna_pair.wasm.gz', '{}', gen_label(8))
print(sienna_pair.address)

print(sienna_pair.execute({
    'mock_pool': {
        'token_a': shade.as_dict(),
        'amount_a': '811474458501',
        'token_b': sscrt.as_dict(),
        'amount_b': '93585484077',
    }
}))

'''
print(sienna_pair.query('pair_info'))
'''

mock_band = Contract('mock_band.wasm.gz', '{}', gen_label(8))
print('mock band', mock_band.address)

print('Mocking BAND Prices')

# normalize band prices
band_txs = [
    {'mock_price': {'symbol': s, 'price': str(int(p * 10**18))}}
    for s, p in band_prices.items()
]

for tx in band_txs:
    print(tx['mock_price']['symbol'], '\t', tx['mock_price']['price'])
    mock_band.execute(tx)

oracle = Oracle(gen_label(8), mock_band, sscrt, admin=ACCOUNT_KEY, uploader=ACCOUNT_KEY, backend=None)

print('oracle', oracle.address)

print('Registering pools for SHD')

print('Registering sswap pool')
print(oracle.execute({
    'register_pair': {
        'pair': sswap_pair.as_dict()
    }
}))

print('price:', oracle.query({'price': {'symbol': 'SHD'}}))

print('Registering sienna pool')
print(oracle.execute({
    'register_pair': {
        'pair': sienna_pair.as_dict()
    }
}))
print('price:', oracle.query({'price': {'symbol': 'SHD'}}))

# normalize index basket
index_basket = [
    {'symbol': s, 'weight': str(int(w * 10**18))}
    for s, w in index_basket.items()
]

print(json.dumps(index_basket, indent=2))

print('Registering SILK Index', index_basket)
print(oracle.execute({
    'register_index': {
        'symbol': 'SILK', 
        'basket': index_basket
    }
}))

symbols = ['USD', 'SCRT', 'SHD', 'SILK']
print('Querying', symbols)

for symbol in symbols:
    print(symbol, int(oracle.query({'price': {'symbol': symbol}})['rate']) / 10**18)
