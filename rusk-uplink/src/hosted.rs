// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

const BUFFER_SIZE_LIMIT: usize = 1024 * 16;

pub use crate::{AbiStore, ContractId, ContractState, Query, RawQuery, ReturnValue, Transaction};
use rkyv::{
    ser::{serializers::AllocSerializer},
    Archive, Fallible, Serialize,
};

// declare available host-calls
pub mod external {
    extern "C" {
        #[allow(unused)]
        pub fn query(target: &u8, buf: &mut u8, gas_limit: u64) -> u32;
    }
}

/// Call another contract at address `target`
///
/// Note that you will have to specify the expected return and argument types
/// yourself.
pub fn query<Q>(
    target: &ContractId,
    q: &Q,
    gas_limit: u64,
) -> Result<u32, <AbiStore as Fallible>::Error>
    where
        Q: Query + Serialize<AllocSerializer<1024>>,
        Q::Return: Archive + Clone,
{
    let mut raw_query = RawQuery::new(q);
    let result_offset = unsafe { external::query(&target.as_bytes()[0], &mut raw_query.mut_data()[0], gas_limit) };
    Ok(result_offset)
}
