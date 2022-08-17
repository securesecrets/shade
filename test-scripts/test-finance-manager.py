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

deposit_amount = '1000000000' 

print('Depositing', deposit_amount)
sscrt.execute({'deposit': {}}, ACCOUNT, deposit_amount + 'uscrt')
print('SSCRT', sscrt.get_balance(ACCOUNT, viewing_key))

print('Deploying Finance Manager')
finance_manager = Contract(
    '../compiled/finance_manager.wasm.gz',
    json.dumps({
        'admin': ACCOUNT,
        'treasury': ACCOUNT,
        'viewing_key': viewing_key,
    }),
    gen_label(8),
)
print('Manager', finance_manager.address)

num_adapters = 2
allocs = [0.1, 0.9]

print(f'Deploying {num_adapters} SCRT Staking Adapters')
adapters = [
    Contract(
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
    for i in range(num_adapters)
]

print('ADAPTERS', '\n'.join([s.address for s in adapters]))

print('Registering sscrt w/ manager')
print(finance_manager.execute({
        'register_asset': {
            'contract': sscrt.as_dict(),
        }
    },
    ACCOUNT,
))

i = 0
for adapter, alloc in zip(adapters, allocs):

    print(f'Registering {i} w/ {alloc * 100}% allocation')
    print(finance_manager.execute({
            'allocate': {
                'asset': sscrt.address,
                'allocation': {
                    'nick': f'SCRT Staking {i}',
                    'contract': adapter.as_dict(),
                    'alloc_type': 'portion',
                    'amount': str(int(alloc * 10**18)),
                }
            }
        },
        ACCOUNT,
    ))
    i += 1

print('Allocations')
print(json.dumps(finance_manager.query({
    'allocations': {
        'asset': sscrt.address,
    }
}), indent=2))

while True:

    allow_amount = 1
    print(f'{allow_amount} sSCRT allowance to manager')
    sscrt.execute({
            'increase_allowance': {
                'spender': finance_manager.address,
                'owner': ACCOUNT,
                'amount': str(int(allow_amount * 10**6))
            }
        },
        ACCOUNT,
    )

    print('Pending Allowance')
    print(finance_manager.query({
        'pending_allowance': {
            'asset': sscrt.address
        }
    }))

    for i in range(1):

        print('Rebalance')
        print(finance_manager.execute({
            'manager': {
                'update': {
                    'asset': sscrt.address,
                }
            }
        }, ACCOUNT))

        print('My Balance')
        print(sscrt.get_balance(ACCOUNT, viewing_key))

        print('\nMANAGER')
        print('Pending Allowance')
        print(finance_manager.query({
            'pending_allowance': {
                'asset': sscrt.address
            }
        }))

        print('Balance')
        print(finance_manager.query({
            'manager': {
                'balance': {
                    'asset': sscrt.address,
                }
            }
        }))
        
        print('Unbonding')
        print(finance_manager.query({
            'manager': {
                'unbonding': {
                    'asset': sscrt.address,
                }
            }
        }))

        print('Claimable')
        print(finance_manager.query({
            'manager': {
                'claimable': {
                    'asset': sscrt.address,
                }
            }
        }))

        for i, adapter in enumerate(adapters):
            print('\nADAPTER', i)
            print('Updating')
            adapter.execute({'adapter': {'update': {}}})
            print('Balance')
            print(adapter.query({
                'adapter': {
                    'balance': {
                        'asset': sscrt.address,
                    }
                }
            }))

            print('Unbonding')
            print(adapter.query({
                'adapter': {
                    'unbonding': {
                        'asset': sscrt.address,
                    }
                }
            }))

            print('Claimable')
            print(adapter.query({
                'adapter': {
                    'claimable': {
                        'asset': sscrt.address,
                    }
                }
            }))

        print('-' * 20, end='\n\n')

        '''
        new_alloc = .01
        print('Reducing Allocation on 0 to', new_alloc)
        print(finance_manager.execute({
                'allocate': {
                    'asset': sscrt.address,
                    'allocation': {
                        'nick': f'SCRT Staking 0',
                        'contract': adapters[0].as_dict(),
                        'alloc_type': 'portion',
                        'amount': str(int(new_alloc * 10**18)),
                    }
                }
            },
            ACCOUNT,
        ))
        print('-' * 20, end='\n\n')
        '''
