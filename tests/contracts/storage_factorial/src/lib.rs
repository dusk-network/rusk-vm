// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![no_std]
#![feature(proc_macro_hygiene)]
#[no_mangle]
use cake_rusk as cake;

#[cake::contract(version = "0.0.1")]
mod storage_factorial {
    use cake_rusk::address;
    use dusk_abi::H256;

    const FACTORIAL: [u8; 32] = address!(
        "a10139386dcf00136361c2150c420435e3708b0b6833f09b0ad2699fc2333cb8"
    );

    const STORAGE: [u8; 32] = address!(
        "caa768a65c3752d83804a63134699c76f6472d864768f4bb6bb610b46b3b9106"
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
