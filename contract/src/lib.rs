
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, setup_alloc, Promise, AccountId, Balance,json_types::{ U128, Base58PublicKey }};
use near_sdk::collections::{ Vector};
//use near_sdk::serde::Serialize;
setup_alloc!();
//#[global_allocator]
//static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const ONE_NEAR:u128 = 1_000_000_000_000_000_000_000_000;
const MAX_ENTRIES:u64 = 18446744073709551615;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearLotto {
    owner_id: AccountId,
    entries: Vector<AccountId>, //UnorderedMap<u64, AccountId>,
    entry_fee: Balance,
    prize_pool: Balance, 
    winner: AccountId, 
    closed: bool, 
    rand: u128
}

impl Default for NearLotto {
    fn default() -> Self {
        panic!("Should be initialized before usage")
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
            entries: Vector::new(b'e'),  //UnorderedMap::new(b"entries".to_vec()),
            entry_fee: ONE_NEAR, 
            prize_pool: ONE_NEAR,
            winner: "".to_string(),
            closed: false, 
            rand: 78
        }
    }

    #[payable]
    pub fn get_attached(&mut self) -> Balance {
        env::attached_deposit()
    } 

    #[payable]
    pub fn enter_draw(&mut self){
        // charge some storage fee 
        let attached = env::attached_deposit();
        assert!(attached >= self.entry_fee, "Entry fee not enough");
        assert!(self.entries.len() < MAX_ENTRIES, "Entries are full");
        env::log(format!("money matches, add entry").as_bytes());
        self.prize_pool = self.prize_pool + (env::attached_deposit()/4)*3; // 75% of entries fees goes into prize pool 
        
        self.entries.push(&env::signer_account_id()); //self.entries.insert(&k, &near_sdk::env::signer_account_id());
        env::log(format!("{} Entering the lottery", env::signer_account_id()).as_bytes());
    }

    pub fn pick_winner(&mut self){
        assert!(env::signer_account_id() == self.owner_id, "Not the contract owner so stop right there");
        //let rand_array = [*env::random_seed().get(0).unwrap_or(&0),*env::random_seed().get(2).unwrap_or(&0),*env::random_seed().get(3).unwrap_or(&0), *env::random_seed().get(4).unwrap_or(&0),*env::random_seed().get(5).unwrap_or(&0)];
        let len:u128 = self.entries.len() as u128;
        let rand: u128 = random_u128(); //(rand_array[0] + rand_array[1] + rand_array[2] + rand_array[3]+ rand_array[4]) as u128;
        self.rand = rand;
        println!("Rand is {:?}", rand);
        println!("len is {:?}", len);
        let mut i = (rand%len)as u64;
        if i == self.entries.len(){
            i = i-1
        }
        let win = self.entries.get(i);
        println!("win is {:?}", win);
        
        assert!(!win.is_none(), "No winnner lets get out of here");
        self.closed = true;
        match win {
            Some(x) => self.winner = x,
            None => panic!("No winners lets go")
        }
        assert!(!self.winner.is_empty(),"No winner WTF");
        println!("the winner is {:?}", self.winner);
        Promise::new(self.winner.to_string()).transfer(self.prize_pool);
    }

    // pub fn winings_transfer_manual(self, winner: AccountId) {
    //     //only account owner. 
    // }

    pub fn collect_charity(self, out:AccountId){
        //owner only function
        assert!(env::signer_account_id() == self.owner_id, "Not the contract owner so stop right there");
        // calculate the stortage and minus from the balance with some buffer. 
        Promise::new(out).transfer(env::account_balance());
    }

    pub fn make_rand(&mut self)->u128{
        self.rand = random_u128();
        self.rand
    }

    pub fn get_closed(self) -> bool {
        self.closed 
    }

    pub fn get_winner(self) -> AccountId {
        self.winner
    }

    pub fn get_prize_pool(self) -> Balance{
        self.prize_pool
    }

    pub fn get_entries(self) -> u64 {
        self.entries.len()
    }

    // `match` is similar to `switch` in other languages; 
    // Learn more: https://doc.rust-lang.org/book/ch06-02-match.html#matching-with-optiont
}

fn random_u128() -> u128 {
    let random_seed = env::random_seed(); // len 32
    println!("Random seed is {:?}", random_seed.to_vec());
    // using first 16 bytes (doesn't affect randomness)
    as_u128(random_seed.get(..16).unwrap())
}

fn as_u128( arr: &[u8]) -> u128 {
    ((arr[0] as u128) << 0) +
    ((arr[1] as u128) << 8) +
    ((arr[2] as u128) << 16) +
    ((arr[3] as u128) << 24) +
    ((arr[4] as u128) << 32) +
    ((arr[5] as u128) << 40) +
    ((arr[6] as u128) << 48) +
    ((arr[7] as u128) << 56) +
    ((arr[8] as u128) << 64) +
    ((arr[9] as u128) << 72) +
    ((arr[10] as u128) << 80) +
    ((arr[11] as u128) << 88) +
    ((arr[12] as u128) << 96) +
    ((arr[13] as u128) << 104) +
    ((arr[14] as u128) << 112) +
    ((arr[15] as u128) << 120)
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

    // #[test]
    // fn make_rand(){
    //     let mut context = get_context(vec![], false);
    //     context.attached_deposit = ONE_NEAR;
    //     testing_env!(context);
    //     let mut contract = NearLotto::new(env::signer_account_id());
    //     contract.make_rand();
    // }

    #[test]
    fn enter_the_draw() {
        let mut context = get_context(vec![], false);
        context.attached_deposit = ONE_NEAR;
        testing_env!(context);
        let mut contract = NearLotto::new(env::signer_account_id());
        contract.enter_draw();
    }

    #[test]
    fn get_the_prize_pool() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let contract = NearLotto::new(env::signer_account_id());
        let prize = contract.get_prize_pool();
        println!("the Prize is: {}", prize);
    }

    #[test]
    fn get_the_entires() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let contract = NearLotto::new(env::signer_account_id());
        let entries = contract.get_entries();
        println!("the Entries are: {:?}", entries);
    }
    #[test]
    fn new_enter_check_prize_entries(){
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = NearLotto::new(env::signer_account_id());
        println!("Enter");
        contract.enter_draw();
        // let prize = contract.get_prize_pool();
        // println!("the Prize is: {}", prize);
        let entries = contract.get_entries();
        println!("the Entries are: {:?}", entries);
    }
    // #[test]
    // fn new_enter_winner(){
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     let mut contract = NearLotto::new(env::signer_account_id());
    //     println!("Enter");
    //     contract.enter_draw();
    //     // let prize = contract.get_prize_pool();
    //     // println!("the Prize is: {}", prize);
    //     //contract.pick_winner();
    //     contract.make_rand();
    //     println!("The winner is: {:?}", contract.winner);
    //     println!("They have won: {:?}", contract.prize_pool);
    //     println!("The rand is {:?}", contract.rand);
    // }
}   
