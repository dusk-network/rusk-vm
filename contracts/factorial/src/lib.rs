#![no_std]
use dusk_abi::{self, ContractCall, CALL_DATA_SIZE};

// Interface
pub fn factorial(of: u64) -> ContractCall<u64> {
    ContractCall::new(of).unwrap()
}

#[no_mangle]
pub fn call() {
    let mut buffer = [0u8; CALL_DATA_SIZE];
    let input: u64 = dusk_abi::call_data(&mut buffer);

    let self_hash = dusk_abi::self_hash();

    if input < 2 {
        dusk_abi::ret(input);
    } else {
        let result = dusk_abi::call_contract(
            &self_hash,
            0,
            &mut factorial(
                input
                    * dusk_abi::call_contract(
                        &self_hash,
                        0,
                        &mut factorial(input - 1),
                    ),
            ),
        );
        dusk_abi::ret(result);
    }
}

#[no_mangle]
pub fn deploy() {
    // Set the initial nonce to zero
    dusk_abi::set_storage("nonce", 0u64)
}
