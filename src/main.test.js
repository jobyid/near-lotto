//const { test } = require("shelljs")

beforeAll(async function () {
  // NOTE: nearlib and nearConfig are made available by near-cli/test_environment
  const near = await nearlib.connect(nearConfig)
  window.accountId = nearConfig.contractName
  window.contract = await near.loadContract(nearConfig.contractName, {
    viewMethods: ['get_prize_pool'],
    changeMethods: ['make_rand', 'new', 'enter_draw'],
    sender: window.accountId
  })

  window.walletConnection = {
    requestSignIn() {
    },
    signOut() {
    },
    isSignedIn() {
      return true
    },
    getAccountId() {
      return window.accountId
    }
  }
  //await initContract(window.accountId)
});

// test('contract hash', async () => {
//   let state = (await new Account(connection, contractName)).state();
//   expect(state.code_hash).not.toEqual('11111111111111111111111111111111');
// });
test('new', async () => {
  let contract = await window.contract.new({owner_id: window.accountId})
  await window.contract.enter_draw()
  const new_lotto = await window.contract.get_prize_pool()
  console.log("Something seems to have happened")
  console.log(new_lotto)
});
// test('get_prize_pool', async () => {
//   const contract = await window.contract.new({owner_id: window.accountId})
//   //const rand = await contract.makeRand()
//   const message = await contract.get_prize_pool()
//   console.log(rand)
//   //expect(message)
// });
