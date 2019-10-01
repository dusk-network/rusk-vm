#![cfg_attr(not(feature = "std"), no_std)]
#![feature(lang_items)]
#![feature(panic_info_message)]

pub use serde::{Deserialize, Serialize};

pub mod encoding;
mod panic;
mod types;

pub use types::{Signature, H256};

// TODO: Extend this error type
pub use fermion::Error;

// declare available host-calls
mod external {
    use super::H256;
    extern "C" {
        pub fn set_storage(key: &H256, value: &H256);
        pub fn caller(buffer: &mut [u8; 32]);
        pub fn balance(buffer: &mut [u8; 32]);
        pub fn debug(text: &str);
        pub fn panic(msg: &[u8]) -> !;
        pub fn call_data(buffer: &mut [u8]);
        pub fn ret(data: &[u8]);
    }
}

// implementations
pub fn set_storage(key: &H256, val: &H256) {
    unsafe {
        external::set_storage(key, val);
    }
}

// implementations
pub fn debug(s: &str) {
    unsafe {
        external::debug(s);
    }
}

pub fn caller() -> H256 {
    let mut buffer = [0u8; 32];
    unsafe { external::caller(&mut buffer) }
    encoding::decode(&buffer[..]).unwrap()
}

pub fn balance() -> H256 {
    let mut buffer = [0u8; 32];
    unsafe { external::balance(&mut buffer) }
    encoding::decode(&buffer[..]).unwrap()
}

pub fn call_data(buffer: &mut [u8]) {
    unsafe { external::call_data(buffer) }
}

pub fn ret<T: Serialize>(_ret: T) {
    unimplemented!("ret")
}
