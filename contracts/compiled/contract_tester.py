import string
import random
from contractlib.secretlib import secretlib
from contractlib.snip20lib import SNIP20
from contractlib.mintlib import Mint


def gen_label(length):
    # With combination of lower and upper case
    return ''.join(random.choice(string.ascii_letters) for i in range(length))


account_key = 'a'
account = secretlib.run_command(['secretcli', 'keys', 'show', '-a', 'a']).rstrip()

print("Configuring sSCRT")
sscrt = SNIP20(gen_label(8), decimals=6, public_total_supply=True, enable_deposit=True)
sscrt_password = sscrt.set_view_key(account_key, "password")

sscrt_mint_amount = '100000000000000'
print(f"\tDepositing {sscrt_mint_amount} uSCRT")
sscrt.deposit(account, sscrt_mint_amount + "uscrt")
sscrt_minted = sscrt.get_balance(account, sscrt_password)
print(f"\tReceived {sscrt_minted} usSCRT")
assert sscrt_mint_amount == sscrt_minted, f"Minted {sscrt_minted}; expected {sscrt_mint_amount}"

print("Configuring silk")
silk = SNIP20(gen_label(8), public_total_supply=True, enable_mint=True)
silk_password = silk.set_view_key(account_key, "password")

print("Configuring Mint contract")
mint = Mint(gen_label(8), silk, "oracle")
# TODO: check that the initialized contract is legit
silk.set_minters([mint.address])
mint.register_asset(sscrt)
assets = mint.get_supported_assets()['supported_assets']['assets'][0]
assert sscrt.address == assets, f"Got {assets}; expected {sscrt.address}"

print("Sending to mint contract")

total_amount = int(sscrt_mint_amount)
minimum_amount = 1000
total_tests = 5

total_sent = 0

for i in range(total_tests):
    send_amount = random.randint(minimum_amount, int(total_amount/total_tests)-1)
    total_sent += send_amount

    print(f"\tSending {send_amount} usSCRT")
    sscrt.send(account_key, mint.address, send_amount)
    silk_minted = silk.get_balance(account, silk_password)
    assert total_sent == int(silk_minted), f"Total minted {silk_minted}; expected {total_sent}"

    print(f"\tSilk balance: {silk_minted} uSILK")
    burned_amount = mint.get_asset(sscrt)["asset"]["asset"]["burned_tokens"]
    print(f"\tTotal burned: {burned_amount} usSCRT")
    assert total_sent == int(burned_amount), f"Burnt {burned_amount}; expected {total_sent}"
