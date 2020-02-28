#![no_std]
use dusk_abi::{self, encoding, ContractCall, CALL_DATA_SIZE, H256};
use phoenix_abi::{Item, ITEM_SIZE, MAX_NOTES_PER_TRANSACTION};
use serde::{Deserialize, Serialize};

// Interface
pub fn transfer(notes: Item) -> ContractCall<Item> {
    ContractCall::new(notes).unwrap()
}

#[no_mangle]
pub fn call() {
    let mut buffer = [0u8; CALL_DATA_SIZE];
    let notes: Item = dusk_abi::call_data(&mut buffer);

    // assert!(notes.len() <= MAX_NOTES_PER_TRANSACTION);
    let mut notes_buf = [0u8; MAX_NOTES_PER_TRANSACTION * ITEM_SIZE];
    encoding::encode(&notes, &mut notes_buf).expect("buffer insufficient");
    phoenix_abi::store(&notes_buf);
    dusk_abi::ret(true);
}
