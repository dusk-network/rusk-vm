// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{Archive, Serialize};
use vm_poc::{Contract, ContractId};

#[derive(Archive, Default, Serialize)]
pub struct Complex {
    hair_count: u32,
    frobnicated: bool,
    bogo_secret: [u8; 11],
}

impl Contract for Complex {
    fn code() -> &'static [u8] {
        include_bytes!("../target/wasm32-unknown-unknown/release/complex.wasm")
    }
}

#[no_mangle]
fn grow_hair(slf: &mut Complex, amount: u32) {
    slf.hair_count += if slf.frobnicated { amount * 2 } else { amount };
}

#[no_mangle]
fn check_hair(slf: &mut Complex) -> u32 {
    slf.hair_count
}

#[no_mangle]
fn frobnicate(slf: &mut Complex) {
    slf.frobnicated = true
}
