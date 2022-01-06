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

impl Query for ComputeFrom {
    const NAME: &'static str = "compute";
    type Return = u32;
}

impl Execute<ComputeFrom> for Fibonacci {
    fn execute(
        &self,
        compute_from: &ComputeFrom,
        store: StoreRef<OffsetLen>,
    ) -> <ComputeFrom as Query>::Return {
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

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::{AbiStore, StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 128] = [0u8; 128];

    #[no_mangle]
    fn compute(written_state: u32, written_data: u32) -> u32 {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<Fibonacci>(&SCRATCH[..written_state as usize])
        };
        let arg = unsafe {
            archived_root::<ComputeFrom>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let de_state: Fibonacci = state.deserialize(&mut store).unwrap();
        let de_query: ComputeFrom = arg.deserialize(&mut store).unwrap();

        let res: <ComputeFrom as Query>::Return =
            de_state.execute(&de_query, store);
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<ComputeFrom as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }
};
