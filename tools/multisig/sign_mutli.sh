#!/bin/bash

# example: ./multisig.sh secret1y277c499f44nxe7geeaqw8t6gpge68rcpla9lf ~/json/output.json jsledger

secretd config node https://rpc.scrt.network:443
secretd config chain-id secret-4
res=`secretd q account $1`
eval sequence=`echo $res | jq ".sequence"`
eval acc_num=`echo $res | jq ".account_number"`
outputdoc="signature_$3.json"
secretd tx sign $2 --multisig ss_multisig --from $3 --output-document $outputdoc --chain-id secret-4 --offline --sequence $sequence --account-number $acc_num --sign-mode amino-json
