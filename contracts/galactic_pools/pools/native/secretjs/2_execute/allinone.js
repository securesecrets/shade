import deposit from './users/deposit.js'
import claim_rewards from './users/claim_rewards.js'
import end_lottery from './admins/end_lottery.js'

async function allinone () {
  for (let i = 0; i < 100; i++) {
    setTimeout(() => {
      //C - 1 second later
    }, 5000)
    await deposit(10000)
    await end_lottery()
    await claim_rewards()
  }
}

export default allinone
