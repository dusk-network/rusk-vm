// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

pub use crate::{
    AbiStore, ArchiveError, ContractId, ContractState, Query, RawQuery,
    ReturnValue, Transaction,
};
use bytecheck::CheckBytes;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{
    ser::serializers::AllocSerializer, Archive, Deserialize, Serialize,
};
use crate::RawTransaction;

const BUFFER_SIZE_LIMIT: usize = 1024 * 16;

// declare available host-calls
pub mod external {
    extern "C" {
        #[allow(unused)]
        pub fn query(
            target: &u8,
            buf: &u8,
            buf_len: u32,
            name: &u8,
            name_len: u32,
            gas_limit: u64,
        ) -> u32;
        pub fn transact(
            target: &u8,
            buf: &u8,
            buf_len: u32,
            name: &u8,
            name_len: u32,
            gas_limit: u64,
        ) -> u32;
        pub fn callee(buffer: &mut u8);
    }
}

/// Call another contract at address `target`
pub fn query_raw(
    target: &ContractId,
    raw_query: &RawQuery,
    gas_limit: u64,
) -> Result<ReturnValue, ArchiveError> {
    let mut buf = [0u8; BUFFER_SIZE_LIMIT];
    let data_len = raw_query.data().len();
    buf[..data_len].copy_from_slice(raw_query.data());
    let name = raw_query.name();
    let result_offset = unsafe {
        external::query(
            &target.as_bytes()[0],
            &buf[0],
            data_len as u32,
            &name.as_bytes()[0],
            name.len() as u32,
            gas_limit,
        )
    };
    let result = ReturnValue::new(&buf[..result_offset as usize]);
    Ok(result)
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
    let raw_query = RawQuery::new(q);

    let result = query_raw(target, &raw_query, gas_limit)?;

    let cast = result
        .cast::<Q::Return>()
        .map_err(|_| ArchiveError::ArchiveValidationError)?;

    let mut store = AbiStore;
    let deserialized: Q::Return =
        cast.deserialize(&mut store).expect("Infallible");

    Ok(deserialized)
}

/// Call another contract at address `target`
pub fn transact_raw(
    target: &ContractId,
    raw_transaction: &RawTransaction,
    gas_limit: u64,
) -> Result<ReturnValue, ArchiveError> {
    let mut buf = [0u8; BUFFER_SIZE_LIMIT];
    let data_len = raw_transaction.data().len();
    buf[..data_len].copy_from_slice(raw_transaction.data());
    let name = raw_transaction.name();
    let result_offset = unsafe {
        external::transact(
            &target.as_bytes()[0],
            &buf[0],
            data_len as u32,
            &name.as_bytes()[0],
            name.len() as u32,
            gas_limit,
        )
    };
    if result_offset > 0 {
        let result = ReturnValue::new(&buf[..result_offset as usize]);
        Ok(result)
    } else {
        Ok(ReturnValue::new(Vec::new())) // todo! remove this
    }
}

///Returns the hash of the currently executing contract
pub fn callee() -> ContractId {
    let mut result = ContractId::default();
    unsafe { external::callee(&mut result.as_bytes_mut()[0]) };
    result
}
