// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Execute, Query, StoreContext};
use rusk_uplink_derive::{execute, init, query, state};

#[state]
pub struct BlockHeight;

#[init]
fn init() {}

#[query]
pub struct ReadBlockHeight;

impl Query for ReadBlockHeight {
    const NAME: &'static str = "read_block_height";
    type Return = u64;
}

#[execute(name = "read_block_height")]
impl Execute<ReadBlockHeight> for BlockHeight {
    fn execute(&self, _: ReadBlockHeight, _: StoreContext) -> u64 {
        rusk_uplink::block_height()
    }
}
