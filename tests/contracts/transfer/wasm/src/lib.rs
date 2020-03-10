#![no_std]
use dusk_abi::{self, encoding, ContractCall, CALL_DATA_SIZE, H256};
use phoenix_abi::{
    types::{
        MAX_NOTES_PER_TRANSACTION, MAX_NULLIFIERS_PER_TRANSACTION, NOTE_SIZE,
        NULLIFIER_SIZE,
    },
    Note, NotesBuffer, Nullifier, NullifiersBuffer,
};
use serde::{Deserialize, Serialize};

// Interface
pub fn transfer(
    nullifiers: [Nullifier; MAX_NULLIFIERS_PER_TRANSACTION],
    notes: [Note; MAX_NOTES_PER_TRANSACTION],
) -> ContractCall<bool> {
    ContractCall::new((nullifiers, notes)).unwrap()
}

#[no_mangle]
pub fn call() {
    let mut buffer = [0u8; CALL_DATA_SIZE];
    let (nullifiers, notes): (
        [Nullifier; MAX_NULLIFIERS_PER_TRANSACTION],
        [Note; MAX_NOTES_PER_TRANSACTION],
    ) = dusk_abi::call_data(&mut buffer);

    let mut nullifiers_buf =
        [0u8; MAX_NULLIFIERS_PER_TRANSACTION * NULLIFIER_SIZE];
    encoding::encode(&nullifiers, &mut nullifiers_buf)
        .expect("buffer insufficient");
    let mut notes_buf = [0u8; MAX_NOTES_PER_TRANSACTION * NOTE_SIZE];
    encoding::encode(&notes, &mut notes_buf).expect("buffer insufficient");

    if !phoenix_abi::verify(&nullifiers_buf, &notes_buf) {
        dusk_abi::ret(0);
    }

    dusk_abi::ret(phoenix_abi::store(&nullifiers_buf, &notes_buf));
}
