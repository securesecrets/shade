#!/usr/bin/python3

from contractlib.secretlib.secretlib import query_contract, run_command
from sys import exit
from collections import defaultdict

import json
import yaml

# I think the max is 30
PAGE_SIZE = 30

FUTURE = ['emeris', 'junoswap', 'terraswap', 'terrastation', 'uniswap']

DEX = {
    'secretswap': {
        'price': lambda sym: None,
        'pools': lambda : None,
        'pool_info': lambda : None,
        'token_info': lambda : None,
    }, 
    'sienna': {
        'price': lambda sym: None,
        'pools': lambda : None,
        'pool_info': lambda : None,
        'token_info': lambda : None,
    },
    'osmosis': {
        'price': lambda sym: None,
        'pools': lambda : None,
        'pool_info': lambda : None,
        'token_info': lambda : None,
    },
}

'''
# TODO implement in later token/pair comparison
COLISSION_SETS = [
    { 'SCRT' },
    { 'OSMO' },
    { 'JUNO' },
    { 'ETH' },
    { 'BTC' },
]

# generate collisions for 
# 'SCRT' <-> 'uscrt' 
# 'SCRT' <-> 'SSCRT'
# 'SCRT' <-> 'WSCRT'
for col in COLISSION_SETS:
    col.update(f'u{c.lower()}' for c in col)
    col.update(f'S{c}' for c in col)
'''


'''
{
    'SHD/sSCRT': {
        'secretswap': {
            'SHD': { token_info },
            'sSCRT': { },
        },
    }
}
'''
all_data = {}

sienna_factory = 'secret18sq0ux28kt2z7dlze2mu57d3ua0u5ayzwp6v2r'
secretswap_factory = 'secret1fjqlk09wp7yflxx7y433mkeskqdtw3yqerkcgp'

expected_size = 0
sienna_pairs = {}
while len(sienna_pairs) == expected_size:

    expected_size += PAGE_SIZE

    print(f'Getting Sienna pairs {len(sienna_pairs)}-{expected_size}...', flush=True)
    sienna_pairs.update({
        p['contract']['address']: (
            p['pair']['token_0']['custom_token']['contract_addr'],
            p['pair']['token_1']['custom_token']['contract_addr'],
        )
        for p in query_contract(
                sienna_factory,
                {'list_exchanges': {
                    'pagination': {'start': len(sienna_pairs), 'limit': PAGE_SIZE}
                }}
            )['list_exchanges']['exchanges']
    })

print('Found', len(sienna_pairs), 'Sienna pairs')

print('Writing sienna pairs to sienna_pairs.json')
with open('sienna_pairs.json', 'w+') as f:
    f.write(json.dumps(sienna_pairs))

expected_size = 0
last = None
secretswap_pairs = {}

while len(secretswap_pairs) == expected_size:

    expected_size += PAGE_SIZE

    print(f'Getting SecretSwap pairs {len(secretswap_pairs)}-{expected_size}...', flush=True)
    print(json.dumps({'pairs': {'start_after': last, "limit": PAGE_SIZE }}))

    pairs = query_contract(secretswap_factory, {'pairs': {'start_after': last, "limit": PAGE_SIZE }})['pairs']
    print(len(pairs), 'found')
    # TODO: this pagination isn't working at all, second run gives 0
    # last = pairs[-1]['asset_infos']
    secretswap_pairs.update({
        p['contract_addr']: (
            p['asset_infos'][0]['token']['contract_addr'],
            p['asset_infos'][1]['token']['contract_addr'],
        )
        for p in pairs
    })

print('Found', len(secretswap_pairs), 'SecretSwap pairs')

print('Writing secretswap pairs to secretswap_pairs.json')
with open('secretswap_pairs.json', 'w+') as f:
    f.write(json.dumps(secretswap_pairs))
'''

osmosis_pairs = {}

print('Gathering Osmosis Pairs...')
pools = yaml.safe_load(run_command(['osmosisd', 'query', 'gamm', 'pools']))
for pool in pools['pools']:
    print(json.dumps(pool, indent=2))
    pool_assets = yaml.safe_load(run_command(['osmosisd', 'query', 'gamm', 'pool-assets', pool['id']]))
    osmosis_pairs.update({
        pool['address']: (
            pool_assets['poolAssets'][0]['denom'],
            pool_assets['poolAssets'][1]['denom'],
        ),
    })

print('Found', len(osmosis_pairs), 'Osmosis pairs')

print('Writing osmosis pairs to osmosis_pairs.json')
with open('osmosis_pairs.json', 'w+') as f:
    f.write(json.dumps(osmosis_pairs))
'''

# TODO: table here for ibc denoms on osmosis https://docs.osmosis.zone/developing/assets/asset-info.html

# { address : token_info }
token_infos = {}
# { 'SHD/SILK': [ sienna_pair, sswap_pair, .. ]}
pairs = defaultdict(list)

print('Gathering token info...', flush=True)
for pair_addr, tokens in list(sienna_pairs.items()) + list(secretswap_pairs.items()):
    for addr in tokens:
        if addr not in token_infos.keys():
            token_infos[addr] = query_contract(addr, {
                'token_info': {}
            })['token_info']
            print('Token:', token_infos[addr]['symbol'])

    pair_symbol = '/'.join([
        token_infos[addr]['symbol'] for addr in tokens
    ])
    pairs[pair_symbol].append(pair_addr)
    print('Pair:', pair_symbol)

print('Writing all pairs to pairs.json')
with open('pairs.json', 'w+') as f:
    f.write(json.dumps(pairs))

print('Writing token info to token_info.json')
with open('token_info.json', 'w+') as f:
    f.write(json.dumps(token_infos))

tokens = sorted(
    info['symbol']
    for addr, info in token_infos.items()
)

matrix = {
    sym0: {
        sym1: 0 
        for sym1 in tokens
    }
    for sym0 in tokens 
}

for pair_sym in pairs:
    sym0, sym1 = pair_sym.split('/')
    matrix[sym0][sym1] = 1
    matrix[sym1][sym0] = 1

'''
print('|' + '\t|'.join(tokens))
print('|', end='')
print('\n   |'.join(('\t|'.join(map(str,matrix[m].values())) for m in matrix)))
'''

cycles = set()

def dfs(source, visited=None, start=None, path=[]):

    visited = visited or {t: False for t in tokens}
    start = start or source
    path.append(source)
    visited[source] = True

    for cur in tokens:
        # Cycle detection
        if matrix[source][cur] and visited[cur]:

            all_indices = [i for i, v in enumerate(path) if v == cur]

            if all_indices:
                inter_path = path[all_indices[-1]:]
            else:
                inter_path = path

            if len(inter_path) > 2:
                cycles.add(tuple(inter_path + [cur]))

        # Recurse DFS
        elif matrix[source][cur] and not visited[cur]:
            visited = dfs(cur, visited, start, path)

    return visited

for t in tokens:
    dfs(t)

cycles = sorted(cycles, key=len)

print('\nCycles')
for cycle in cycles:
    line = ','.join(cycle)
    print(line)

print(f'\nWriting {len(cycles)} Cycles to cycles.json')
with open('cycles.json', 'w+') as f:
    f.write(json.dumps(cycles))
