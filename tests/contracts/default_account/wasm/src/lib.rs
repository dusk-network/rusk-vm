#![no_std]
use dusk_abi::{self, encoding, ContractCall, Signature, CALL_DATA_SIZE, H256};
use serde::{Deserialize, Serialize};

const NONCE: [u8; 1] = [0u8];

#[no_mangle]
static PUBLIC_KEY: [u8; 32] = [0u8; 32];

#[derive(Serialize, Deserialize, Debug)]
pub enum AccountCall<'a> {
    CallThrough {
        to: H256,
        amount: u128,
        call_data: &'a [u8],
        nonce: u64,
        signature: Signature,
    },
    Balance,
}

pub fn call() {
    let mut buffer = [0u8; CALL_DATA_SIZE];
    let data: AccountCall = dusk_abi::call_data(&mut buffer);
    match data {
        // Transfer funds and call through to another contract
        AccountCall::CallThrough {
            to,
            amount,
            nonce,
            signature,
            call_data,
            ..
        } => {
            let current_nonce = dusk_abi::get_storage(&NONCE).unwrap();

            assert!(nonce == current_nonce);

            let mut verify_buf = [0u8; 32 + 16 + 8];
            let encoded =
                encoding::encode(&(to, amount, nonce), &mut verify_buf)
                    .expect("buffer insufficient");

            if dusk_abi::verify_ed25519_signature(
                &PUBLIC_KEY,
                &signature,
                encoded,
            ) {
                let call = ContractCall::<()>::new_raw(call_data);
                dusk_abi::call_contract(&to, amount, &call);
                dusk_abi::set_storage(&NONCE, current_nonce + 1);
            } else {
                panic!("invalid signature!");
            }
        }
        // Return the account balance
        AccountCall::Balance => {
            let balance = dusk_abi::balance();
            dusk_abi::ret(balance);
        }
    }
}

pub fn deploy() {
    // Set the initial nonce to zero
    dusk_abi::set_storage(&NONCE, 0u64)
}
