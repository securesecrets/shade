import pkg from 'secretjs'
const { SecretNetworkClient, grpc } = pkg
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'
import fs from 'fs'
import { debug } from 'console'

async function sponsors () {
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

  let sponsors = await secretjs.query.compute.queryContract({
    contractAddress: contract,
    codeHash: hash, // optional but way faster
    query: { sponsors: {} }
  })

  console.log(
    '-------------------------------- Sponsors Start --------------------------------'
  )
  console.log(sponsors)

  return true

  // debug(supply_pool_info)
}

export default sponsors
