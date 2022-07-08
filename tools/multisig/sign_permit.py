import argparse
import os

parser = argparse.ArgumentParser(description="Create a cosmwasm msg for offline signing")

parser.add_argument("msg", type=str, help="Permit data")
parser.add_argument("account", type=str, help="Permit signer")
parser.add_argument("--account_number", type=str, help="Account number", default="0")
parser.add_argument("--chain_id", type=str, help="Chain id to which this permit is written for", default="secret-4")
parser.add_argument("--memo", type=str, help="Memo for the permit", default="")
parser.add_argument("--msg_type", type=str, help="Msg type used on the signed msg", default="signature_proof")
parser.add_argument("--sequence", type=str, help="Signature sequence number", default="0")

parser.add_argument("-o", "--output", type=str, help="Output message")
parser.add_argument("--use_old", action="store_true", help="Uses secretcli instead of secretd")
args = parser.parse_args()

bin = "secretd"

if args.use_old:
    bin = "secretcli"

output = "signed.json"

if args.output:
    output = args.output

unsigned_permit = f'echo \' {{ "account_number": "{args.account_number}", ' \
                  f'"chain_id": "{args.chain_id}", ' \
                  f'"fee": {{ "amount": [{{ "amount": "0", "denom": "uscrt"}}], "gas": "1" }}, ' \
                  f'"memo": "{args.memo}", "msgs": [{{ "type": "{args.msg_type}", "value": {args.msg} }}], ' \
                  f'"sequence": "{args.sequence}"}} \'> unsigned.json'
os.system(unsigned_permit)

command = f'{bin} tx sign-doc unsigned.json --from {args.account} > {output}'
os.system(command)