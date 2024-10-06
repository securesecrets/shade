import pkg from 'secretjs'
const { SecretNetworkClient, grpc } = pkg
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'
import fs from 'fs'
import { debug } from 'console'

async function config () {
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

  let reward_stats = await secretjs.query.compute.queryContract({
    contractAddress: contract,
    codeHash: hash, // optional but way faster
    query: { rewards_stats: {} }
  })

  console.log(
    '--------------------------------Reward Stats Start --------------------------------'
  )

  debug(reward_stats.distribution_per_tiers.tier_0.claimed)
  debug(reward_stats.distribution_per_tiers.tier_1.claimed)
  debug(reward_stats.distribution_per_tiers.tier_2.claimed)
  debug(reward_stats.distribution_per_tiers.tier_3.claimed)
  debug(reward_stats.distribution_per_tiers.tier_4.claimed)
  debug(reward_stats.distribution_per_tiers.tier_5.claimed)
  debug(reward_stats)
  return true
}

export default config
