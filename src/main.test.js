//const { test } = require("shelljs")

beforeAll(async function () {
  // NOTE: nearlib and nearConfig are made available by near-cli/test_environment
  const near = await nearlib.connect(nearConfig)
  window.accountId = nearConfig.contractName
  window.contract = await near.loadContract(nearConfig.contractName, {
    viewMethods: ['get_prize_pool', 'get_lotto_list', 'get_lotto'],
    changeMethods: ['make_rand', 'new', 'enter_draw', 'add_lotto'],
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
  await window.contract.add_lotto({owner_id: window.accountId, entry_fee: 1, close_date_time: 1638965117})
  let list = await window.contract.get_lotto_list()
  console.log("number of loteries: ", list)
  let l1 = await window.contract.get_lotto({lotto_id:0})
  console.log(l1)
  //await window.contract.enter_draw(0)
  console.log(await window.contract.get_prize_pool({lotto_id:0}))
});
// test('get_prize_pool', async () => {
//   const contract = await window.contract.new({owner_id: window.accountId})
//   //const rand = await contract.makeRand()
//   const message = await contract.get_prize_pool()
//   console.log(rand)
//   //expect(message)
// });
