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

seed_amount = 100000000000

print('Depositing', seed_amount)
sscrt.execute({'deposit': {}}, ACCOUNT, str(seed_amount) + 'uscrt')

tolerance = .05
print(f'Deploying Treasury w/ tolerance {tolerance * 100}%')
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

print('Registering Account w/ treasury')
print(treasury.execute({
    'add_account': {
        'holder': ACCOUNT,
    }
}))

print('Registering sSCRT w/ treasury')
print(treasury.execute({
    'register_asset': {
        'contract': sscrt.as_dict(),
    }
}))

print('Deploying Finance Manager')
farming_manager = Contract(
    '../compiled/farming_manager.wasm.gz',
    json.dumps({
        'admin': ACCOUNT,
        'treasury': treasury.address,
        'viewing_key': viewing_key,
    }),
    gen_label(8),
)
print('Manager', farming_manager.address)

print('Registering sscrt w/ manager')
print(farming_manager.execute({
        'register_asset': {
            'contract': sscrt.as_dict(),
        }
    },
    ACCOUNT,
))

print(f'Registering Manager with Treasury')
print(treasury.execute({
    'register_manager': {
        'contract': farming_manager.as_dict(),
    }
}))

allowance = .9
print(f'Register Finance Manager allowance {allowance * 100}%')
print(treasury.execute({
    'allowance': {
        'asset': sscrt.address,
        'allowance': {
            'portion': {
                'spender': farming_manager.address,
                'portion': str(int(allowance * 10**18)),
                'last_refresh': '',
                'tolerance': str(int(tolerance * 10**18)),
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
print(farming_manager.execute({
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

print(f'Sending {seed_amount} usscrt to treasury')
print(sscrt.execute({
        "send": {
            "recipient": treasury.address,
            "amount": str(seed_amount),
        },
    },
    ACCOUNT,
))


while True:

    print('\nTreasury')
    print('Balance')
    treasury_balance = treasury.query({
        'balance': {
            'asset': sscrt.address
        },
    })['balance']['amount']
    print(treasury_balance)

    print('\nFinance Manager')

    print('Balance')
    farming_balance = farming_manager.query({
        'adapter': {
            'balance': {
                'asset': sscrt.address,
            }
        }
    })['balance']['amount']
    print(farming_balance)

    outstanding = sum(map(int, [farming_balance]))
    reserves = int(treasury_balance) - outstanding

    print('ALLOCS')
    print('Finance', int(farming_balance) / int(treasury_balance))
    print('Reserves', int(reserves) / int(treasury_balance))
    
    print('Rebalance...')
    print(treasury.execute({
        'rebalance': {
            'asset': sscrt.address
        },
    }))
    print(farming_manager.query({
        'pending_allowance': {
            'asset': sscrt.address
        }
    }))

    print('Unbonding')
    unbonding = farming_manager.query({
        'adapter': {
            'unbonding': {
                'asset': sscrt.address,
            }
        }
    })['unbonding']['amount']
    print(unbonding)

    print('Updating...')
    farming_manager.execute({
        'adapter': {
            'update': {
                'asset': sscrt.address,
            }
        }
    }, ACCOUNT)
    scrt_staking.execute({
        'adapter': {
            'update': {
                'asset': sscrt.address,
            }
        }
    }, ACCOUNT)

    print(farming_manager.query({
        'pending_allowance': {
            'asset': sscrt.address
        }
    }))

    print(farming_manager.query({
        'adapter': {
            'unbonding': {
                'asset': sscrt.address,
            }
        }
    }))

    claimable = farming_manager.query({
        'adapter': {
            'claimable': {
                'asset': sscrt.address,
            }
        }
    })
    print(claimable)
    if claimable['claimable']['amount'] != '0': 
        print('Claiming...')
        farming_manager.execute({
            'adapter': {
                'claim': {'asset': sscrt.address}
            }
        })


    print('=' * 20, end='\n')

