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

mint_amount = '100000000'
print(f"\tDepositing {mint_amount} uSCRT")
sscrt.deposit(account, mint_amount+"uscrt")
minted_amount = sscrt.get_balance(account, sscrt_password)
print(f"\tReceived {minted_amount} usSCRT")
assert mint_amount == minted_amount, f"Minted {minted_amount}; expected {mint_amount}"

print("Configuring silk")
silk = SNIP20(gen_label(8), public_total_supply=True, enable_mint=True)
silk_password = silk.set_view_key(account_key, "password")

print("Configuring Mint contract")
mint = Mint(gen_label(8), silk, "oracle")
silk.set_minters([mint.address])
mint.register_asset(sscrt)

print("Sending to mint contract")
send_amount = '50000000'
print(f"\tSending {send_amount} usSCRT")
sscrt.send(account_key, mint.address, send_amount)
minted_amount = silk.get_balance(account, silk_password)
print(f"\tReceived {minted_amount} uSILK")
assert send_amount == minted_amount, f"Sent {minted_amount}; expected {send_amount}"
