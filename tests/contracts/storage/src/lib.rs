// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![no_std]
use cake_rusk as cake;

#[cake::contract(version = "0.0.1")]
mod storage {
    pub fn get_value() -> i32 {
        match dusk_abi::get_storage::<_, i32>(b"test") {
            Some(val) => val,
            None => -1,
        }
    }

    pub fn delete() -> i32 {
        dusk_abi::delete_storage(b"test");
        -2
    }

    pub fn set_value(n: i32) -> i32 {
        dusk_abi::set_storage(b"test", n);
        n
    }
}
