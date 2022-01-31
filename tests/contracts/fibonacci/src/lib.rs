// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(
    core_intrinsics,
    lang_items,
    alloc_error_handler,
    option_result_unwrap_unchecked
)]

use microkelvin::{OffsetLen, StoreRef};
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Execute, Query};
use rusk_uplink_derive::query;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct Fibonacci;

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct ComputeFrom {
    value: u32,
}

impl ComputeFrom {
    pub fn new(n: u32) -> Self {
        Self { value: n }
    }
}

#[query(name="compute", buf=128)]
impl Execute<ComputeFrom> for Fibonacci {
    fn execute(
        &self,
        compute_from: ComputeFrom,
        store: StoreRef<OffsetLen>,
    ) -> u32 {
        let n = compute_from.value;
        if n < 2 {
            n
        } else {
            let callee = rusk_uplink::callee();

            let a = rusk_uplink::query::<ComputeFrom>(
                &callee,
                ComputeFrom::new(n - 1),
                0,
                store.clone(),
            )
            .unwrap();

            let b = rusk_uplink::query::<ComputeFrom>(
                &callee,
                ComputeFrom::new(n - 2),
                0,
                store,
            )
            .unwrap();
            a + b
        }
    }
}
