#![no_std]
#![feature(proc_macro_hygiene)]
#[no_mangle]
use cake_rusk as cake;

#[cake::contract(version = "0.0.1")]
mod storage_factorial {
    use cake_rusk::address;
    use dusk_abi::H256;

    const FACTORIAL: [u8; 32] = address!(
        "6bfdaf2e75d5b0613a60cb0c3c7b7bb05c402d36828ddbd4ac8099d0bd4af099"
    );

    const STORAGE: [u8; 32] = address!(
        "a11d39fb84deb4eed1037c5ab50640bcd8d8de00cbfe2b534888bc12544057c6"
    );

    const FACTORIAL_OF: u8 = 1;
    const SET_VALUE: u8 = 3;
    const GET_VALUE: u8 = 1;

    pub fn factorial(n: u64) -> i32 {
        let f = if n <= 1 {
            0
        } else {
            n * dusk_abi::call_contract_operation::<u64, u64>(
                &H256::from_bytes(&FACTORIAL),
                FACTORIAL_OF,
                0,
                n - 1,
            )
        };

        dusk_abi::call_contract_operation::<i32, i32>(
            &H256::from_bytes(&STORAGE),
            SET_VALUE,
            0,
            f as i32,
        );
        1 // success
    }

    pub fn get_value() -> i32 {
        dusk_abi::call_contract_operation::<i32, i32>(
            &H256::from_bytes(&STORAGE),
            GET_VALUE,
            0,
            0,
        )
    }
}