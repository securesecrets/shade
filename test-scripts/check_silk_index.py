#!/usr/bin/env python3

from contractlib.secretlib.secretlib import query_contract, run_command
from contractlib.contractlib import Contract, PreInstantiatedContract

'''
band = PreInstantiatedContract(
        'secret1ulxxh6erkmk4p6cjehz58cqspw3qjuedrsxp8f',
        'dc6ff596e1cd83b84a6ffbd857576d7693d89a826471d58e16349015e412a3d3',
        0,
)
'''

band = PreInstantiatedContract(
        'secret14swdnnllsfvtnvwmtvnvcj2zu0njsl9cdkk5xp',
        '00230665fa8dc8bb3706567cf0a61f282edc34d2f7df56192b2891fd9cd27b06',
        0,
)


index = {
    'USD': 39.32,
    'CNY': 7.13,
    'EUR': 15.97,
    'JPY': 7.64,
    'GBP': 3.40,
    'CAD': 4.58,
    'KRW': 1.53,
    'AUD': 2.32,
    'IDR': 2.50,
    'CHF': 4.44,
    'SEK': 0.84,
    'NOK': 0.82,
    'SGD': 2.50,
    'XAU': 5.0,
    'WBTC': 2.00,

    'BTC': 2.00,
    'ETH': 0.00,
    'SCRT': 0.00,
}

good = []
bad = []

for sym, weight in index.items():
    try: 
        band.query({'get_reference_data': {'quote_symbol': sym, 'base_symbol': 'USD'}})
        good.append(sym)
    except:
        bad.append(sym)

print('AVAIL', good)
print('MISSING', bad)

