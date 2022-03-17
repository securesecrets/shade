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
print('sSCRT', sscrt.address, sscrt.code_hash)
sscrt.execute({'set_viewing_key': {'key': viewing_key}})

deposit_amount = '1000000000uscrt' 
# lol
# half_amount = '100000000uscrt' 

print('Depositing', deposit_amount)
sscrt.execute({'deposit': {}}, account, deposit_amount)
print('Wallet SSCRT', sscrt.get_balance(account, viewing_key))

treasury = Contract(
    '../compiled/treasury.wasm.gz',
    json.dumps({
        'admin': account,
        'viewing_key': viewing_key,
        'sscrt': sscrt.as_dict(),
    }),
    gen_label(8),
)
print('TREASURY', treasury.address)

print('Registering sSCRT w/ treasury')
print(treasury.execute({
    'register_asset': {
        'contract': sscrt.as_dict(),
    }
}))

print('Deploying SCRT Staking')
scrt_staking = Contract(
    '../compiled/scrt_staking.wasm.gz',
    json.dumps({
        'admin': account,
        'treasury': treasury.address,
        'sscrt': sscrt.as_dict(),
        'viewing_key': viewing_key,
    }),
    gen_label(8),
)
print(scrt_staking.address)

allocation = .01
print(f'Allocating {allocation * 100}% sSCRT to staking')
print(treasury.execute({
    'register_allocation': {
        'asset': sscrt.address,
        'allocation': {
            'single_asset': {
                'contract': scrt_staking.as_dict(),
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
print(sscrt.execute({
        "send": {
            "recipient": treasury.address,
            "amount": str(100000000),
        },
    },
    account,
))
print('Treasury sSCRT Balance')
treasury_balance = treasury.query({'balance': {'asset': sscrt.address}})
print(treasury_balance)
print('Wallet SSCRT', sscrt.get_balance(account, viewing_key))
print('scrt_staking sscrt balance')
print(sscrt.query({'balance': {'address': scrt_staking.address, 'key': viewing_key}}))
print('scrt_staking L1 balance')
print(run_command(['secretd', 'q', 'bank', 'balances', scrt_staking.address]))

print('DELEGATIONS')
delegations = scrt_staking.query({'delegations': {}})
print(delegations)

if treasury_balance == '0':
    print('No treasury balance!')

print('Waiting for rewards', end='')
while scrt_staking.query({'adapter': {'rewards': {}}}) == '0':
    print('.', end='', flush=True)
print()
    
print('REWARDS', scrt_staking.query({
    'adapter': {
        'rewards': {}
    }}))

print('CLAIMING')
for delegation in delegations:
    print(scrt_staking.execute({'adapter': {'claim': {}}}))

print('Treasury sSCRT Balance')
print(treasury.query({'balance': {'asset': sscrt.address}}))
