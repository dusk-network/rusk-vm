#![no_std]
use dusk_abi::{self, ContractCall};
use phoenix_abi::{Note, Nullifier};

// Interface
pub fn transfer(
    nullifiers: [Nullifier; Nullifier::MAX],
    notes: [Note; Note::MAX],
) -> ContractCall<bool> {
    ContractCall::new((nullifiers, notes)).unwrap()
}

#[no_mangle]
pub fn call() {
    let (nullifiers, notes): ([Nullifier; Nullifier::MAX], [Note; Note::MAX]) =
        dusk_abi::args();

    if !phoenix_abi::verify(&nullifiers, &notes) {
        dusk_abi::ret(0);
    }
    dusk_abi::ret(phoenix_abi::store(&nullifiers, &notes));
}
