#![no_std]
extern crate pwasm_std;

use dusk_abi::{self, U256};

#[no_mangle]
pub fn deploy() {
    dusk_abi::set_storage(&U256::max_value(), &U256::max_value());
    // pwasm_std::logger::debug("abcd");
}

#[no_mangle]
pub fn call() {
    unimplemented!()
}
