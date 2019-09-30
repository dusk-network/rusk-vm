#![no_std]
use core::mem;

use dusk_abi::{
    self, encoding,
    types::{Signature, H256},
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[no_mangle]
static PUBLIC_KEY: [u8; 32] = [
    31, 219, 211, 229, 83, 65, 117, 109, 186, 109, 192, 104, 149, 115, 224,
    125, 78, 64, 1, 64, 61, 111, 119, 247, 103, 98, 75, 151, 220, 85, 33, 159,
];

lazy_static! {
    static ref NONCE_KEY: H256 = { H256::zero() };
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AccountCall<'a> {
    CallThrough {
        to: H256,
        amount: u128,
        call_data: &'a [u8],
        signature: Signature,
    },
    Balance,
}

#[no_mangle]
pub fn call() {
    dusk_abi::debug("default contract called");
    let mut args = [0u8; 256];

    // read the arguments into buffer
    dusk_abi::call_data(&mut args);
    dusk_abi::debug("a");
    let call: AccountCall = encoding::decode(&args).unwrap();
    dusk_abi::debug("b");
    panic!("hehe {:?}", call);

    // let nonce = dusk_abi::get_storage(&NONCE_KEY);

    // // xoring hashes, crypto review needed ;)
    // let digest = call.to.digest()
    //     ^ call.call_data.digest()
    //     ^ call.amount.digest()
    //     ^ nonce.digest();

    // if dusk_abi::verify_signature(call.signature, PUBLIC_KEY, digest) {
    //     dusk_abi::set_storage(&NONCE_KEY, nonce + 1);
    //     dusk_abi::call(call.to, amount, call_data);
    // }
}

#[no_mangle]
pub fn deploy() {
    // Set the initial nonce to zero
    dusk_abi::set_storage(&NONCE_KEY, &H256::zero())
}
