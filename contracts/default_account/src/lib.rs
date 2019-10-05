#![no_std]
use dusk_abi::{self, encoding, Signature, H256, MAX_CALL_DATA_SIZE};
use serde::{Deserialize, Serialize};

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

#[no_mangle]
pub fn call() {
    let mut buffer = [0u8; MAX_CALL_DATA_SIZE];
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
            let current_nonce = dusk_abi::get_storage("nonce").unwrap();

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
                dusk_abi::call_contract(&to, amount, &call_data);
                dusk_abi::set_storage("nonce", current_nonce + 1);
            } else {
                panic!("invalid signature!");
            }
        }
        // Return the account balance
        Balance => {
            let balance = dusk_abi::balance();
            dusk_abi::ret(balance);
        }
    }
}

#[no_mangle]
pub fn deploy() {
    // Set the initial nonce to zero
    dusk_abi::set_storage("nonce", 0u64)
}
