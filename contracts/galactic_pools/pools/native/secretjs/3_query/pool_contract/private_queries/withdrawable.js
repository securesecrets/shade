import pkg from 'secretjs'
const { Wallet, SecretNetworkClient, grpc, signAmino } = pkg
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'
import fs from 'fs'
import { debug } from 'console'
import * as dotenv from 'dotenv' // see https://github.com/motdotla/dotenv#how-do-i-use-dotenv-with-import
dotenv.config({ path: '../.env' })

async function withdrawable () {
  const wallet = new Wallet(process.env.MNEMONIC)
  const myAddress = wallet.address

  // To create a readonly secret.js client, just pass in a gRPC-web endpoint
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId: 'pulsar-2',
    wallet: wallet,
    walletAddress: myAddress
  })

  let bufferData = fs.readFileSync('../contract_details.json')
  let stData = bufferData.toString()
  let data = JSON.parse(stData)
  const contract = data.address

  const hash = data.hash

  const allowedTokens = [data.address]
  const permissions = ['owner', 'delegated']
  const chainId = process.env.SECRET_CHAIN_ID
  let permitName = 'Permit1'

  let permit = await secretjs.utils.accessControl.permit.sign(
    myAddress,
    chainId,
    permitName,
    allowedTokens,
    permissions,
    false
  )
  let withdrawable = await secretjs.query.compute.queryContract({
    contractAddress: contract,
    codeHash: hash, // optional but way faster
    query: {
      with_permit: {
        permit,

        query: {
          withdrawable: {}
        }
      }
    }
  })

  console.log(
    '-------------------------------- withdrawbable Start --------------------------------'
  )

  console.log(
    `Total withdrawbable:  ${parseInt(
      withdrawable.amount / process.env.SCRT_TO_USCRT
    )} SCRT`
  )
  return true
}

export default withdrawable
