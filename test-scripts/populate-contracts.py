#!/usr/bin/env python3
import json
from collections import defaultdict
from contractlib.secretlib import secretlib

contracts = json.loads(open('contracts.json').read())
pop_contracts = defaultdict(dict)

for contract, addr in contracts.items():
    pop_contracts[contract]['address'] = addr
    pop_contracts[contract]['code_hash'] = secretlib.query_contract_hash(addr)

print(json.dumps(pop_contracts, indent=4))
open('pop_contracts.json', 'w+').write(json.dumps(pop_contracts, indent=4))
