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

deposit_amount = '200000000' 

print('Depositing', deposit_amount)
sscrt.execute({'deposit': {}}, account, deposit_amount + 'uscrt')
print('SSCRT', sscrt.get_balance(account, viewing_key))

scrt_staking = Contract(
    '../compiled/scrt_staking.wasm.gz',
    json.dumps({
        'admin': account,
        'treasury': account,
        'sscrt': {
            'address': sscrt.address,
            'code_hash': sscrt.code_hash,
        },
        'viewing_key': viewing_key,
    }),
    gen_label(8),
)
print('STAKING', scrt_staking.address)

print('Sending 100000000 usscrt direct to staking')
print(sscrt.execute({
        "send": {
            "recipient": scrt_staking.address,
            "amount": str(100000000),
        },
    },
    account,
))


print('DELEGATIONS')
delegations = scrt_staking.query({'delegations': {}})
print(delegations)

print('SSCRT', sscrt.get_balance(account, viewing_key))
print('scrt staking L1 bal')
print(json.loads(run_command(['secretd', 'q', 'bank', 'balances', scrt_staking.address])))

print('Waiting on rewards', end='')
while scrt_staking.query({'adapter': {'rewards': {}}}) == '0':
    print('.', end='')
    pass
    
print()
print('REWARDS', scrt_staking.query({'adapter': {'rewards': {}}}))
print('scrt staking L1 bal')
print(json.loads(run_command(['secretd', 'q', 'bank', 'balances', scrt_staking.address])))

print('CLAIMING')
print(scrt_staking.execute({'adapter': {'claim': {}}}))

print('scrt staking L1 bal')
print(json.loads(run_command(['secretd', 'q', 'bank', 'balances', scrt_staking.address])))
print('SSCRT', sscrt.get_balance(account, viewing_key))
print('REWARDS', scrt_staking.query({'adapter': {'rewards': {}}}))

print('scrt staking L1 bal')
print(json.loads(run_command(['secretd', 'q', 'bank', 'balances', scrt_staking.address])))

print()
print('UNBONDING', deposit_amount)
print(scrt_staking.execute({'adapter': {'unbond': {'amount': deposit_amount}}}))

print('DELEGATIONS')
delegations = scrt_staking.query({'delegations': {}})
print(delegations)

print('scrt staking L1 bal')
print(json.loads(run_command(['secretd', 'q', 'bank', 'balances', scrt_staking.address])))

'''
print('Waiting on rewards', end='')
while scrt_staking.query({'adapter': {'rewards': {}}}) == '0':
    print('.', end='')
    pass
'''

print('Waiting a few sec for unbond')
sleep(5)
    
print()
print('REWARDS', scrt_staking.query({'adapter': {'rewards': {}}}))

print('CLAIMING')
print(scrt_staking.execute({'adapter': {'claim': {}}}))

print('scrt staking L1 bal')
print(json.loads(run_command(['secretd', 'q', 'bank', 'balances', scrt_staking.address])))
print('SSCRT', sscrt.get_balance(account, viewing_key))
print('REWARDS', scrt_staking.query({'adapter': {'rewards': {}}}))

