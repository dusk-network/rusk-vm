#![no_std]
#![feature(lang_items)]
#![feature(panic_info_message)]

pub use serde::{Deserialize, Serialize};
use ssmarshal;

pub mod encoding;
mod panic;
mod signature;
mod u256;

pub use signature::Signature;
pub use u256::U256;

// declare available host-calls
mod external {
    use super::U256;
    #[rustfmt::skip]
    extern {
        pub fn set_storage(key: *const U256, value: *const U256);
        pub fn caller(buffer: &mut [u8; 32]);
        pub fn balance(buffer: &mut [u8; 32]);
        pub fn debug(text: &str);
        pub fn panic(msg: &[u8]) -> !;
        pub fn args(buffer: &mut [u8]);
        pub fn ret(data: &[u8]);
    }
}

// implementations
pub fn set_storage(key: &U256, val: &U256) {
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

pub fn caller() -> U256 {
    let mut buffer = [0u8; 32];
    unsafe { external::caller(&mut buffer) }
    ssmarshal::deserialize(&buffer[..]).unwrap().0
}

pub fn balance() -> U256 {
    let mut buffer = [0u8; 32];
    unsafe { external::balance(&mut buffer) }
    ssmarshal::deserialize(&buffer[..]).unwrap().0
}

pub fn args(buffer: &mut [u8]) {
    unsafe { external::args(buffer) }
}

pub fn ret<T: Serialize>(_ret: T) {
    unimplemented!()
}
