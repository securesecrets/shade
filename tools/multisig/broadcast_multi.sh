#!/bin/bash

# GUIDE:
# Organize all the signatures and the tx to be signed in one directory
# Pass that directory as $1
# Pass the name of the file to be signed as $2



cd $1
res=`ls`
signatures=""
for files in $res
do 
  if [ $files == signedMultiTx.json ]
  then
    rm signedMultiTx.json
  fi
  if [ $files == $2 ]
  then
    continue
  else
    signatures=$signatures" "$files
  fi
done

secretd tx multisign $2 ss_multisig $signatures --chain-id secret-4 > signedMultiTx.json
secretd tx broadcast signedMultiTx.json
