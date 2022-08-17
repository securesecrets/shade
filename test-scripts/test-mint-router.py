#!/usr/bin/env python3
import json
from time import sleep
from contractlib.contractlib import Contract, PreInstantiatedContract
from contractlib.utils import gen_label
from contractlib.secretlib.secretlib import run_command, execute_contract, query_contract
from contractlib.snip20lib import SNIP20
from contractlib.mintlib import Mint
from contractlib.oraclelib import Oracle

viewing_key = 'password'


ACCOUNT_KEY = 'drpresident'
backend = 'test'
ACCOUNT = run_command(['secretd', 'keys', 'show', '-a', ACCOUNT_KEY]).rstrip()

sscrt = PreInstantiatedContract(
        'secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg',
        '',
        1,
)
shade = PreInstantiatedContract(
        'secret19ymc8uq799zf36wjsgu4t0pk8euddxtx5fggn8',
        '5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1',
        1,
)
silk = PreInstantiatedContract(
        'secret18k8a6lytr3gxppv96qus5qazg093gks7pk4q5x',
        '5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1',
        1,
)
oracle = PreInstantiatedContract(
        'secret1shtrrcqjsq53yw99wn3dhacyan6e8mjs23y0hd',
        '813F80046A26DAA3D4310603AA3B2D0C98C0C0956E7C72DCD38778C1F7E3D472',
        1,
)

treasury = PreInstantiatedContract(
        ACCOUNT,
        '',
        1,
)

print('Configuring Silk Mint')
silk_mint = Mint(gen_label(8), 
    native_asset=silk,
    oracle=oracle,
    treasury=treasury,
    admin=ACCOUNT_KEY,
    uploader=ACCOUNT_KEY,
)
print(silk_mint.address)

print('Registering Silk Mint')
silk.execute({
    'set_minters': {
        'minters': [silk_mint.address]
    }
}, sender=ACCOUNT)

print('Configuring Shade Mint')
shade_mint = Mint(gen_label(8), 
    native_asset=shade, 
    oracle=oracle,
    treasury=treasury,
    admin=ACCOUNT_KEY,
    uploader=ACCOUNT_KEY,
)
print(shade_mint.address)

print('Registering Shade Mint')
shade.execute({
    'set_minters': {
        'minters': [shade_mint.address]
    }
}, sender=ACCOUNT)

print('Enable minting SHD with SILK & sSCRT')
shade_mint.execute({
    'register_asset': {
        'contract': sscrt.as_dict(),
        'capture': str(int(1 * 10 **18)),
    }
}, sender=ACCOUNT)
shade_mint.execute({
    'register_asset': {
        'contract': silk.as_dict(),
    }
}, sender=ACCOUNT)

print('Enable minting SILK with SHD')
silk_mint.execute({
    'register_asset': {
        'contract': shade.as_dict(),
    }
}, sender=ACCOUNT)

print('Configuring Mint Router')
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

print('ACCOUNT', ACCOUNT)
print('SHD-Mint', shade_mint.as_dict())
print('SILK-Mint', silk_mint.as_dict())
print('ROUTER', mint_router.as_dict())

print(sscrt.get_balance(ACCOUNT, viewing_key), 'sSCRT')

print('Minting SHD with sSCRT')
print(json.dumps(sscrt.execute({
        "send": {
            "recipient": shade_mint.address,
            "amount": amount,
        },
    },
    ACCOUNT,
), indent=2))

print(shade.get_balance(ACCOUNT, viewing_key), 'SHD')
print(silk.get_balance(ACCOUNT, viewing_key), 'SILK')

'''
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
'''
