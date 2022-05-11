#!/usr/bin/env python3
import json
from time import sleep
from contractlib.contractlib import Contract
from contractlib.utils import gen_label
from contractlib.secretlib.secretlib import run_command, execute_contract, query_contract
from contractlib.snip20lib import SNIP20

viewing_key = 'password'

ACCOUNT_KEY = 'a' #if chain_config['chain-id'] == 'holodeck-2' else 'a'
backend = 'test' #None if chain_config['chain-id'] == 'holodeck-2' else 'test'
ACCOUNT = run_command(['secretd', 'keys', 'show', '-a', ACCOUNT_KEY]).rstrip()

print('ACCOUNT', ACCOUNT)

print('Configuring sSCRT')
sscrt = SNIP20(gen_label(8), 
            name='secretSCRT', symbol='SSCRT', 
            decimals=6, public_total_supply=True, 
            enable_deposit=True, enable_burn=True,
            enable_redeem=True, admin=ACCOUNT, 
            uploader=ACCOUNT, backend=backend)
print(sscrt.address)
sscrt.execute({'set_viewing_key': {'key': viewing_key}})

# 200
#deposit_amount = '200000000' 
# 10
deposit_amount = '10000000' 

print('Depositing', deposit_amount)
sscrt.execute({'deposit': {}}, ACCOUNT, deposit_amount + 'uscrt')
print('SSCRT', sscrt.get_balance(ACCOUNT, viewing_key))

scrt_staking = Contract(
    '../compiled/scrt_staking.wasm.gz',
    json.dumps({
        'admin': ACCOUNT,
        'treasury': ACCOUNT,
        'sscrt': {
            'address': sscrt.address,
            'code_hash': sscrt.code_hash,
        },
        'viewing_key': viewing_key,
    }),
    gen_label(8),
)
print('STAKING', scrt_staking.address)

print(f'Sending {deposit_amount} usscrt direct to staking')
print(sscrt.execute({
        "send": {
            "recipient": scrt_staking.address,
            "amount": deposit_amount,
        },
    },
    ACCOUNT,
))

while True:

    #print('user sSCRT', sscrt.get_balance(ACCOUNT, viewing_key))

    print('DELEGATIONS')
    delegations = scrt_staking.query({'delegations': {}})
    print(delegations)

    print('L1 bal')
    print(json.loads(run_command(['secretd', 'q', 'bank', 'balances', scrt_staking.address])))

    print('Balance')
    balance = scrt_staking.query({'adapter': {'balance': {'asset': sscrt.address}}})['balance']['amount']
    print(balance)

    #unbond_amount = str(int(10 * 10**6))
    unbond_amount = str(int(int(balance) * .8))

    print('Unbond', unbond_amount)
    print(scrt_staking.execute({'adapter': {'unbond': {'asset': sscrt.address, 'amount': unbond_amount}}}))

    print('Unbonding')
    print(scrt_staking.query({'adapter': {'unbonding': {'asset': sscrt.address}}}))

    print('Balance')
    balance = scrt_staking.query({'adapter': {'balance': {'asset': sscrt.address}}})['balance']['amount']
    print(balance)

    print('Updating')
    print(scrt_staking.execute({'adapter': {'update': {}}}))

    print('Claimable')
    print(scrt_staking.query({'adapter': {'claimable': {'asset': sscrt.address}}}))

    print('Claiming')
    print(scrt_staking.execute({'adapter': {'claim': {'asset': sscrt.address}}}))

    '''
    print('Waiting on claimable', end='')
    while scrt_staking.query({'adapter': {'claimable': {'asset': sscrt.address}}})['amount'] == '0':
        print('.', end='')
        pass
    '''
    print()
    print('=' * 15)
    print()
