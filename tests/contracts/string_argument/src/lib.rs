// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(
    core_intrinsics,
    lang_items,
    alloc_error_handler,
    option_result_unwrap_unchecked
)]

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Execute, Query};

#[derive(Clone, Debug, Archive, Deserialize, Serialize)]
pub struct Stringer;

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct Passthrough(String);

impl Passthrough {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Passthrough(s.into())
    }
}

impl Query for Passthrough {
    const NAME: &'static str = "pass";
    type Return = String;
}

impl Execute<Passthrough> for Stringer {
    fn execute(&self, p: &Passthrough) -> <Passthrough as Query>::Return {
        p.0.clone()
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 1024] = [0u8; 1024];

    #[no_mangle]
    fn pass(written: u32) -> u32 {
        let mut store = AbiStore;

        let (state, arg) = unsafe {
            archived_root::<(Stringer, Passthrough)>(
                &SCRATCH[..written as usize],
            )
        };

        let de_state: Stringer = (state).deserialize(&mut store).unwrap();
        let de_query: Passthrough = (arg).deserialize(&mut store).unwrap();

        let res: <Passthrough as Query>::Return = de_state.execute(&de_query);
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<Passthrough as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }
};
