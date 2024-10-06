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
  let contract_status = await secretjs.query.compute.queryContract({
    contractAddress: contract,
    codeHash: hash, // optional but way faster
    query: { contract_status: {} }
  })

  console.log(
    '--------------------------------Contract Status Start --------------------------------'
  )
  console.log(`contract_status: ${contract_status.status}`)

  return true
}

export default config
