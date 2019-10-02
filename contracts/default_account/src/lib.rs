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
    let mut args = [0u8; 256];
    dusk_abi::debug("called");
    // read the arguments into buffer
    dusk_abi::call_data(&mut args);
    dusk_abi::debug("a");
    let call: AccountCall = encoding::decode(&args).unwrap();

    let mut verify_buf = [0u8; 32 + 16 + 8];
    let encoded =
        encoding::encode(&(call.to, call.amount, call.nonce), &mut verify_buf)
            .expect("Buffer big enough");

    if dusk_abi::verify_ed25519_signature(&PUBLIC_KEY, &call.signature, encoded)
    {
        panic!("correct!");
    } else {
        panic!("incorrect!");
    }
}

#[no_mangle]
pub fn deploy() {
    // Set the initial nonce to zero
    dusk_abi::set_storage(&NONCE_KEY, &H256::zero())
}
