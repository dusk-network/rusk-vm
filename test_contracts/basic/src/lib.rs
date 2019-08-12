#![no_std]
extern crate pwasm_std;

#[no_mangle]
pub fn entry() {
    pwasm_std::logger::debug("abcd");
}
