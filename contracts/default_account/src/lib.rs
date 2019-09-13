#![no_std]
use dusk_abi::{self, encoding, Signature, U256};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[no_mangle]
static PUBLIC_KEY: [u8; 32] = [
    31, 219, 211, 229, 83, 65, 117, 109, 186, 109, 192, 104, 149, 115, 224,
    125, 78, 64, 1, 64, 61, 111, 119, 247, 103, 98, 75, 151, 220, 85, 33, 159,
];

lazy_static! {
    static ref NONCE: U256 = { U256::zero() };
}

#[derive(Serialize, Deserialize)]
enum AccountCall<'a> {
    CallThrough {
        to: U256,
        amount: u128,
        call_data: &'a [u8],
        signature: Signature,
    },
    Balance,
}

#[no_mangle]
pub fn call() {
    // dusk_abi::debug("Who was phone?");
    // let mut args = [0u8; mem::size_of::<AccountCall>()];
    // dusk_abi::args(&mut args);
    // let _call: AccountCall = encoding::deserialize(&args);
}

#[no_mangle]
pub fn deploy() {
    // Set the initial nonce to zero

    dusk_abi::set_storage(&NONCE, &U256::zero())
}
