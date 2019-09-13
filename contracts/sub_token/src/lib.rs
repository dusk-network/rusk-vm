#![no_std]
use dusk_abi::{self, dusk_derive, U256};

pub trait SubTokenAPI {
    fn balance(&self, account: &U256) -> Option<&U256>;
    fn transfer(&mut self, to: &U256);
}

#[allow(unused)]
struct SubToken;

impl SubToken {
    fn transfer(&mut self, to: &U256) {
        unimplemented!()
    }
}

// #[no_mangle]
// pub fn call() {
//     SubTokenAPI::new()
// }

#[no_mangle]
pub fn deploy() {
    dusk_abi::set_storage(&dusk_abi::caller(), &1_000_000_000.into());
}
