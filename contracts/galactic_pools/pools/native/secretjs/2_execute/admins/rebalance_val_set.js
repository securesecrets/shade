import { Wallet, SecretNetworkClient } from 'secretjs'
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'
import * as dotenv from 'dotenv'
dotenv.config({ path: '../.env' })
import fs from 'fs'

async function unbond_batch () {
  const wallet = new Wallet(process.env.MNEMONIC)
  const myAddress = wallet.address

  // To create a signer secret.js client, also pass in a wallet
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId: 'pulsar-2',
    wallet: wallet,
    walletAddress: myAddress
    // encryptionSeed: new Uint8Array(
    //   Buffer.from('helloworld1234567891123456789123')
    // )
  })

  let bufferData = fs.readFileSync('./../contract_details.json')
  let stData = bufferData.toString()
  let data = JSON.parse(stData)

  const contractAddress = data.address

  const codeHash = data.hash

  // let sim = await secretjs.tx.compute.executeContract.simulate({
  //   sender: myAddress,
  //   contractAddress: contractAddress,
  //   codeHash: codeHash, // optional but way faster
  //   msg: {
  //     unbond_batch: {}
  //   }
  // })

  // let gasLimit = Math.ceil(sim.gasInfo.gasUsed * 1.9)
  let gasLimit = 1000000
  console.log('gasLimit = ' + gasLimit)

  const tx = await secretjs.tx.compute.executeContract(
    {
      sender: myAddress,
      contractAddress: contractAddress,
      codeHash: codeHash, // optional but way faster
      msg: {
        rebalance_validator_set: {}
      }
    },
    {
      gasLimit
    }
  )
  console.log(tx)
}

export default unbond_batch
