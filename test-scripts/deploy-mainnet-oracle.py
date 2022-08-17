#!/usr/bin/env python3
import json
from contractlib.secretlib.secretlib import query_contract, run_command
from contractlib.contractlib import Contract, PreInstantiatedContract
from contractlib.oraclelib import Oracle
from contractlib.snip20lib import SNIP20
from contractlib.utils import gen_label

ACCOUNT_KEY = 'drpresident'
ACCOUNT = run_command(['secretd', 'keys', 'show', '-a', ACCOUNT_KEY]).rstrip()

sscrt = PreInstantiatedContract(
        'secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek', 
        'af74387e276be8874f07bec3a87023ee49b0e7ebe08178c49d0a49c3c98ed60e',
        0,
)

band = PreInstantiatedContract(
        'secret1hlmdes5at42xs833dqs0gkxe7j0h4y7z26d7dy', 
        '72a2a86c2648aae1dbce96a373b261c29ab8a8da1cdfe07561d4a516dacd008d',
        0,
)

sswap_pair = PreInstantiatedContract(
        'secret1wwt7nh3zyzessk8c5d98lpfsw79vzpsnerj6d0', 
        '0dfd06c7c3c482c14d36ba9826b83d164003f2b0bb302f222db72361e0927490',
        0,
)
sienna_pair = PreInstantiatedContract(
        'secret1drm0dwvewjyy0rhrrw485q4f5dnfm6j25zgfe5', 
        '33eac42c44ee69acfe1f56ce7b14fe009a7b611e86f275d7af2d32dd0d33d5a9',
        0,
)
print('Deploying Oracle')
oracle = Oracle(gen_label(8), band, sscrt, admin=ACCOUNT_KEY, uploader=ACCOUNT_KEY)

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

symbols = ['USD', 'SCRT', 'SHD']
print('Querying', symbols)

for symbol in symbols:
    print(symbol, int(oracle.query({'price': {'symbol': symbol}})['rate']) / 10**18)
