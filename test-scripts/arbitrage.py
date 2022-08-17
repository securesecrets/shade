import random
from re import U
import secrets
import json
from secret_sdk.client.lcd import LCDClient
from secret_sdk.key.mnemonic import MnemonicKey
from secret_sdk.core.wasm import MsgExecuteContract
from secret_sdk.core.auth import StdFee
import base64
from secret_sdk.core.bank import MsgSend

mk = MnemonicKey(mnemonic="easy oxygen bone search trophy soccer video float tiny rack fragile cactus uphold acoustic carbon warm hand pilot topic session because seed magnet domain")
client = LCDClient('https://api.scrt.network', 'secret-4')
#client = LCDClient('https://lcd.secret.llc', 'secret-4')

SSWAP_SSCRT_SHD_PAIR = "secret1wwt7nh3zyzessk8c5d98lpfsw79vzpsnerj6d0"
SSWAP_SSCRT_SHD_PAIR_HASH = client.wasm.contract_hash(SSWAP_SSCRT_SHD_PAIR)
SSWAP_QUERY = { 'pool': {} }
SIENNA_SSCRT_SHD_PAIR = "secret1drm0dwvewjyy0rhrrw485q4f5dnfm6j25zgfe5"
SIENNA_SSCRT_SHD_PAIR_HASH = client.wasm.contract_hash(SIENNA_SSCRT_SHD_PAIR)
SIENNA_QUERY = 'pair_info'

SSCRT_ADRESS = 'secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek'
SSCRT_KEY = 'api_key_nE9AgouX7GVnT0+3LhAGoNmwUZ7HHRR4sUxNB+tbWW4='

SHD_ADRESS = 'secret1qfql357amn448duf5gvp9gr48sxx9tsnhupu3d'
SHD_KEY = 'api_key_xSmcfnyU8z5750bZSC9icFYUz6whMLIUeEloEqORQ7M='

wallet = client.wallet(mk)
temp = wallet.account_number_and_sequence()
accNum, sequence = temp['account_number'], temp['sequence']
scrtBal, sscrtBal, shdBal = 0, 0, 0
testShdBal = 10
fee = StdFee(100000, "25000uscrt")

def getBalances():
  scrtBalRes = client.bank.balance(mk.acc_address)
  sscrtBalRes = client.wasm.contract_query(SSCRT_ADRESS, { "balance": { "address": mk.acc_address, "key": SSCRT_KEY }})
  shdBalRes = client.wasm.contract_query(SHD_ADRESS, { "balance": { "address": mk.acc_address, "key": SHD_KEY }})
  scrtBal, sscrtBal, shdBal = scrtBalRes * 10**-6, float(sscrtBalRes['balance']['amount'])* 10**-6, float(shdBalRes['balance']['amount'])* 10**-8
getBalances()

def getSiennaRatio():
  siennaInfo = client.wasm.contract_query(SIENNA_SSCRT_SHD_PAIR, SIENNA_QUERY)
  shdAmount, sscrtAmount = float(siennaInfo['pair_info']['amount_0']) * 10**-8, float(siennaInfo['pair_info']['amount_1'])*10**-6
  return sscrtAmount/shdAmount

def getSSwapRatio():
  sswapInfo = client.wasm.contract_query(SSWAP_SSCRT_SHD_PAIR, SSWAP_QUERY)
  sscrtAmount, shdAmount = float(sswapInfo['assets'][0]['amount'])*10**-6, float(sswapInfo['assets'][1]['amount'])*10**-8
  return sscrtAmount/shdAmount

def calculateProfitability(r1, r2):
  aveprice = (r1 + r2)/2
  gasFeeShd = .15 / aveprice
  print("gas: ", gasFeeShd ) #.99499999
  
  workingBal = testShdBal * .8 #Change to shdBal when actually trading
  firstSwap = (workingBal - workingBal * .003 ) * r1
  print("firstswap: ", firstSwap)
  secondSwap = (firstSwap - firstSwap * .003 ) / r2
  print("secondswap: ", secondSwap)
  return secondSwap - workingBal - gasFeeShd

def calculateProfitability2(r1, r2):
  aveprice = (r1 + r2)/2
  gasFeeShd = .15 / aveprice
  print("gas: ", gasFeeShd )
  
  workingBal = testShdBal * .8 #Change to shdBal when actually trading
  firstSwap = workingBal * r1 * .99
  print("firstswap: ", firstSwap)
  secondSwap = firstSwap / r2 * .99
  print("secondswap: ", secondSwap)
  return secondSwap - workingBal - gasFeeShd

def swapSienna():
  msg = json.dumps({
      'swap': {
          'to': None,
          'expected_return': '1001375',
      }
  })
  encryptedMsg = str( base64.b64encode(msg.encode("utf-8")), "utf-8")
  handleMsg = { "send": {"recipient": SIENNA_SSCRT_SHD_PAIR, "amount": "10000000", "msg": encryptedMsg }}
  #data = {"value": {"sender": mk.acc_address, "contract": SHD_ADRESS, "msg": handleMsg, "sent_funds": ""}}
  #executeMsg = MsgExecuteContract(mk.acc_address, SHD_ADRESS, handleMsg)
  #print(executeMsg.to_data())
  msgTest = client.wasm.contract_execute_msg(mk.acc_address, SHD_ADRESS, handleMsg, [])
  #print(msgTest.to_data())
  #encryptedMsg = client.utils.encrypt(client.utils.get_tx_encryption_key(secrets.randbelow(10000)),executeMsg)
  #msg = MsgExecuteContract(sender=mk.acc_address, contract=SIENNA_SSCRT_SHD_PAIR, execute_msg="")
  #print(SIENNA_SSCRT_SHD_PAIR_HASH)
  #encryptedMsg = client.utils.encrypt(SIENNA_SSCRT_SHD_PAIR_HASH, executeMsg)
  tx = wallet.create_and_sign_tx([msgTest], fee, account_number=135673, sequence=10)
  print(tx.to_data())
  #signedTx = wallet.execute_tx()
  res = client.auth.account_info(mk.acc_address)
  signedTx = mk.create_signature(tx)
  #print(signedTx.to_data())
  print(sequence)
  #wallet.execute_tx
  #res = client.tx.broadcast(tx)
  #print(res)

def main():
  siennaRatio = getSiennaRatio()
  sswapRatio = getSSwapRatio()
  difference = siennaRatio - sswapRatio 
  print("Sienna: ", siennaRatio)
  print("SSwap: ", sswapRatio)
  print(siennaRatio - sswapRatio)
  profit = 0
  if( difference > 0 ):
    profit = calculateProfitability(siennaRatio, sswapRatio)
    print("profit: ", profit)
  if( difference < 0 ):
    profit = calculateProfitability(sswapRatio, siennaRatio)
    print("profit: ", profit)
  if( profit > 0 and difference > 0):
    #swapSienna()
    return
  if( profit > 0 and difference < 0):
    #swapSswap()
    return
  swapSienna()
  #print(res)

main()


#wallet = client.wallet(mk)

#tx = wallet.create_and_sign_tx(
#    msgs=[MsgSend(
#        wallet.key.acc_address,
#        "secret1kq56y5s8tfawa0k6gv9np86r4wtm8q86zl3n5v",
#        "1000000uscrt" # send 1 scrt
#    )],
#    memo="test transaction!",
#    fee=StdFee(200000, "120000uscrt")
#)




