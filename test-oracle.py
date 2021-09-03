#!/usr/bin/env python3
import json
from contractlib.secretlib.secretlib import query_contract
from contractlib.contractlib import PreInstantiatedContract as Contract
from contractlib.oraclelib import Oracle
from contractlib.utils import gen_label

####### SSCRT Pairs ONLY ###
#sscrt_snip = 'secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx'
#sscrt_code_hash = 'cd400fb73f5c99edbc6aab22c2593332b8c9f2ea806bf9b42e3a523f3ad06f62'

silk_snip = 'secret12dlkq02clar92hrtxsd8dy54xcm088mzu2vftu'
silk_pair = 'secret1j3llhgudfqtuqq3qhfflqzyhu5dgrxeeztxhqx'

socean_snip = 'secret10zr3azpmr42vatq3pey2aaxurug0c668km6rzl'
socean_pair = Contract(
    'secret1nv90j233x88teghhwdz9l0hj4vzrrcwjl4q6fg',
    'f86b5c3ca0381ce7edfffa534789501ae17cf6b21515213693baf980765729c2',
    code_id=None
)

seth_snip = 'secret1ttg5cn3mv5n9qv8r53stt6cjx8qft8ut9d66ed'
seth_pair = Contract(
    'secret148jpzfh6lvencwtxa6czsk8mxm7kuecncz0g0y',
    'f86b5c3ca0381ce7edfffa534789501ae17cf6b21515213693baf980765729c2',
    code_id=None
)

tsusdt_snip = 'secret196uyuzlw039hztfmv4g0kjp2u376ucsfv0fmss'
tsusdt_pair = 'secret1qv83078fdq6swmrh34f93ps3xlf5nxkhq9mlqx'

band = Contract(
    'secret1p0jtg47hhwuwgp4cjpc46m7qq6vyjhdsvy2nph',
    '77c854ea110315d5103a42b88d3e7b296ca245d8b095e668c69997b265a75ac5',
code_id=None
)
sscrt = Contract(
    'secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx',
    'cd400fb73f5c99edbc6aab22c2593332b8c9f2ea806bf9b42e3a523f3ad06f62',
    code_id=None
)

sim = {
  "simulation": {
    "offer_asset": {
    # 1 sSCRT?
      "amount": '1000000',
      "info": {
        "token": {
        # Sending sSCRT
          "contract_addr": sscrt.address,
          "token_code_hash": sscrt.code_hash,
          "viewing_key": "SecretSwap"
        }
      }
    }
  }
}
token_info = {'token_info': {}}


def normalize(amount, decimals: int):
    print('normalizing', amount, decimals)
    amount = str(amount)
    i = len(amount) - decimals
    if i < 0:
        norm = '.' + '0' * abs(i) + amount
    else:
        norm = amount[:i] + '.' + amount[i:]

    return float(norm)


# sent_decimals = query_contract(sscrt_snip, json.dumps(token_info))['token_info']['decimals']

def sswap_price(snip20, sscrt_pair):
    info = query_contract(snip20, json.dumps(token_info))['token_info']

    results = query_contract(sscrt_pair, json.dumps(sim))
    return_amount = int(results['return_amount'])
    return normalize(return_amount, info['decimals']), info['symbol']

oracle = Oracle(gen_label(8), band, sscrt, admin='drpresident', uploader='drpresident', backend=None)
print(oracle.address)
print(oracle.code_hash)
'''
pre_oracle = Contract(
    'secret1tz04utcn7du2lpc6k2jx92mvjmkru88znnsdts',
    '513BF4DB66107285F1F6E3DE3BCCF7189B1AFC9BEA921D454B5CE085CEE0A475',
code_id=None)
oracle = Oracle('', band, sscrt, instantiated_contract=pre_oracle)
'''

print('Registering SETH')
print(oracle.register_sswap_pair(seth_pair))
print(oracle.get_price('SETH')['rate'], 'SETH')

print('Registering SOCEAN')
print(oracle.register_sswap_pair(socean_pair))
print(oracle.get_price('SOCEAN')['rate'], 'SOCEAN')
