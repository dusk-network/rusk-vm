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

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Execute, Query, StoreContext};

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct BlockHeight;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct ReadBlockHeight;

impl Query for ReadBlockHeight {
    const NAME: &'static str = "read_block_height";
    type Return = u64;
}

impl Execute<ReadBlockHeight> for BlockHeight {
    fn execute(
        &self,
        _: &ReadBlockHeight,
        _: StoreContext,
    ) -> <ReadBlockHeight as Query>::Return {
        rusk_uplink::block_height()
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    #[no_mangle]
    static mut SCRATCH: [u8; 128] = [0u8; 128];

    query_state_arg_fun!(read_block_height, BlockHeight, ReadBlockHeight);
};
