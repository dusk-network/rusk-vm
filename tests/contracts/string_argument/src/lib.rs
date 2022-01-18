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
use rusk_uplink::{Execute, Query, StoreContext};
use rusk_uplink::{get_state_and_arg, q_return, query_state_arg_fun};

// #[macro_use]
// extern crate rusk_uplink;

#[derive(Clone, Debug, Archive, Deserialize, Serialize)]
pub struct Stringer;

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct Passthrough {
    string: String,
    repeat: u32,
    junk: u32,
}

impl Passthrough {
    pub fn new<S: Into<String>>(s: S, repeat: u32) -> Self {
        Passthrough {
            string: s.into(),
            junk: 82,
            repeat,
        }
    }
}

impl Query for Passthrough {
    const NAME: &'static str = "pass";
    type Return = String;
}

impl Execute<Passthrough> for Stringer {
    fn execute(
        &self,
        p: &Passthrough,
        _: StoreContext,
    ) -> <Passthrough as Query>::Return {
        p.string.repeat(p.repeat as usize)
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 1024] = [0u8; 1024];

    // #[no_mangle]
    // fn pass(written_state: u32, written_data: u32) -> u32 {
    //     let (de_state, de_query): (Stringer, Passthrough) = unsafe { get_state_and_arg(written_state, written_data, &SCRATCH) };
    //
    //     let store =
    //         StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
    //     let res: <Passthrough as Query>::Return =
    //         de_state.execute(&de_query, store);
    //
    //     unsafe { q_return(&res, &mut SCRATCH) }
    // }
    query_state_arg_fun!(pass, Stringer, Passthrough);
};
