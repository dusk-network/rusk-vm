// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

const BUFFER_SIZE_LIMIT: usize = 1024 * 16;

pub use crate::{
    AbiStore, ArchiveError, ContractId, ContractState, Query, RawQuery,
    ReturnValue, Transaction,
};
use bytecheck::CheckBytes;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{
    ser::serializers::AllocSerializer, Archive, Deserialize, Fallible,
    Serialize,
};

// declare available host-calls
pub mod external {
    extern "C" {
        #[allow(unused)]
        pub fn query(
            target: &u8,
            buf: &mut u8,
            name: &u8,
            name_len: u32,
            gas_limit: u64,
        ) -> u32;
        pub fn callee(buffer: &mut u8);
    }
}

/// Call another contract at address `target`
///
/// Note that you will have to specify the expected return and argument types
/// yourself.
pub fn query<Q>(
    target: &ContractId,
    q: Q,
    gas_limit: u64,
) -> Result<Q::Return, ArchiveError>
where
    Q: Query + Serialize<AllocSerializer<1024>>,
    Q::Return: Archive + Clone,
    <Q::Return as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
        + Deserialize<Q::Return, AbiStore>,
{
    let mut raw_query = RawQuery::new(q);
    let name = raw_query.name_clone();
    let name_str = name.as_str();
    let data = raw_query.mut_data();
    let result_offset = unsafe {
        external::query(
            &target.as_bytes()[0],
            &mut data[0],
            &name.as_bytes()[0],
            name_str.len() as u32,
            gas_limit,
        )
    };
    let result =
        ReturnValue::new(&raw_query.mut_data()[..result_offset as usize]);
    let cast = result
        .cast::<Q::Return>()
        .map_err(|_| ArchiveError::ArchiveValidationError)?;

    let mut store = AbiStore;
    let deserialized: Q::Return =
        cast.deserialize(&mut store).expect("Infallible");

    Ok(deserialized)
}

///Returns the hash of the currently executing contract
pub fn callee() -> ContractId {
    let mut result = ContractId::default();
    unsafe { external::callee(&mut result.as_bytes_mut()[0]) };
    result
}
