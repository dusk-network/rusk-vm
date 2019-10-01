#![no_std]
use dusk_abi::{self, encoding, Error, Signature, H256};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[no_mangle]
static PUBLIC_KEY: [u8; 32] = [0u8; 32];

lazy_static! {
    static ref NONCE_KEY: H256 = { H256::zero() };
}

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
    let mut args = [0u8; 256];

    // read the arguments into buffer
    dusk_abi::call_data(&mut args);
    dusk_abi::debug("a");
    let call: AccountCall = encoding::decode(&args).unwrap();
    dusk_abi::debug("b");
    panic!("{:?}", call);
}

#[no_mangle]
pub fn deploy() {
    // Set the initial nonce to zero
    dusk_abi::set_storage(&NONCE_KEY, &H256::zero())
}
