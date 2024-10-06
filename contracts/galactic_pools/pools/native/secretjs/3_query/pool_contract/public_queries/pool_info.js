import pkg from 'secretjs'
const { SecretNetworkClient, grpc } = pkg
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'
import fs from 'fs'
import { debug } from 'console'

async function pool_info () {
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

  let supply_pool_info = await secretjs.query.compute.queryContract({
    contractAddress: contract,
    codeHash: hash, // optional but way faster
    query: { supply_pool_info: {} }
  })

  console.log(
    '-------------------------------- Pool Info Start --------------------------------'
  )
  console.log(`total_delegated: ${supply_pool_info.total_delegated}`)
  console.log(
    `rewards_returned_to_contract: ${supply_pool_info.rewards_returned_to_contract}`
  )
  console.log(`total_withdrawn: ${supply_pool_info.total_withdrawn}`)
  console.log(`total_reserves: ${supply_pool_info.total_reserves}`)
  console.log(`total_sponsored: ${supply_pool_info.total_sponsored}`)

  return true

  // debug(supply_pool_info)
}

export default pool_info
