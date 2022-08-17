#!/usr/bin/env python3
import json
from time import sleep
from sys import argv, exit

from contractlib.contractlib import Contract, PreInstantiatedContract as PreContract
from contractlib.utils import gen_label
from contractlib.secretlib.secretlib import run_command, execute_contract, query_contract
from contractlib.snip20lib import SNIP20
from contractlib.oraclelib import Oracle
from contractlib.micro_mintlib import MicroMint


from time import sleep

viewing_key = 'SecureSecrets'

account_key = 'drpresident' #if chain_config['chain-id'] == 'holodeck-2' else 'a'
backend = 'test' #None if chain_config['chain-id'] == 'holodeck-2' else 'test'
account = run_command(['secretd', 'keys', 'show', '-a', account_key]).rstrip()

print('ACCOUNT', account)

if not argv[1:]:
    print('Missing arg: treasury_addr')
    print('deploy-scrt-staking.py <treasury_addr>')
    exit(1)

treasury = argv[1]

print('TREASURY', treasury)

'''
print('Configuring sSCRT')
sscrt = SNIP20(gen_label(8), 
            name='secretSCRT', symbol='SSCRT', 
            decimals=6, public_total_supply=True, 
            enable_deposit=True, enable_burn=True,
            enable_redeem=True, admin=account, 
            uploader=account, backend=backend)
'''
# Pulsar 2
sscrt = {
    'address': 'secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg',
    'code_hash': '0x9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813',
}
#sscrt = PreContract(sscrt['address'], sscrt['code_hash'], sscrt['code_id'])

print('Configuring sSCRT Staking')
scrt_staking = Contract(
    '../compiled/scrt_staking.wasm.gz',
    json.dumps({
        'treasury': treasury,
        'sscrt': sscrt,
        'viewing_key': viewing_key,
    }),
    gen_label(8),
    admin=account_key, 
    uploader=account_key, 
)
alloc = 0.9

alloc = str(int(alloc * (10**18))),
print(f'Allocating {alloc} sSCRT to staking')

execute_contract(treasury,
    {
        'register_allocation': {
            'asset': sscrt['address'],
            'allocation': {
                'staking': {
                    'contract': {
                        'address': scrt_staking.address, 
                        'code_hash': scrt_staking.code_hash,
                    },
                    'allocation': alloc,
                },
            }
        }
    }
)

contracts = {
    'scrt_staking': scrt_staking.address,
}

print(json.dumps(contracts, indent=4))
