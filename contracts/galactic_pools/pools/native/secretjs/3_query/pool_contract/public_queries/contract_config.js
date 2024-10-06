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

  let contract_config = await secretjs.query.compute.queryContract({
    contractAddress: contract,
    codeHash: hash, // optional but way faster
    query: { contract_config: {} }
  })

  let time_left = 0
  if (Date.now() / 1000 < contract_config.next_unbonding_batch_time) {
    time_left = parseInt(
      Number(
        contract_config.next_unbonding_batch_time - Number(Date.now() / 1000)
      )
    )
  }

  console.log(
    '--------------------------------Contract Config Start --------------------------------'
  )

  console.log(`Admin: ${contract_config.admin}`)
  console.log(`Triggerer: ${contract_config.triggerer}`)
  console.log(`Denom: ${contract_config.denom}`)
  console.log(`Contract Address: ${contract_config.contract_address}`)
  for (const val of contract_config.validators) {
    console.log(val)
  }
  console.log(`Time left to unbond: ${time_left}`)

  console.log(
    `Next_unbonding_batch_amount: ${contract_config.next_unbonding_batch_amount}`
  )
  console.log(
    `unbonding_batch_duration: ${contract_config.unbonding_batch_duration}`
  )
  console.log(`unbonding_duration: ${contract_config.unbonding_duration}`)

  return true
}

export default config
