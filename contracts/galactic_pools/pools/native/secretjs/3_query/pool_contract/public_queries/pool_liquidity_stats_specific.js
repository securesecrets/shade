import pkg from 'secretjs'
const { SecretNetworkClient, grpc } = pkg
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'
import fs from 'fs'
import { debug } from 'console'

async function pool_liq_stats_specific () {
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

  let supply_pool_liquidity_stats_specific = await secretjs.query.compute.queryContract(
    {
      contractAddress: contract,
      codeHash: hash, // optional but way faster
      query: {
        supply_pool_liquidity_stats_specific: {
          round_index: 1
        }
      }
    }
  )

  console.log(
    '-------------------------------- Pool Liq Stats Specific Start --------------------------------'
  )
  console.log(
    `total_liq: ${supply_pool_liquidity_stats_specific.total_liquidity}`
  )
}

export default pool_liq_stats_specific
