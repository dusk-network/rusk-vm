#![no_std]
use dusk_abi::{self, ContractCall, CALL_DATA_SIZE};

// Interface
pub fn factorial(of: u64) -> ContractCall<u64> {
    ContractCall::new(of)
}

#[no_mangle]
pub fn call() {
    let n: u64 = dusk_abi::call_data();

    let self_hash = dusk_abi::self_hash();

    if n <= 1 {
        dusk_abi::ret(1);
    } else {
        dusk_abi::ret(
            n * dusk_abi::call_contract(&self_hash, 0, &factorial(n - 1)),
        );
    }
}
