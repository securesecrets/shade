#!/usr/bin/env python3
import json
from time import sleep
from contractlib.contractlib import Contract, PreInstantiatedContract as PreContract
from contractlib.utils import gen_label
from contractlib.secretlib.secretlib import run_command, execute_contract, query_contract
from contractlib.snip20lib import SNIP20
from contractlib.oraclelib import Oracle
from contractlib.mintlib import Mint

from time import sleep

viewing_key = 'SecureSecrets'

account_key = 'drpresident' #if chain_config['chain-id'] == 'holodeck-2' else 'a'
backend = 'test' #None if chain_config['chain-id'] == 'holodeck-2' else 'test'
account = run_command(['secretd', 'keys', 'show', '-a', account_key]).rstrip()

print('ACCOUNT', account)

'''
print('Configuring sSCRT')
sscrt = SNIP20(gen_label(8), 
            name='secretSCRT', symbol='SSCRT', 
            decimals=6, public_total_supply=True, 
            enable_deposit=True, enable_burn=True,
            enable_redeem=True, admin=account, 
            uploader=account, backend=backend)
'''
# Pulsar 2
sscrt = {
    'address': 'secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg',
    'code_hash': '0x9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813',
    'code_id': 1,
}
sscrt = PreContract(sscrt['address'], sscrt['code_hash'], sscrt['code_id'])

print('Configuring SHD')
'''
shade = SNIP20(
        gen_label(8),
        name='Shade',
        symbol='SHD',
        decimals=8,
        public_total_supply=True, 
        enable_mint=True, 
        enable_burn=True, 
        admin=account_key, 
        uploader=account_key, 
        backend=backend,
        initial_balances=[
            {
                'address': account,
                'amount': '1000000000000000',
            },
        ]
)
'''

shade = {
    'address': 'secret19ymc8uq799zf36wjsgu4t0pk8euddxtx5fggn8',
    'code_hash': '5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1',
    'code_id': 1,
}
shade = PreContract(shade['address'], shade['code_hash'], shade['code_id'])

print('Configuring SILK')
'''
silk = SNIP20(
        gen_label(8),
        name='Silk',
        symbol='SILK',
        decimals=6,
        public_total_supply=True, 
        enable_mint=True, 
        enable_burn=True, 
        admin=account_key, 
        uploader=account_key, 
        backend=backend,
)
'''

silk = {
    'address': 'secret18k8a6lytr3gxppv96qus5qazg093gks7pk4q5x',
    'code_hash': '5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1',
    'code_id': 1,
}
silk = PreContract(silk['address'], silk['code_hash'], silk['code_id'])

print('Configuring Mock BAND')
'''
mock_band = Contract(
    '../compiled/mock_band.wasm.gz',
    json.dumps({}),
    gen_label(8),
    admin=account_key, 
    uploader=account_key, 
)
'''
band = {
    'address': 'secret1ulxxh6erkmk4p6cjehz58cqspw3qjuedrsxp8f',
    'code_hash': 'dc6ff596e1cd83b84a6ffbd857576d7693d89a826471d58e16349015e412a3d3',
    'code_id': 1,
}
band = PreContract(band['address'], band['code_hash'], band['code_id'])

'''
print('Setting price feeds')
print('SHD $15')
mock_band.execute({'mock_price': {'symbol': 'SHD', 'price': str(int(15 * 10**18))}})
print('SILK $1.2')
mock_band.execute({'mock_price': {'symbol': 'SHD', 'price': str(int(1.2 * 10**18))}})
'''

print('Configuring Oracle')
oracle = Oracle(
    gen_label(8),
    band_contract=band,
    sscrt=sscrt,
    admin=account_key, 
    uploader=account_key, 
)

# Set to account for fee collection
treasury = PreContract(account, code_hash='', code_id=1)
'''
print('Configuring Treasury')
treasury = Contract(
    '../compiled/treasury.wasm.gz',
    json.dumps({
        'admin': account,
        'viewing_key': viewing_key,
    }),
    gen_label(8),
    admin=account_key, 
    uploader=account_key, 
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
    admin=account_key, 
    uploader=account_key, 
)
print('Registering as SHD Minter')
shade.execute({'set_minters': {'minters': [shade_mint.address]}}, sender=account)

print('Registering sSCRT %100 capture')
shade_mint.execute({'register_asset': {'contract': sscrt.as_dict(), 'capture': str(int(1 * 10 ** 18))}}, sender=account)

print('Registering SILK no capture')
shade_mint.execute({'register_asset': {'contract': silk.as_dict()}}, sender=account)

print('Configuring SILK minting')
silk_mint = Mint(
    gen_label(8),
    native_asset=silk,
    oracle=oracle,
    treasury=treasury,
    admin=account_key, 
    uploader=account_key, 
)
print('Registering as SILK Minter')
silk.execute({'set_minters': {'minters': [silk_mint.address]}}, sender=account)

print('Registering sSCRT %100 capture')
silk_mint.execute({'register_asset': {'contract': sscrt.as_dict(), 'capture': str(int(1 * 10**18))}}, sender=account)
print('Registering SHD no capture')
silk_mint.execute({'register_asset': {'contract': shade.as_dict()}}, sender=account)

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
    admin=account_key, 
    uploader=account_key, 
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
    'scrt_staking': scrt_staking.address,
    #'airdrop': airdrop.address,
}

print(json.dumps(contracts, indent=4))
open('contracts.json', 'w+').write(json.dumps(contracts, indent=4))
