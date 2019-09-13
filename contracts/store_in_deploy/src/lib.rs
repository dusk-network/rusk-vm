#![no_std]
use dusk_abi::{self, U256};

#[no_mangle]
pub fn call() {
    let val = U256::max_value() - 1;
    dusk_abi::set_storage(&val, &val);
}

#[no_mangle]
pub fn deploy() {
    let max = &U256::max_value();
    dusk_abi::set_storage(max, max);
}
