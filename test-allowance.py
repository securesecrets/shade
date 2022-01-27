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

print('Configuring treasury')
print(treasury.execute({
    'register_asset': {
        'contract': {
            'address': sscrt.address, 
            'code_hash': sscrt.code_hash,
        }
    }
}))

print(treasury.execute({
    'register_allocation': {
        'asset': sscrt.address,
        'allocation': {
            'allowance': {
                'address': account,
                'amount': '1000000', # (uscrt) = 1 SCRT
            },
        }
    }
}))

print('Last refresh')
print(treasury.query({'last_allowance_refresh': {}}))

print('Refreshing allowance')
print(treasury.execute({'refresh_allowance': {}}))

print('Last refresh')
print(treasury.query({'last_allowance_refresh': {}}))

print('Treasury sSCRT Balance')
print(treasury.query({'balance': {'asset': sscrt.address}}))

print('Refreshing allowance (should fail/do nothing)')
print(treasury.execute({'refresh_allowance': {}}))

print('sSCRT Allowance')
print(sscrt.query({
    'allowance': {
        'owner': treasury.address,
        'spender': account,
        'key': viewing_key,
    }
}))
