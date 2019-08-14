#![no_std]
extern crate pwasm_std;

#[no_mangle]
pub fn deploy() {
    pwasm_std::logger::debug("abcd");
}
