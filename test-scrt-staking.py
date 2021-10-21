#!/usr/bin/env python3
import json
from contractlib.contractlib import Contract
from contractlib.utils import gen_label
from contractlib.secretlib.secretlib import run_command, execute_contract, query_contract
from contractlib.snip20lib import SNIP20

chain_config = run_command(['secretcli', 'config'])

chain_config = {
    key.strip('" '): val.strip('" ')
    for key, val in 
    (
        line.split('=') 
        for line in chain_config.split('\n')
        if line
    )
}

viewing_key = 'password'

account_key = 'drpresident' if chain_config['chain-id'] == 'holodeck-2' else 'a'
backend = None if chain_config['chain-id'] == 'holodeck-2' else 'test'
account = run_command(['secretcli', 'keys', 'show', '-a', account_key]).rstrip()

print('ACCOUNT', account)

print('Configuring sSCRT')
sscrt = SNIP20(gen_label(8), 
        name='secretSCRT', symbol='SSCRT', 
        decimals=6, public_total_supply=True, 
        enable_deposit=True, enable_burn=True,
        enable_redeem=True, admin=account_key, 
        uploader=account_key, backend=backend)
print(sscrt.address)
sscrt.execute({'set_viewing_key': {'key': viewing_key}})

deposit_amount = '100000000uscrt' 
print('Depositing', deposit_amount)
sscrt.execute({'deposit': {}}, account, deposit_amount)

init = {
    'admin': account,
    'treasury': account,
    'sscrt': {
        'address': sscrt.address,
        'code_hash': sscrt.code_hash,
    },
    'viewing_key': viewing_key,
}

scrt_staking = Contract(
    '../compiled/scrt_staking.wasm.gz',
    json.dumps(init),
    gen_label(8),
)

print('get_config')
print(scrt_staking.query({'get_config': {}}))
print('delegations')
print(scrt_staking.query({'delegations': {}}))
print('Sending 100000000 usscrt for staking')
print(sscrt.execute({
        "send": {
            "recipient": scrt_staking.address,
            "amount": str(100000000),
        },
    },
    account,
))


print('sSCRT')
print(sscrt.query({'balance': {'address': scrt_staking.address, 'key': viewing_key}}))

print('SCRT')
print(run_command(['secretcli', 'q', 'account', scrt_staking.address]))

print('delegations')
for delegation in scrt_staking.query({'delegations': {}}):
    print(scrt_staking.query({'delegation': {'validator': delegation['validator']}}))
