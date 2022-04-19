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
print('sSCRT', sscrt.address, sscrt.code_hash)
sscrt.execute({'set_viewing_key': {'key': viewing_key}})

deposit_amount = '1000000000uscrt' 
# lol
# half_amount = '100000000uscrt' 

print('Depositing', deposit_amount)
sscrt.execute({'deposit': {}}, ACCOUNT, deposit_amount)
print('Wallet SSCRT', sscrt.get_balance(ACCOUNT, viewing_key))

print('Deploying Treasury')
treasury = Contract(
    '../compiled/treasury.wasm.gz',
    json.dumps({
        'admin': ACCOUNT,
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

print('Deploying Finance Manager')
finance_manager = Contract(
    '../compiled/finance_manager.wasm.gz',
    json.dumps({
        'admin': ACCOUNT,
        'treasury': treasury.address,
        'viewing_key': viewing_key,
    }),
    gen_label(8),
)
print('Manager', finance_manager.address)

print('Registering sscrt w/ manager')
print(finance_manager.execute({
        'register_asset': {
            'contract': sscrt.as_dict(),
        }
    },
    ACCOUNT,
))

print(f'Registering Manager with Treasury')
print(treasury.execute({
    'register_manager': {
        'contract': finance_manager.as_dict(),
    }
}))

allowance = .9
print(f'Register Finance Manager allowance {allowance * 100}%')
print(treasury.execute({
    'allowance': {
        'asset': sscrt.address,
        'allowance': {
            'portion': {
                'spender': finance_manager.address,
                'portion': str(int(allowance * 10**18)),
                'last_refresh': '',
            }
        }
    }
}))

print('Deploying SCRT Staking')
scrt_staking = Contract(
    '../compiled/scrt_staking.wasm.gz',
    json.dumps({
        'admin': ACCOUNT,
        'treasury': treasury.address,
        'sscrt': sscrt.as_dict(),
        'viewing_key': viewing_key,
    }),
    gen_label(8),
)
print(scrt_staking.address)

allocation = 1

print(f'Allocating {allocation * 100}% sSCRT to scrt-staking')
print(finance_manager.execute({
    'allocate': {
        'asset': sscrt.address,
        'allocation': {
            'nick': 'SCRT Staking',
            'contract': scrt_staking.as_dict(),
            'alloc_type': 'portion',
            'amount': str(int(allocation * 10**18)),
        }
    }
}))

print('Treasury Assets')
print(treasury.query({'assets': {}}))

print('Treasury sSCRT Balance')
print(treasury.query({'balance': {'asset': sscrt.address}}))

print('Sending 100000000 usscrt to treasury')
print(sscrt.execute({
        "send": {
            "recipient": treasury.address,
            "amount": str(100000000),
        },
    },
    ACCOUNT,
))


while True:

    print('\nTreasury')
    print('Rebalance')
    print(treasury.execute({
        'rebalance': {
            'asset': sscrt.address
        },
    }))
    print('Balance')
    print(treasury.query({
        'balance': {
            'asset': sscrt.address
        },
    }))

    print('\nFinance Manager')
    print('Pending Allowance')
    print(finance_manager.query({
        'pending_allowance': {
            'asset': sscrt.address
        }
    }))

    print('Update')
    print(finance_manager.execute({
        'adapter': {
            'update': {
                'asset': sscrt.address,
            }
        }
    }, ACCOUNT))

    print('Pending Allowance')
    print(finance_manager.query({
        'pending_allowance': {
            'asset': sscrt.address
        }
    }))

    print('Balance')
    print(finance_manager.query({
        'adapter': {
            'balance': {
                'asset': sscrt.address,
            }
        }
    }))
    
    print('Unbonding')
    print(finance_manager.query({
        'adapter': {
            'unbonding': {
                'asset': sscrt.address,
            }
        }
    }))

    print('Claimable')
    claimable = finance_manager.query({
        'adapter': {
            'claimable': {
                'asset': sscrt.address,
            }
        }
    })
    print(claimable)
    if claimable['claimable']['amount'] != '0': 
        print('Claiming...')
        print(finance_manager.execute({
            'adapter': {
                'claim': {'asset': sscrt.address}
            }
        }))

    print('\nSCRT Staking')
    print('Updating')
    scrt_staking.execute({'adapter': {'update': {}}})

    print('Balance')
    print(scrt_staking.query({
        'adapter': {
            'balance': {
                'asset': sscrt.address,
            }
        }
    }))

    print('Unbonding')
    print(scrt_staking.query({
        'adapter': {
            'unbonding': {
                'asset': sscrt.address,
            }
        }
    }))

    print('Claimable')
    claimable = scrt_staking.query({
        'adapter': {
            'claimable': {
                'asset': sscrt.address,
            }
        }
    })
    print(claimable)
    if claimable['claimable']['amount'] != '0': 
        print('Claiming...')
        print(scrt_staking.execute({
            'adapter': {
                'claim': {'asset': sscrt.address}
            }
        }))

    print('-' * 20, end='\n\n')

