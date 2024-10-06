import { Wallet, SecretNetworkClient } from 'secretjs'
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'

import * as dotenv from 'dotenv' // see https://github.com/motdotla/dotenv#how-do-i-use-dotenv-with-import
dotenv.config({ path: '../.env' })
import fs from 'fs'
import { debug } from 'console'

async function deposits (amt = 10) {
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
  let amount = String(amt * process.env.SCRT_TO_USCRT)

  let sim = await secretjs.tx.compute.executeContract.simulate({
    sender: myAddress,
    contractAddress: contractAddress,
    codeHash: codeHash, // optional but way faster
    msg: {
      sponsor: {}
    },
    sentFunds: [{ amount: amount, denom: 'uscrt' }] // optional
  })

  let gasLimit = Math.ceil(sim.gasInfo.gasUsed * 1.34)
  try {
    const tx = await secretjs.tx.compute.executeContract(
      {
        sender: myAddress,
        contractAddress: contractAddress,
        codeHash: codeHash, // optional but way faster
        msg: {
          sponsor: {}
        },
        sentFunds: [{ amount: amount, denom: 'uscrt' }] // optional
      },
      {
        gasLimit
      }
    )
    console.log(
      `Deposited ${amount / process.env.SCRT_TO_USCRT} Scrt successfully`
    )
  } catch (err) {
    console.log(err)
  }

  return true
}

export default deposits
