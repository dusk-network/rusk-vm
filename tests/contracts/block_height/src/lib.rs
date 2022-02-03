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
use rusk_uplink_derive::query;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct BlockHeight;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct ReadBlockHeight;

#[query(name = "read_block_height")]
impl Execute<ReadBlockHeight> for BlockHeight {
    fn execute(&self, _: ReadBlockHeight, _: StoreContext) -> u64 {
        rusk_uplink::block_height()
    }
}
