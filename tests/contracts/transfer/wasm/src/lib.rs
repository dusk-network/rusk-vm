#![no_std]
use dusk_abi::{self, ContractCall, CALL_DATA_SIZE, H256};
use phoenix_abi::{Item, Proof, ITEM_SIZE};
use phoenix_lib::MAX_NOTES_PER_TRANSACTION;

// Interface
pub fn transfer(notes: &[Item], proof: Proof) -> ContractCall<&[Item], Proof> {
    ContractCall::new(&(notes, proof)).unwrap()
}

#[no_mangle]
pub fn call() {
    let mut buffer = [0u8; CALL_DATA_SIZE];
    let (notes, proof): (&[Item], Proof) = dusk_abi::call_data(&mut buffer);

    assert!(notes.len() <= MAX_NOTES_PER_TRANSACTION);
    let mut notes_buf = [0u8; MAX_NOTES_PER_TRANSACTION * ITEM_SIZE];
    let encoded =
        encoding::encode(notes, &mut notes_buf).expect("buffer insufficient");
    phoenix_abi::store(encoded, proof);
    dusk_abi::ret(true);
}
