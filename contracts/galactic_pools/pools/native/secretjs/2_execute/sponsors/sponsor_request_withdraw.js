import { Wallet, SecretNetworkClient } from 'secretjs'
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'

import * as dotenv from 'dotenv' // see https://github.com/motdotla/dotenv#how-do-i-use-dotenv-with-import
dotenv.config({ path: '../.env' })
import fs from 'fs'
import { debug } from 'console'

async function request_withdraw (am = 10) {
  const wallet = new Wallet(process.env.MNEMONIC)
  const myAddress = wallet.address

  // To create a signer secret.js client, also pass in a wallet
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId: 'pulsar-2',
    wallet: wallet,
    walletAddress: myAddress
  })

  let bufferData = fs.readFileSync('./../contract_details.json')
  let stData = bufferData.toString()
  let data = JSON.parse(stData)

  const contractAddress = data.address
  const codeHash = data.hash
  let amount = String(am * process.env.SCRT_TO_USCRT)

  let sim = await secretjs.tx.compute.executeContract.simulate({
    sender: myAddress,
    contractAddress: contractAddress,
    codeHash: codeHash, // optional but way faster
    msg: {
      sponsor_request_withdraw: {
        amount
      }
    }
  })

  let gasLimit = Math.ceil(sim.gasInfo.gasUsed * 1.34)
  console.log('gasLimit = ' + gasLimit)

  const tx = await secretjs.tx.compute.executeContract(
    {
      sender: myAddress,
      contractAddress: contractAddress,
      codeHash: codeHash, // optional but way faster
      msg: {
        sponsor_request_withdraw: { amount }
      }
    },
    {
      gasLimit
    }
  )
  console.log(tx)
  return true
}

export default request_withdraw
