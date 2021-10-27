/*
 * This is an example of a Rust smart contract with two simple, symmetric functions:
 *
 * 1. set_greeting: accepts a greeting, such as "howdy", and records it for the user (account_id)
 *    who sent the request
 * 2. get_greeting: accepts an account_id and returns the greeting saved for it, defaulting to
 *    "Hello"
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://github.com/near/near-sdk-rs
 *
 */

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, Balance};
use near_sdk::collections::{LookupMap, Vector};

setup_alloc!();

const ONE_NEAR:u128 = 1_000_000_000_000_000_000_000_000;


// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearLotto {
    owner_id: AccountId,
    records: LookupMap<String, String>,
    //entries: Vector<AccountId>, 
    entry_fee: Balance,
    prize_pool: Balance 
}

impl Default for NearLotto {
  fn default() -> Self {
      // records: LookupMap::new(b"a".to_vec()),
      Self {
        owner_id: env::current_account_id(),
        records: LookupMap::new(b"a".to_vec()),
        //entries: Vector::new(),
        entry_fee: ONE_NEAR, 
        prize_pool: 0,
    }
  }
}

#[near_bindgen]
impl NearLotto {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Invalid owner account");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id,
            records: LookupMap::new(b"a".to_vec()),
            //entries: Vector::new(),
            entry_fee: ONE_NEAR, 
            prize_pool: 0,
        }

    }

    #[payable]
    pub fn enter_draw(&mut self){
        let attached = env::attached_deposit();
        // if attached == self.entry_fee{
        //     env::log(format!("money mtaches add entry").as_bytes());
        //     self.prize_pool = self.prize_pool + (env::attached_deposit()/4)*3;
        //     //self.entries.push(&near_sdk::env::signer_account_id());
        // }else{
        //     panic!("Entry Fee not enough!!");
        // }
        env::log(format!("Entering the lottery").as_bytes());
    }

    pub fn get_prize_pool(self) -> Balance{
        return self.prize_pool;
    }

    pub fn set_greeting(&mut self, message: String) {
        let account_id = env::signer_account_id();

        // Use env::log to record logs permanently to the blockchain!
        env::log(format!("Saving greeting '{}' for account '{}'", message, account_id,).as_bytes());

        self.records.insert(&account_id, &message);
    }

    // `match` is similar to `switch` in other languages; here we use it to default to "Hello" if
    // self.records.get(&account_id) is not yet defined.
    // Learn more: https://doc.rust-lang.org/book/ch06-02-match.html#matching-with-optiont
    pub fn get_greeting(&self, account_id: String) -> String {
        match self.records.get(&account_id) {
            Some(greeting) => greeting,
            None => "Hello".to_string(),
        }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 15,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn set_then_get_greeting() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = NearLotto::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(
            "howdy".to_string(),
            contract.get_greeting("bob_near".to_string())
        );
    }

    #[test]
    fn get_default_greeting() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = NearLotto::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(
            "Hello".to_string(),
            contract.get_greeting("francis.near".to_string())
        );
    }

    #[test]
    fn enter_the_draw() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let mut contract = NearLotto::default();
        contract.enter_draw();
    }

    #[test]
    fn get_the_prize_pool(){
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = NearLotto::default();
        let prize = contract.get_prize_pool();
        println!("the Prize is: {}", prize)
    }
}
