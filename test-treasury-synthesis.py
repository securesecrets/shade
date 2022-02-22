#!/usr/bin/env python3
import json
from time import sleep
from contractlib.contractlib import Contract
from contractlib.utils import gen_label
from contractlib.secretlib.secretlib import run_command, execute_contract, query_contract
from contractlib.snip20lib import SNIP20

'''
chain_config = run_command(['secretd', 'config'])

chain_config = {
    key.strip('" '): val.strip('" ')
    for key, val in 
    (
        line.split('=') 
        for line in chain_config.split('\n')
        if line
    )
}
'''

viewing_key = 'password'


account_key = 'a' #if chain_config['chain-id'] == 'holodeck-2' else 'a'
backend = 'test' #None if chain_config['chain-id'] == 'holodeck-2' else 'test'
account = run_command(['secretd', 'keys', 'show', '-a', account_key]).rstrip()


print('ACCOUNT', account)

print('Configuring sSCRT')
sscrt = SNIP20(gen_label(8), 
            name='secretSCRT', symbol='SSCRT', 
            decimals=6, public_total_supply=True, 
            enable_deposit=True, enable_burn=True,
            enable_redeem=True, admin=account, 
            uploader=account, backend=backend)
print(sscrt.address)
sscrt.execute({'set_viewing_key': {'key': viewing_key}})

deposit_amount = '200000000uscrt' 
# lol
half_amount = '100000000uscrt' 

print('Depositing', deposit_amount)
sscrt.execute({'deposit': {}}, account, deposit_amount)
print('SSCRT', sscrt.get_balance(account, viewing_key))

treasury = Contract(
    '../compiled/treasury.wasm.gz',
    json.dumps({
        'admin': account,
        'viewing_key': viewing_key,
    }),
    gen_label(8),
)
print('TREASURY', treasury.address)

staking_init = {
    'admin': account,
    'treasury': treasury.address,
    'sscrt': {
        'address': sscrt.address,
        'code_hash': sscrt.code_hash,
    },
    'viewing_key': viewing_key,
}

print('Registering sSCRT w/ treasury')
print(treasury.execute({
    'register_asset': {
        'contract': {
            'address': sscrt.address, 
            'code_hash': sscrt.code_hash,
        }
    }
}))

scrt_staking = Contract(
    '../compiled/scrt_staking.wasm.gz',
    json.dumps(staking_init),
    gen_label(8),
)
print('STAKING', scrt_staking.address)

print('Allocating 90% sSCRT to staking')
allocation = .9
print(treasury.execute({
    'register_allocation': {
        'asset': sscrt.address,
        'allocation': {
            'staking': {
                'contract': {
                    'address': scrt_staking.address, 
                    'code_hash': scrt_staking.code_hash,
                },
                'allocation': str(int(allocation * 10**18)),
            },
        }
    }
}))


print('Treasury Assets')
print(treasury.query({'assets': {}}))

print('Treasury sSCRT Balance')
print(treasury.query({'balance': {'asset': sscrt.address}}))

print('Treasury sSCRT Applications')
print(treasury.query({'allocations': {'asset': sscrt.address}}))

print('Sending 100000000 usscrt to treasury')
sscrt.execute({
        "send": {
            "recipient": treasury.address,
            "amount": str(100000000),
        },
    },
    account,
)
print('Treasury sSCRT Balance')
print(treasury.query({'balance': {'asset': sscrt.address}}))

print('DELEGATIONS')
delegations = scrt_staking.query({'delegations': {}})
print(delegations)

print('Waiting for rewards',)
while scrt_staking.query({'rewards': {}}) == '0':
    print('.',)
print()
    
print('REWARDS', scrt_staking.query({'rewards': {}}))

print('CLAIMING')
for delegation in delegations:
    print(scrt_staking.execute({'claim': {'validator': delegation['validator']}}))

print('Treasury sSCRT Balance')
print(treasury.query({'balance': {'asset': sscrt.address}}))
