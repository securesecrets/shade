#!/usr/bin/env python3
import json
from contractlib.secretlib.secretlib import query_contract, run_command
from contractlib.contractlib import Contract, PreInstantiatedContract
from contractlib.oraclelib import Oracle
from contractlib.mintlib import Mint
from contractlib.snip20lib import SNIP20
from contractlib.utils import gen_label

ACCOUNT_KEY = 'drpresident'
ACCOUNT = run_command(['secretd', 'keys', 'show', '-a', ACCOUNT_KEY]).rstrip()

sscrt = PreInstantiatedContract(
        'secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg',
        '9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813',
        1,
)
shade = PreInstantiatedContract(
        'secret19ymc8uq799zf36wjsgu4t0pk8euddxtx5fggn8',
        '5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1',
        1,
)
shade_mint = PreInstantiatedContract(
        'secret1yp5kkkpdf98ld0q4vy8g3p2awrmlzx6l2eyr64',
        'C8B7AC72513B8DFC27E76E09149F87BF13584011BF1056D55CED5A5A617F6B80',
        1,
)
silk = PreInstantiatedContract(
        'secret18k8a6lytr3gxppv96qus5qazg093gks7pk4q5x',
        '5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1',
        1,
)
silk_mint = PreInstantiatedContract(
        'secret1qam6fr2msajxx8tr079k7x6tx47j4ftuew649p',
        'C8B7AC72513B8DFC27E76E09149F87BF13584011BF1056D55CED5A5A617F6B80',
        1,
)

sienna_pair = PreInstantiatedContract(
        'secret1pak8feexy97myp22pjkxmsp5p8dmlkp4mkfxsl',
        '33eac42c44ee69acfe1f56ce7b14fe009a7b611e86f275d7af2d32dd0d33d5a9',
        0,
)

band = PreInstantiatedContract(
        'secret1ulxxh6erkmk4p6cjehz58cqspw3qjuedrsxp8f',
        'dc6ff596e1cd83b84a6ffbd857576d7693d89a826471d58e16349015e412a3d3',
        0,
)
'''
sswap_pair = PreInstantiatedContract(
        'secret1wwt7nh3zyzessk8c5d98lpfsw79vzpsnerj6d0', 
        '0dfd06c7c3c482c14d36ba9826b83d164003f2b0bb302f222db72361e0927490',
        0,
)
'''

treasury = PreInstantiatedContract(
        ACCOUNT,
        '',
        0,
)

silk_index = {
    'USD': .899,
    'ETH': .001,
    'SCRT': .1,
}

# normalize index basket
silk_index = [
    {'symbol': s, 'weight': str(int(w * 10**18))}
    for s, w in silk_index.items()
]

# normalize
'''
silk_index = [
        { 'symbol': i['symbol'], 'weight': str(int(i['weight'] * 10**18))}
        for i in silk_index
]
'''


print('Deploying Oracle')
oracle = Oracle(gen_label(8), band, sscrt, admin=ACCOUNT_KEY, uploader=ACCOUNT_KEY)

print('oracle', oracle.address)

print('Registering pools for SHD')

'''
print('Registering sswap pool')
print(oracle.execute({
    'register_pair': {
        'pair': sswap_pair.as_dict()
    }
}))

print('price:', oracle.query({'price': {'symbol': 'SHD'}}))
'''

print('Registering sienna pool')
print(oracle.execute({
    'register_pair': {
        'pair': sienna_pair.as_dict()
    }
}))

print('SHD:', oracle.query({'price': {'symbol': 'SHD'}}))

print('Registering Silk index')
print(oracle.execute({
    'register_index': {
        'symbol': 'SILK',
        'basket': silk_index,
    }
}))

print('Configuring mints to new oracle')

for mint in (shade_mint, silk_mint):
    config = mint.query({"config": {}})
    config['oracle'] = oracle.as_dict()
    print(mint.execute({'update_config': {'config': config}}, sender=ACCOUNT))

'''
print('Deploying Shade Mint')
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

print('Registering sSCRT %100 capture')
silk_mint.execute({'register_asset': {'contract': sscrt.as_dict(), 'capture': str(int(1 * 10**18))}}, sender=ACCOUNT)
print('Registering SHD no capture')
silk_mint.execute({'register_asset': {'contract': shade.as_dict()}}, sender=ACCOUNT)

print('Deploying Mint Router')
mint_router = Contract(
    contract='mint_router.wasm.gz',
    initMsg=json.dumps({
        'path': [
            shade_mint.as_dict(),
            silk_mint.as_dict(),
        ]
    }),
    label=gen_label(8),
    admin=ACCOUNT_KEY,
    uploader=ACCOUNT_KEY,
)
'''

print('ORACLE')
print(oracle.as_dict())
'''
print('SHD Mint')
print(shade_mint.as_dict())
print('SILK Mint')
print(silk_mint.as_dict())
print('Mint Router')
print(mint_router.as_dict())
'''

'''
symbols = ['USD', 'SCRT', 'SHD']
print('Querying', symbols)

for symbol in symbols:
    print(symbol, int(oracle.query({'price': {'symbol': symbol}})['rate']) / 10**18)
'''
