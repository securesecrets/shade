#!/usr/bin/env python3
import json
from contractlib.secretlib.secretlib import query_contract
from contractlib.contractlib import Contract
from contractlib.oraclelib import Oracle
from contractlib.snip20lib import SNIP20
from contractlib.utils import gen_label

sscrt = SNIP20(gen_label(8), name='secretSCRT', symbol='SSCRT', 
               decimals=6, public_total_supply=True, 
               enable_deposit=True, enable_burn=True, 
               admin='a', uploader='a', 
               backend='test')

mock_band = Contract('mock_band.wasm.gz', '{}', gen_label(8))
print('mock band')
print(mock_band.address)

oracle = Oracle(gen_label(8), mock_band, sscrt, admin='a', uploader='a', backend=None)

print('oracle')
print(oracle.address)

band_prices = {
    # symbol: price
    'USD': 1,
    'SCRT': 7.5,
}

index_basket = {
    # symbol: weight
    'USD': .2,
    'SCRT': .5,
}

# normalize band prices
band_prices = [
    {'mock_price': {'symbol': s, 'price': str(int(p * 10**18))}}
    for s, p in band_prices.items()
]
# normalize index basket
index_basket = [
    {'symbol': s, 'weight': str(int(w * 10**18))}
    for s, w in index_basket.items()
]
print(json.dumps(index_basket, indent=2))

for b in band_prices:
    print('mocking', b)
    print(mock_band.execute(b))

print(oracle.execute({'register_index': {'symbol': 'SILK', 'basket': index_basket}}))

print('\n'.join(oracle.query({'prices': {'symbols': ['USD', 'SCRT']}})))

usd = int(oracle.query({'price': {'symbol': 'USD'}})['rate']) / 10**18
scrt = int(oracle.query({'price': {'symbol': 'SCRT'}})['rate']) / 10**18
silk = int(oracle.query({'price': {'symbol': 'SILK'}})['rate']) / 10**18

print('USD:', usd)
print('SCRT:', scrt)
print('SILK:', silk)
