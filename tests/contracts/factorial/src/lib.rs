// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![no_std]
#[no_mangle]
use cake_rusk as cake;

#[cake::contract(version = "0.0.1")]
mod factorial {
    pub fn factorial(n: u64) -> u64 {
        let self_hash = dusk_abi::self_hash();
        if n <= 1 {
            1
        } else {
            n * dusk_abi::call_contract_operation::<u64, u64>(
                &self_hash,
                1,
                0,
                n - 1,
            )
        }
    }
}
