#!/usr/bin/python3
import json

from os import listdir

from secret_sdk.client.lcd import LCDClient
from secret_sdk.key.mnemonic import MnemonicKey
from secret_sdk.core.auth import StdFee
from secret_sdk.core.bank import MsgSend
from secret_sdk.core.msg import Msg

from contractlib.secretlib.secretlib import run_command

def to_decimal(amount, decimals):
    return float(str(amount[:-decimals]) + '.' + str(amount)[-decimals:])

def pool_change(shd_pool, sscrt_pool, desired_ratio):
    # (sscrt_pool * shd_pool) / (sswap_sscrt - sscrt_change)
    cp = shd_pool * sscrt_pool
    cur_ratio = shd_pool / sscrt_pool


# temp
MNEMONIC = 'crucial design under luxury disagree diet grid pen fish live lava draw '

CHAIN_ID = 'secret-4'
REST_URL = 'http://104.217.248.14:26657'

SHD = 'secret1qfql357amn448duf5gvp9gr48sxx9tsnhupu3d'
SSCRT = 'secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek'
SIENNA_PAIR = 'secret1drm0dwvewjyy0rhrrw485q4f5dnfm6j25zgfe5'
SECRETSWAP_PAIR = 'secret1wwt7nh3zyzessk8c5d98lpfsw79vzpsnerj6d0'

DESIRED_RATIO = 1.15

print()
print('SecretSwap')

sswap = json.loads(run_command(f'secretd q compute query {SECRETSWAP_PAIR}'.split(' ') + [json.dumps({'pool': {}})]))
sswap_shd = to_decimal(sswap['assets'][1]['amount'], 8)
sswap_sscrt = to_decimal(sswap['assets'][0]['amount'], 6)

print('SHD:\t\t', sswap_shd)
print('SCRT:\t\t', sswap_sscrt)

print('SHD/SCRT:\t', sswap_shd / sswap_sscrt)
print('CP:\t\t', sswap_shd * sswap_sscrt)

# print(sswap)

print()

print('Sienna')

sienna = json.loads((run_command(f'secretd q compute query {SIENNA_PAIR}'.split(' ') + ['"pair_info"'])))
sienna_shd = to_decimal(sienna['pair_info']['amount_0'], 8)
sienna_sscrt = to_decimal(sienna['pair_info']['amount_1'], 6)

print('SHD:\t\t', sienna_shd)
print('SCRT:\t\t', sienna_sscrt)
print('SHD/SCRT:\t', sienna_shd / sienna_sscrt)
print('CP:\t\t', sienna_shd * sienna_sscrt)

print()

# print(sienna)
