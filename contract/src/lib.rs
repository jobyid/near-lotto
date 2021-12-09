
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, setup_alloc, Promise, AccountId, Balance,json_types::{ U128, Base58PublicKey }};
use near_sdk::collections::{ Vector, UnorderedMap};
use near_sdk::serde::{Serialize, Deserialize};
//use near_sdk::serde::Serialize;
setup_alloc!();
//#[global_allocator]
//static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const ONE_NEAR:u128 = 1_000_000_000_000_000_000_000_000;
const MAX_ENTRIES:u64 = 18446744073709551615;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LottoList{
    owner_id: AccountId,
    lotteries: UnorderedMap<u64,NearLotto>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NearLotto {
    lotto_id: u64,
    owner_id: AccountId,
    entries: Vec<AccountId>, //UnorderedMap<u64, AccountId>,
    entry_fee: Balance,
    prize_pool: Balance, 
    climate_pool: Balance,
    winner: AccountId, 
    closed: bool, 
    rand: u128, 
    close_date_time: u64
}

impl Default for LottoList {
    fn default() -> Self {
        panic!("Should be initialized before usage")
    }
}

#[near_bindgen]
impl LottoList {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Invalid owner account");
        assert!(!env::state_exists(), "Already initialized");
        
        Self {
            owner_id,
            lotteries: UnorderedMap::new(b'e')
        } 
    }

    pub fn add_lotto(&mut self, owner_id: AccountId, entry_fee:u8, close_date_time: u64){
        assert!(self.owner_id == env::signer_account_id(), "Only account owner cna make more lottos");
        let lotto = NearLotto {
            lotto_id: self.lotteries.len(),
            owner_id,
            entries: Vec::new(),  //UnorderedMap::new(b"entries".to_vec()),
            entry_fee: (entry_fee as u128) * ONE_NEAR, 
            prize_pool: 0,
            climate_pool:0,
            winner: "".to_string(),
            closed: false, 
            rand: 78,
            close_date_time: close_date_time
        };
        self.lotteries.insert(&self.lotteries.len(),&lotto);
    }

    #[payable]
    pub fn get_attached(&mut self) -> Balance {
        env::attached_deposit()
    } 

    #[payable]
    pub fn enter_draw(&mut self, lotto_id:u32){
        // charge some storage fee 
        let mut lotto = self.lotteries.get(&(lotto_id as u64)).unwrap();
        let attached = env::attached_deposit();
        assert!(lotto.closed == false, "Lotto closed");
        assert!(attached >= lotto.entry_fee, "Entry fee not enough");
        assert!(lotto.entries.len() < MAX_ENTRIES as usize, "Entries are full");
        env::log(format!("money matches, add entry").as_bytes());
        lotto.prize_pool = lotto.prize_pool + (env::attached_deposit()/4)*3; // 75% of entries fees goes into prize pool 
        lotto.climate_pool = lotto.climate_pool + (env::attached_deposit()/4)*3; //25% of entries go to climate change
        lotto.entries.push(env::signer_account_id()); //self.entries.insert(&k, &near_sdk::env::signer_account_id());
        env::log(format!("{} Entering the lottery", env::signer_account_id()).as_bytes());
        self.lotteries.insert(&(lotto_id as u64), &lotto);
    }

    pub fn pick_winner(&mut self, lotto_id:u32) -> String{
        //assert close date is before now 

        assert!(env::signer_account_id() == self.owner_id, "Not the contract owner so stop right there");
        let mut lotto = self.lotteries.get(&(lotto_id as u64)).unwrap();
        let now = env::block_timestamp();
        assert!(lotto.close_date_time < now, "Wait until cloase date and time is passed");
        let len:u128 = lotto.entries.len() as u128;
        let rand: u128 = random_u128(); //(rand_array[0] + rand_array[1] + rand_array[2] + rand_array[3]+ rand_array[4]) as u128;
        lotto.rand = rand;
        println!("Rand is {:?}", rand);
        println!("len is {:?}", len);
        let mut i = (rand%len)as u64;
        if i == lotto.entries.len() as u64{
            i = i-1
        }
        let win = lotto.entries.get(i as usize);
        println!("win is {:?}", win);
        
        assert!(!win.is_none(), "No winnner lets get out of here");
        lotto.closed = true;
        match win {
            Some(x) => lotto.winner = x.to_string(),
            None => panic!("No winners lets go")
        }
        assert!(!lotto.winner.is_empty(),"No winner WTF");
        println!("the winner is {:?}", lotto.winner);
        Promise::new(lotto.winner.to_string()).transfer(lotto.prize_pool);
        self.lotteries.insert(&(lotto_id as u64), &lotto);
        lotto.winner
    }

    // pub fn winings_transfer_manual(self, winner: AccountId) {
    //     //only account owner. 
    // }

    pub fn collect_charity(self, out:AccountId, lotto_id:u32){
        //owner only function
        let lotto = self.lotteries.get(&(lotto_id as u64)).unwrap();
        assert!(lotto.closed == true, "Lotto not closed yet");
        assert!(env::signer_account_id() == self.owner_id, "Not the contract owner so stop right there");
        // calculate the storage and minus from the balance with some buffer. 
        Promise::new(out).transfer(lotto.climate_pool); // saving 3% for storage 
    }

    // pub fn make_rand(&mut self)->u128{
    //     self.rand = random_u128();
    //     self.rand
    // }

    pub fn get_closed(self, lotto_id: u32) -> bool {
        let lotto = self.lotteries.get(&(lotto_id as u64)).unwrap();
        lotto.closed 
    }

    pub fn get_winner(self, lotto_id: u32) -> String {
        let lotto = self.lotteries.get(&(lotto_id as u64)).unwrap();
        lotto.winner.to_string()
    }

    pub fn get_prize_pool(self, lotto_id: u32) -> Balance{
        let lotto = self.lotteries.get(&(lotto_id as u64)).unwrap();
        lotto.prize_pool
    }

    pub fn get_entries(self, lotto_id: u32) -> u64 {
        let lotto = self.lotteries.get(&(lotto_id as u64)).unwrap();
        lotto.entries.len() as u64
    }
    pub fn get_lotto_list(self) -> u64 {//UnorderedMap<u64,NearLotto>{
        self.lotteries.len()
    }
    pub fn get_lotto(self, lotto_id: u32) -> NearLotto{
        let lotto = self.lotteries.get(&(lotto_id as u64)).unwrap();
        return lotto
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
    fn create_lotto_and_enter_the_draw() {
        let mut context = get_context(vec![], false);
        context.attached_deposit = ONE_NEAR;
        testing_env!(context);
        let mut contract = LottoList::new(env::signer_account_id());
        contract.add_lotto(env::signer_account_id(), 1, 1638965117);
        contract.enter_draw(0);
        //let winner = contract.pick_winner(0);
        let prize = contract.get_prize_pool(0);
        println!("the Prize is: {}", prize);
        //println!("the winner is: {}", winner);
    }

    // #[test]
    // fn get_the_prize_pool() {
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     let contract = NearLotto::new(env::signer_account_id());
    //     let prize = contract.get_prize_pool();
    //     println!("the Prize is: {}", prize);
    // }

    // #[test]
    // fn get_the_entires() {
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     let contract = NearLotto::new(env::signer_account_id());
    //     let entries = contract.get_entries();
    //     println!("the Entries are: {:?}", entries);
    // }
    // #[test]
    // fn new_enter_check_prize_entries(){
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     let mut contract = NearLotto::new(env::signer_account_id());
    //     println!("Enter");
    //     contract.enter_draw();
    //     // let prize = contract.get_prize_pool();
    //     // println!("the Prize is: {}", prize);
    //     let entries = contract.get_entries();
    //     println!("the Entries are: {:?}", entries);
    // }
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
