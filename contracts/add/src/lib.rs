#![no_std]
use dusk_abi::{self, ContractCall, CALL_DATA_SIZE};

// Interface
pub fn add(a: u32, b: u32) -> ContractCall<u32> {
    ContractCall::new(&[a, b]).unwrap()
}

#[no_mangle]
pub fn call() {
    let mut buffer = [0u8; CALL_DATA_SIZE];
    let (a, b): (u32, u32) = dusk_abi::call_data(&mut buffer);
    let _self_hash = dusk_abi::self_hash();
    dusk_abi::ret(a + b);
}
