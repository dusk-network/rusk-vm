#![no_std]
use dusk_abi::{self, TransferCall};

// TODO: obfuscated approve and transferfrom
// TODO: proof verification

#[no_mangle]
pub fn call() {
    let data: TransferCall = dusk_abi::args();
    match data {
        TransferCall::Transfer { nullifiers, notes } => {
            if !phoenix_abi::verify(&nullifiers, &notes) {
                dusk_abi::ret(0);
            }
            dusk_abi::ret(phoenix_abi::store(&nullifiers, &notes));
        }
        TransferCall::Approve {
            nullifiers,
            notes,
            pk,
            value,
        } => {
            if !phoenix_abi::verify(&nullifiers, &notes) {
                dusk_abi::ret(0);
            }

            phoenix_abi::store(&nullifiers, &notes);
            dusk_abi::set_storage(&pk, value);
            dusk_abi::ret(1);
        }
        TransferCall::TransferFrom {
            sender,
            recipient,
            value,
        } => {
            let approved_value = dusk_abi::get_storage(&sender).unwrap();
            if value > approved_value {
                dusk_abi::ret(0);
            }

            dusk_abi::set_storage(&sender, approved_value - value);
            phoenix_abi::credit(value, &recipient);
            dusk_abi::ret(1);
        }
    }
}
