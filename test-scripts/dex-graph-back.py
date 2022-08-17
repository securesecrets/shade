#!/usr/bin/python3

from contractlib.secretlib.secretlib import run_command_compute_hash

SIENNA_FACTORY = 'secret18sq0ux28kt2z7dlze2mu57d3ua0u5ayzwp6v2r'

tokens = [
    'SHD',
    'SCRT',
    'ETH',
    'BTC',
    'XAU',
    'OSMO',
    'ATOM',
    'JUNO',
    'ABC',
    'XYZ',
]

# token_list = [i for i in range(10)]
pair_list = [
    (0, 1),
    (1, 2),
    (2, 3),
    (2, 4),
    (4, 8),
    (5, 4),
    (0, 5),
    (0, 4),
    (0, 6),
    (6, 7),
    (7, 0),
]

matrix = [
    [0 for i in range(len(tokens))]
    for i in tokens 
]

for pairs in pair_list:
    matrix[pairs[0]][pairs[1]] = 1
    matrix[pairs[1]][pairs[0]] = 1

print('\t|' + '\t|'.join(tokens))
print('\t|', end='')
print('\n\t|'.join(('\t|'.join(map(str,m)) for m in matrix)))

cycles = set()

def dfs(source, visited=None, start=None, path=[]):

    visited = visited or [False for i in tokens]
    start = start or source
    path.append(source)
    visited[source] = True

    for cur in range(len(tokens)):

        # Cycle detected
        if matrix[source][cur] and visited[cur]:

            all_indices = [i for i, v in enumerate(path) if v == cur]

            if all_indices:
                inter_path = path[all_indices[-1]:]
            else:
                inter_path = path

            if len(path) > 1:
                cycles.add(tuple(inter_path + [cur]))

        # Continue to DFS at new node
        elif matrix[source][cur] and not visited[cur]:
            visited = dfs(cur, visited, start, path)

    return visited

for i in range(len(tokens)):
    dfs(i)

print('Cycles')
for cycle in sorted(cycles, key=len):
    print(','.join([tokens[c] for c in cycle]))
