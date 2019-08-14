#![no_std]
extern crate pwasm_std;

use dusk_abi::{self, U256};

#[no_mangle]
pub fn deploy() {
    dusk_abi::set_storage(&U256::max(), &U256::max());
    // pwasm_std::logger::debug("abcd");
}
