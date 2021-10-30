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
use near_sdk::{env, near_bindgen, setup_alloc, Promise, AccountId, Balance,json_types::{ U128, Base58PublicKey }};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::serde::Serialize;
setup_alloc!();

const ONE_NEAR:u128 = 1_000_000_000_000_000_000_000_000;
const max_entries:u64 = 18446744073709551615;


// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearLotto {
    owner_id: AccountId,
    entries: UnorderedMap<u64, AccountId>,//Vector<AccountId>, 
    entry_fee: Balance,
    prize_pool: Balance, 
    winner: AccountId, 
    closed: bool
}

impl Default for NearLotto {
  fn default() -> Self {
      Self {
        owner_id: env::current_account_id(),
        entries: UnorderedMap::new(b"entries".to_vec()), //Vector::new(b'e'),
        entry_fee: ONE_NEAR, 
        prize_pool: 0,
        winner: "".to_string(),
        closed: false
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
            entries: UnorderedMap::new(b"entries".to_vec()),//Vector::new(b'e'),
            entry_fee: ONE_NEAR, 
            prize_pool: 0,
            winner: "".to_string(),
            closed: false 
        }

    }

    #[payable]
    pub fn get_attached(&mut self) -> Balance {
        env::attached_deposit()
    } 

    #[payable]
    pub fn enter_draw(&mut self){
        let attached = env::attached_deposit();
        assert!(attached >= self.entry_fee, "Entry fee not enough");
        assert!(self.entries.len() < max_entries, "Entries are full");
        env::log(format!("money matches, add entry").as_bytes());
        self.prize_pool = self.prize_pool + (env::attached_deposit()/4)*3;
        let k = self.entries.len();
        self.entries.insert(&k, &near_sdk::env::signer_account_id());
        env::log(format!("{} Entering the lottery", env::signer_account_id()).as_bytes());
    }

    pub fn pick_winner(&mut self){
        let rand_array = [*env::random_seed().get(0).unwrap(),*env::random_seed().get(2).unwrap(),*env::random_seed().get(3).unwrap(), *env::random_seed().get(4).unwrap(),*env::random_seed().get(5).unwrap()];
        let len:u128 = self.entries.len() as u128;
        let rand = (rand_array[0] + rand_array[1] + rand_array[2] + rand_array[3]+ rand_array[4]) as u128;
        let keys = self.entries.keys_as_vector();
        let win_key = keys.get((rand%len)as u64);
        let winner;
        match win_key {
            Some(x) => winner = self.entries.get(&x),
            None => panic!("Arh no winner")
        }
        match winner{
            Some(x) => self.winner = x,
            None => panic!(" not got a winner")
        }
        let win = &self.winner;
        assert!(win != "", "No winnner lets get out of here");
        self.closed = true;
        Promise::new(win.to_string()).transfer(self.prize_pool);
    }

    pub fn get_closed(self) -> bool {
        self.closed 
    }

    pub fn get_winner(self) -> AccountId {
        self.winner
    }

    pub fn get_prize_pool(self) -> Balance{
        return self.prize_pool;
    }

    pub fn get_entriies(self) -> UnorderedMap<u64,AccountId> {
        self.entries
    }

    // `match` is similar to `switch` in other languages; here we use it to default to "Hello" if
    // self.records.get(&account_id) is not yet defined.
    // Learn more: https://doc.rust-lang.org/book/ch06-02-match.html#matching-with-optiont

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
            attached_deposit: ONE_NEAR,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn enter_the_draw() {
        let mut context = get_context(vec![], false);
        context.attached_deposit = ONE_NEAR;
        testing_env!(context);
        let mut contract = NearLotto::default();
        contract.enter_draw();
    }

    #[test]
    fn get_the_prize_pool() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = NearLotto::default();
        let prize = contract.get_prize_pool();
        println!("the Prize is: {}", prize);
    }

    #[test]
    fn get_the_entires() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = NearLotto::default();
        let entries = contract.get_entriies();
        println!("the Entries are: {:?}", entries.values_as_vector());
    }
}
