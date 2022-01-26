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
use rusk_uplink::{Execute, Query, StoreContext};
use rusk_uplink_derive::query;


#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct BlockHeight;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct ReadBlockHeight;

impl Query for ReadBlockHeight {
    const NAME: &'static str = "read_block_height";
    type Return = u64;
}

// impl Execute<ReadBlockHeight> for BlockHeight {
//     fn execute(
//         &self,
//         _: ReadBlockHeight,
//         _: StoreContext,
//     ) -> <ReadBlockHeight as Query>::Return {
//         rusk_uplink::block_height()
//     }
// }

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    scratch_memory!(128);

    #[query]
    pub fn read_block_height(_state: BlockHeight, _arg: ReadBlockHeight, _store: StoreRef<OffsetLen>) -> u64 {
        rusk_uplink::block_height()
    }
};
