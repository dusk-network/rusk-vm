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
use rusk_uplink::Query;
extern crate alloc;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct BlockHeight;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct ReadBlockHeight;

impl Query for ReadBlockHeight {
    const NAME: &'static str = "read_block_height";
    type Return = u64;
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;

    #[no_mangle]
    static mut SCRATCH: [u8; 128] = [0u8; 128];

    #[no_mangle]
    fn read_block_height(_written_state: u32, _written_data: u32) -> u32 {
        let block_height = rusk_uplink::block_height();
        let res: <ReadBlockHeight as Query>::Return = block_height;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<ReadBlockHeight as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }
};
