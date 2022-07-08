import argparse
import os

parser = argparse.ArgumentParser(description="Create a cosmwasm msg for offline signing")
parser.add_argument("contract_address", type=str, help="Smart contract's address")
parser.add_argument("contract_codehash", type=str, help="Smart contract's code hash")
parser.add_argument("msg", type=str, help="Smart contract's msg to execute")
parser.add_argument("sender", type=str, help="The msg sender")
parser.add_argument("key", type=str, help="Enclave key certificate")
parser.add_argument("-o", "--output", type=str, help="Output message")
parser.add_argument("--use_old", action="store_true", help="Uses secretcli instead of secretd")
args = parser.parse_args()

bin = "secretd"

if args.use_old:
    bin = "secretcli"

output = "output.json"

if args.output:
    output = args.output

command = f"{bin} tx compute execute {args.contract_address} '{args.msg}' --from {args.sender} --generate-only --enclave-key {args.key} --code-hash {args.contract_codehash} --offline --sign-mode amino-json > {output}"
os.system(command)