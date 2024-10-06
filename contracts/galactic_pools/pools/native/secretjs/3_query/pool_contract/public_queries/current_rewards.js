import pkg from 'secretjs'
const { SecretNetworkClient, grpc } = pkg
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'
import fs from 'fs'
import { debug } from 'console'

async function current_rewards () {
  // To create a readonly secret.js client, just pass in a gRPC-web endpoint
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId: 'pulsar-2'
  })

  let bufferData = fs.readFileSync('../contract_details.json')
  let stData = bufferData.toString()
  let data = JSON.parse(stData)

  const contract = data.address

  const hash = data.hash

  let current_rewards = await secretjs.query.compute.queryContract({
    contractAddress: contract,
    codeHash: hash, // optional but way faster
    query: { current_rewards: {} }
  })

  console.log(
    '--------------------------------Contract Rewards Start --------------------------------'
  )
  console.log(`current_rewards: ${current_rewards.rewards}`)

  return true
}

export default current_rewards
