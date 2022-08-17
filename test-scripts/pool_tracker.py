#!/usr/bin/python3
import json

from os import listdir
from time import sleep
from datetime import datetime, timezone
from os.path import exists

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

HEADER = ','.join(('Timestamp', 'SHD', 'sSCRT', 'Ratio', 'CP'))
FILE_SUFFIX = '_pool_data.csv'

DELAY = 30

pairs = {
    'sienna': 'secret1drm0dwvewjyy0rhrrw485q4f5dnfm6j25zgfe5',
    'secretswap': 'secret1wwt7nh3zyzessk8c5d98lpfsw79vzpsnerj6d0',
}

if not exists('secretswap'+ FILE_SUFFIX):
    open('secretswap' + FILE_SUFFIX, 'w+').write(HEADER + '\n')

if not exists('sienna'+ FILE_SUFFIX):
    open('sienna' + FILE_SUFFIX, 'w+').write(HEADER + '\n')

# for determining diff
prev_data = ()
while True:

    if prev_data:
        print('Sleeping...')
        sleep(DELAY)
        print('-' * 24)

    timestamp = datetime.now(timezone.utc)

    print()
    print('SecretSwap')

    try:
        sswap = json.loads(run_command(f'secretd q compute query {SECRETSWAP_PAIR}'.split(' ') + [json.dumps({'pool': {}})]))
    except:
        print('SecretSwap Failure')
        continue

    sswap_shd = to_decimal(sswap['assets'][1]['amount'], 8)
    sswap_sscrt = to_decimal(sswap['assets'][0]['amount'], 6)
    sswap_ratio = sswap_shd / sswap_sscrt
    sswap_cp = sswap_shd * sswap_sscrt

    print('SHD:\t', sswap_shd)
    print('sSCRT:\t', sswap_sscrt)

    print('Ratio:\t', sswap_ratio)
    print('CP:\t', sswap_cp)


    print()

    print('Sienna')

    try:
        sienna = json.loads((run_command(f'secretd q compute query {SIENNA_PAIR}'.split(' ') + ['"pair_info"'])))
    except:
        print('Sienna Failure')
        continue

    sienna_shd = to_decimal(sienna['pair_info']['amount_0'], 8)
    sienna_sscrt = to_decimal(sienna['pair_info']['amount_1'], 6)
    sienna_ratio = sienna_shd / sienna_sscrt
    sienna_cp = sienna_shd * sienna_sscrt

    print('SHD:\t', sienna_shd)
    print('sSCRT:\t', sienna_sscrt)
    print('Ratio:\t', sienna_ratio)
    print('CP:\t', sienna_cp)

    print()


    cur_data = (sswap_shd, sswap_sscrt, sienna_shd, sienna_sscrt)

    if cur_data != prev_data:

        print('Writing...')
        open('secretswap' + FILE_SUFFIX, 'a').write(','.join(map(str, (timestamp, sswap_shd, sswap_sscrt, sswap_ratio, sswap_cp))) + '\n')
        open('sienna' + FILE_SUFFIX, 'a').write(','.join(map(str, (timestamp, sienna_shd, sienna_sscrt, sienna_ratio, sienna_cp))) + '\n')
        prev_data = cur_data

