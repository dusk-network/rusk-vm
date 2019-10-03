#![no_std]
use dusk_abi::{self, encoding, Signature, H256};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[no_mangle]
static PUBLIC_KEY: [u8; 32] = [0u8; 32];

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountCall<'a> {
    to: H256,
    amount: u128,
    call_data: &'a [u8],
    nonce: u64,
    signature: Signature,
}

impl<'a> AccountCall<'a> {
    pub fn new(
        to: H256,
        amount: u128,
        call_data: &'a [u8],
        nonce: u64,
        signature: Signature,
    ) -> Self {
        AccountCall {
            to,
            amount,
            call_data,
            nonce,
            signature,
        }
    }
}

#[no_mangle]
pub fn call() {
    dusk_abi::debug("callz");
    let mut args = [0u8; 1024];
    // read the arguments into buffer
    dusk_abi::call_data(&mut args);
    let call: AccountCall = encoding::decode(&args).unwrap();

    let current_nonce = dusk_abi::get_storage("nonce").unwrap();

    assert!(call.nonce == current_nonce);

    let mut verify_buf = [0u8; 32 + 16 + 8];
    let encoded =
        encoding::encode(&(call.to, call.amount, call.nonce), &mut verify_buf)
            .expect("buffer insufficient");

    if dusk_abi::verify_ed25519_signature(&PUBLIC_KEY, &call.signature, encoded)
    {
        dusk_abi::call_contract(&call.to, call.amount, &call.call_data);
        dusk_abi::set_storage("nonce", current_nonce + 1);
    } else {
        panic!("incorrect!");
    }
}

#[no_mangle]
pub fn deploy() {
    // Set the initial nonce to zero
    dusk_abi::debug("deployz");
    dusk_abi::set_storage("nonce", 0u64)
}
