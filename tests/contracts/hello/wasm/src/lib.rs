#![no_std]
#[allow(unused)]
use dusk_abi;

#[no_mangle]
pub fn call() {
    dusk_abi::debug!("Hello world!");
}
