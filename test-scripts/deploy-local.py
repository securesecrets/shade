#!/usr/bin/env python3
import json
from time import sleep
from contractlib.contractlib import Contract, PreInstantiatedContract
from contractlib.utils import gen_label
from contractlib.secretlib.secretlib import run_command, execute_contract, query_contract
from contractlib.snip20lib import SNIP20
from contractlib.oraclelib import Oracle
from contractlib.mintlib import Mint

from time import sleep

viewing_key = 'SecureSecrets'

ACCOUNT_KEY = 'a'
backend = 'test'
ACCOUNT = run_command(['secretd', 'keys', 'show', '-a', ACCOUNT_KEY]).rstrip()

treasury = PreInstantiatedContract(
        ACCOUNT,
        '',
        1,
)

print('ACCOUNT', ACCOUNT)

print('Configuring sSCRT')
sscrt = SNIP20(gen_label(8), 
            name='secretSCRT', symbol='SSCRT', 
            decimals=6, public_total_supply=True, 
            enable_deposit=True, enable_burn=True,
            enable_redeem=True, admin=ACCOUNT, 
            uploader=ACCOUNT, backend=backend)
print('Setting viewing key')
sscrt.execute({'set_viewing_key': {'key': viewing_key}})

deposit_amount = '200000000uscrt' 
# lol
half_amount = '100000000uscrt' 

print('Depositing', deposit_amount)
sscrt.execute({'deposit': {}}, ACCOUNT, deposit_amount)
print('SSCRT', sscrt.get_balance(ACCOUNT, viewing_key))

print('Configuring SHD')
shade = SNIP20(
        gen_label(8),
        name='Shade',
        symbol='SHD',
        decimals=8,
        public_total_supply=True, 
        enable_mint=True, 
        enable_burn=True, 
        admin=ACCOUNT_KEY, 
        uploader=ACCOUNT_KEY, 
        backend=backend,
        #initial_balances=[
        #    {
        #        'address': ACCOUNT,
        #        'amount': '1000000000000000',
        #    },
        #]
)
print('Setting viewing key')
shade.execute({'set_viewing_key': {'key': viewing_key}})

print('Configuring SILK')
silk = SNIP20(
        gen_label(8),
        name='Silk',
        symbol='SILK',
        decimals=6,
        public_total_supply=True, 
        enable_mint=True, 
        enable_burn=True, 
        admin=ACCOUNT_KEY, 
        uploader=ACCOUNT_KEY, 
        backend=backend,
)
print('Setting viewing key')
silk.execute({'set_viewing_key': {'key': viewing_key}})

mock_band = Contract(
    '../compiled/mock_band.wasm.gz',
    json.dumps({}),
    gen_label(8),
    admin=ACCOUNT_KEY, 
    uploader=ACCOUNT_KEY, 
)

print('Setting price feeds')
print('SHD $15')
mock_band.execute({'mock_price': {'symbol': 'SHD', 'price': str(int(15 * 10**18))}})
print('SILK $1.2')
mock_band.execute({'mock_price': {'symbol': 'SILK', 'price': str(int(1.2 * 10**18))}})
print('SCRT $2.5')
mock_band.execute({'mock_price': {'symbol': 'SCRT', 'price': str(int(2.5 * 10**18))}})

print('Configuring Oracle')
oracle = Oracle(
    gen_label(8),
    band_contract=mock_band,
    sscrt=sscrt,
    admin=ACCOUNT_KEY, 
    uploader=ACCOUNT_KEY, 
)

'''
# Set to ACCOUNT for fee collection
treasury = PreContract(ACCOUNT, code_hash='', code_id=1)
print('Configuring Treasury')
treasury = Contract(
    '../compiled/treasury.wasm.gz',
    json.dumps({
        'admin': ACCOUNT,
        'viewing_key': viewing_key,
    }),
    gen_label(8),
    admin=ACCOUNT_KEY, 
    uploader=ACCOUNT_KEY, 
)
print('Registering sSCRT')
treasury.execute(
    {
        'register_asset': {
            'contract': {
                'address': sscrt.address,
                'code_hash': sscrt.code_hash,
            },
            'reserves': str(int(.2 * 10**18)),
        }
    }
)
print('Registering SHD')
treasury.execute(
    {
        'register_asset': {
            'contract': {
                'address': shade.address,
                'code_hash': shade.code_hash,
            },
            'reserves': str(int(.2 * 10**18)),
        }
    }
)

print('Registering SILK')
treasury.execute(
    {
        'register_asset': {
            'contract': {
                'address': silk.address,
                'code_hash': silk.code_hash,
            },
            'reserves': str(int(.2 * 10**18)),
        }
    }
)

print('Taking a quick break...')
sleep(5)
'''

