import pkg from 'secretjs'
const { Wallet, SecretNetworkClient, grpc, signAmino } = pkg
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'
import fs from 'fs'
import { debug } from 'console'
import * as dotenv from 'dotenv' // see https://github.com/motdotla/dotenv#how-do-i-use-dotenv-with-import
dotenv.config({ path: '../.env' })

async function sscrt_balance () {
  const wallet = new Wallet(process.env.MNEMONIC)
  const myAddress = wallet.address

  // To create a readonly secret.js client, just pass in a gRPC-web endpoint
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId: process.env.SECRET_CHAIN_ID,
    wallet: wallet,
    walletAddress: myAddress
  })

  let bufferData = fs.readFileSync('../contract_details.json')
  let stData = bufferData.toString()
  let data = JSON.parse(stData)

  const allowedTokens = [process.env.SSCRT_ADDRESS]
  const permissions = ['owner', 'balance']
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

  let query = await secretjs.query.snip20.getBalance({
    contract: {
      address: process.env.SSCRT_ADDRESS,
      codeHash: process.env.SSCRT_HASH
    },
    address: myAddress,
    auth: { permit }
  })

  //   let supply_pool_info = await secretjs.query.compute.queryContract({
  //     contractAddress: env.process.SSCRT_ADDRESS,
  //     codeHash: env.process.SSCRT_HASH, // optional but way faster
  //     query: {
  //       with_permit: {
  //         permit,
  //         balance: {}
  //       }
  //     }
  //   })

  console.log(
    '-------------------------------- sScrt Balance Start --------------------------------'
  )

  debug(
    'sScrt Balance: ' +
      query.balance.amount / process.env.SCRT_TO_USCRT +
      ' SSCRT'
  )

  return true
  //   if (token_info.decimals == 6) {
  //     console.log(`test example.js ... ok`)
  //     return true
  //   } else {
  //     console.log(`test example.js ... FAILED`)
  //     return false
  //   }

  // console.log(`sSCRT has ${token_info.decimals} decimals!`);
}

export default sscrt_balance
