import pkg from 'secretjs'
const { SecretNetworkClient, grpc } = pkg
const grpcWebUrl = 'https://grpc.testnet.secretsaturn.net/'

async function example () {
  // To create a readonly secret.js client, just pass in a gRPC-web endpoint
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId: 'pulsar-2'
  })

  // const {
  //   balance: { amount }
  // } = await secretjs.query.bank.balance(
  //   {
  //     address: 'secret1uzzzzr02xk9cuxn6ejp2axsyf4cjzznklzjmq7',
  //     denom: 'uscrt'
  //   } /*,
  // // optional: query at a specific height (using an archive node)
  // new grpc.Metadata({"x-cosmos-block-height": "2000000"})
  // */
  // )

  // console.log(`I have ${Number(amount) / 1e6} SCRT!`);

  const sSCRT = 'secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg'
  const sScrtCodeHash =
    '9587D60B8E6B078ACE12014CEEEE089530B9FABCD76535D93666A6C127AD8813'

  const { token_info } = await secretjs.query.compute.queryContract({
    contractAddress: sSCRT,
    codeHash: sScrtCodeHash, // optional but way faster
    query: { token_info: {} }
  })

  if (token_info.decimals == 6) {
    console.log(`test example.js ... ok`)
    return true
  } else {
    console.log(`test example.js ... FAILED`)
    return false
  }

  // console.log(`sSCRT has ${token_info.decimals} decimals!`);
}

export default example
