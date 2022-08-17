#!/usr/bin/python3

from contractlib.secretlib.secretlib import query_contract
from sys import exit
from collections import defaultdict
import json

PAGE_LIMIT = 100

sienna_factory = 'secret18sq0ux28kt2z7dlze2mu57d3ua0u5ayzwp6v2r'
secretswap_factory = 'secret1fjqlk09wp7yflxx7y433mkeskqdtw3yqerkcgp'

print('Getting Sienna pairs...', end='', flush=True)
sienna_pairs = {
    p['contract']['address']: (
        { 'address': p['pair']['token_0']['custom_token']['contract_addr'] },
        { 'address': p['pair']['token_1']['custom_token']['contract_addr'] },
    )
    for p in query_contract(
            sienna_factory,
            {'list_exchanges': {
                'pagination': {'start': 0, 'limit': PAGE_LIMIT}
            }}
        )['list_exchanges']['exchanges']
}
print(len(sienna_pairs))

print('Writing sienna pairs to sienna_pairs.json')
with open('sienna_pairs.json', 'w+') as f:
    f.write(json.dumps(sienna_pairs))

if len(sienna_pairs) >= PAGE_LIMIT:
    print('Page limit', PAGE_LIMIT, 'exceeded')
    exit(1)

print('Getting SecretSwap pairs...', end='', flush=True)
secretswap_pairs = {
    p['contract_addr']: (
        { 'address': p['asset_infos'][0]['token']['contract_addr'] },
        { 'address': p['asset_infos'][1]['token']['contract_addr'] },
    )
    for p in query_contract(
            secretswap_factory, 
            {'pairs': {}}
        )['pairs']
}
print(len(secretswap_pairs))

print('Writing secretswap pairs to secretswap_pairs.json')
with open('secretswap_pairs.json', 'w+') as f:
    f.write(json.dumps(secretswap_pairs))

# { address : token_info }
token_infos = {}
# { 'SHD/SILK': [ sienna_pair, sswap_pair, .. ]}
pairs = defaultdict(list)

print('Gathering token info...', flush=True)
for addr, tokens in list(sienna_pairs.items()) + list(secretswap_pairs.items()):
    for t in tokens:
        if t['address'] not in token_infos.keys():
            token_infos[t['address']] = query_contract(t['address'], {
                'token_info': {}
            })['token_info']
            print('Token:', token_infos[t['address']]['symbol'])

    pair_symbol = '/'.join([token_infos[t['address']]['symbol'] for t in tokens])
    pairs[pair_symbol].append(addr)
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

print(len(cycles), 'Cycles')
for cycle in cycles:
    line = ','.join(cycle)
    print(line)

print()
print('Writing Cycles to cycles.json')
with open('cycles.json', 'w+') as f:
    f.write(json.dumps(cycles))