print('Configuring SHD minting')
shade_mint = Mint(
    gen_label(8),
    native_asset=shade,
    oracle=oracle,
    treasury=treasury,
    admin=ACCOUNT_KEY, 
    uploader=ACCOUNT_KEY, 
)
print('Registering as SHD Minter')
shade.execute({'set_minters': {'minters': [shade_mint.address]}}, sender=ACCOUNT)

print('Registering sSCRT %100 capture')
shade_mint.execute({'register_asset': {'contract': sscrt.as_dict(), 'capture': str(int(1 * 10 ** 18))}}, sender=ACCOUNT)

print('Registering SILK no capture')
shade_mint.execute({'register_asset': {'contract': silk.as_dict()}}, sender=ACCOUNT)

print('Configuring SILK minting')
silk_mint = Mint(
    gen_label(8),
    native_asset=silk,
    oracle=oracle,
    treasury=treasury,
    admin=ACCOUNT_KEY, 
    uploader=ACCOUNT_KEY, 
)
print('Registering as SILK Minter')
silk.execute({'set_minters': {'minters': [silk_mint.address]}}, sender=ACCOUNT)

# Dont want to register sscrt, no entry mint to silk
#print('Registering sSCRT %100 capture')
#silk_mint.execute({'register_asset': {'contract': sscrt.as_dict(), 'capture': str(int(1 * 10**18))}}, sender=ACCOUNT)

print('Registering SHD no capture')
silk_mint.execute({'register_asset': {'contract': shade.as_dict()}}, sender=ACCOUNT)

mint_router = Contract(
    '../compiled/mint_router.wasm.gz',
    json.dumps({
        'path': [
            shade_mint.as_dict(),
            silk_mint.as_dict(),
        ]
    }),
    gen_label(8),
    admin=ACCOUNT_KEY,
    uploader=ACCOUNT_KEY,
)

'''
print('Configuring sSCRT Staking')
scrt_staking = Contract(
    '../compiled/scrt_staking.wasm.gz',
    json.dumps({
        'treasury': treasury.address,
        'sscrt': {
            'address': sscrt.address,
            'code_hash': sscrt.code_hash,
        },
        'viewing_key': viewing_key,
    }),
    gen_label(8),
    admin=ACCOUNT_KEY, 
    uploader=ACCOUNT_KEY, 
)

allocation = .9

print(f'Allocating {allocation * 100}% sSCRT to staking')
treasury.execute({
    'register_allocation': {
        'asset': sscrt.address,
        'allocation': {
            'staking': {
                'contract': {
                    'address': scrt_staking.address, 
                    'code_hash': scrt_staking.code_hash,
                },
                'allocation': str(int(allocation * (10**18))),
            },
        }
    }
})
'''

contracts = {
    # snip-20s
    'sscrt': sscrt.address,
    'shade': shade.address,
    'silk': silk.address,

    # mints
    'shade_mint': shade_mint.address,
    'silk_mint': silk_mint.address,

    'oracle': oracle.address,
    'band': mock_band.address,

    #'treasury': treasury.address,
    #'scrt_staking': scrt_staking.address,
    #'airdrop': airdrop.address,
}

print(json.dumps(contracts, indent=4))
open('contracts.json', 'w+').write(json.dumps(contracts, indent=4))

print(sscrt.get_balance(ACCOUNT, viewing_key), 'sSCRT')

print('Minting SHD with sSCRT')
print(json.dumps(sscrt.execute({
        "send": {
            "recipient": shade_mint.address,
            "amount": str(10 * 10**6),
        },
    },
    ACCOUNT,
), indent=2))

print(shade.get_balance(ACCOUNT, viewing_key), 'SHD')
print(silk.get_balance(ACCOUNT, viewing_key), 'SILK')

amount = str(10 * 10**6)
print('Routing 10 sSCRT -> SILK -- ({amount} usscrt)')

print('Route')
print(json.dumps(mint_router.query({
        "route": {
            "asset": sscrt.address,
            "amount": amount,
        },
    },
), indent=2))

print(json.dumps(sscrt.execute({
        "send": {
            "recipient": mint_router.address,
            "amount": amount,
        },
    },
    ACCOUNT,
), indent=2))

print('User')
print(silk.get_balance(ACCOUNT, viewing_key), 'SILK')
print(shade.get_balance(ACCOUNT, viewing_key), 'SHD')
print(sscrt.get_balance(ACCOUNT, viewing_key), 'sSCRT')
