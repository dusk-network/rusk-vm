// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

extern crate alloc;

pub use crate::{
    ArchiveError, ContractId, ContractState, Query, RawEvent, RawQuery,
    RawTransaction, ReturnValue, Transaction,
};

use alloc::string::String;

use bytecheck::CheckBytes;
use microkelvin::{OffsetLen, StoreRef, StoreSerializer};
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Archive, Deserialize, Serialize};

const BUFFER_SIZE_LIMIT: usize = 1024 * 16;

// declare available host-calls
pub mod external {
    extern "C" {
        pub fn debug(buffer: &u8, len: i32);

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
        ) -> u64;

        pub fn emit(buf: &u8, buf_len: u32, name: &u8, name_len: u32);

        pub fn callee(buffer: &mut u8);

        pub fn caller(buffer: &mut u8);

        pub fn block_height() -> u64;

        pub fn gas_consumed() -> u64;

        pub fn gas_left() -> u64;
    }
}

/// Write debug string
pub fn debug_raw(debug_string: impl AsRef<str>) {
    let mut buffer = [0u8; 1024];
    let string = debug_string.as_ref();
    buffer[..string.len()].copy_from_slice(string.as_bytes());
    unsafe { external::debug(&buffer[0], string.len() as i32) }
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
    mut store: StoreRef<OffsetLen>,
) -> Result<Q::Return, ArchiveError>
where
    Q: Query + Serialize<StoreSerializer<OffsetLen>>,
    Q::Return: Archive,
    <Q::Return as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
        + Deserialize<Q::Return, StoreRef<OffsetLen>>,
{
    let raw_query = RawQuery::new(q, &store);

    let result = query_raw(target, &raw_query, gas_limit)?;

    let cast = result
        .cast::<Q::Return>()
        .map_err(|_| ArchiveError::ArchiveValidationError)?;

    let deserialized: Q::Return =
        cast.deserialize(&mut store).expect("Infallible");

    Ok(deserialized)
}

/// Call another contract at address `target`
pub fn transact_raw<Slf>(
    slf: &mut Slf,
    target: &ContractId,
    raw_transaction: &RawTransaction,
    gas_limit: u64,
    mut store: StoreRef<OffsetLen>,
) -> Result<ReturnValue, ArchiveError>
where
    Slf: Archive,
    <Slf as Archive>::Archived: Deserialize<Slf, StoreRef<OffsetLen>>,
{
    let mut buf = [0u8; BUFFER_SIZE_LIMIT];
    let data_len = raw_transaction.data().len();
    buf[..data_len].copy_from_slice(raw_transaction.data());
    let name = raw_transaction.name();

    let offsets = unsafe {
        external::transact(
            &target.as_bytes()[0],
            &buf[0],
            data_len as u32,
            &name.as_bytes()[0],
            name.len() as u32,
            gas_limit,
        )
    };
    let result_offset = (offsets & 0xffffffff00000000) >> 32;
    let state_offset = offsets & 0xffffffff;
    let result = ReturnValue::with_state(
        &buf[state_offset as usize..result_offset as usize],
        &buf[..state_offset as usize],
    );
    let cast_state = result.cast_state::<Slf>();
    let deserialized_state: Slf =
        cast_state.deserialize(&mut store).expect("Infallible");
    *slf = deserialized_state;

    Ok(result)
}

/// Call another contract at address `target`
///
/// Note that you will have to specify the expected return and argument types
/// yourself.
pub fn transact<T, Slf>(
    slf: &mut Slf,
    target: &ContractId,
    transaction: T,
    gas_limit: u64,
    mut store: StoreRef<OffsetLen>,
) -> Result<T::Return, ArchiveError>
where
    T: Transaction + Serialize<StoreSerializer<OffsetLen>>,
    T::Return: Archive,
    <T::Return as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
        + Deserialize<T::Return, StoreRef<OffsetLen>>,
    Slf: Archive,
    <Slf as Archive>::Archived: Deserialize<Slf, StoreRef<OffsetLen>>,
{
    let raw_transaction = RawTransaction::new(transaction, &store);

    let result =
        transact_raw(slf, target, &raw_transaction, gas_limit, store.clone())?;

    let cast = result
        .cast::<T::Return>()
        .map_err(|_| ArchiveError::ArchiveValidationError)?;

    let deserialized_result: T::Return =
        cast.deserialize(&mut store).expect("Infallible");

    Ok(deserialized_result)
}

pub fn emit_raw(raw_event: &RawEvent) {
    let mut buf = [0u8; BUFFER_SIZE_LIMIT];
    let data_len = raw_event.data().len();
    buf[..data_len].copy_from_slice(raw_event.data());
    let name = raw_event.name();

    unsafe {
        external::emit(
            &buf[0],
            data_len as u32,
            &name.as_bytes()[0],
            name.len() as u32,
        )
    }
}

pub fn emit<S, E>(name: S, event: E, store: StoreRef<OffsetLen>)
where
    S: Into<String>,
    E: Archive + Serialize<StoreSerializer<OffsetLen>>,
{
    let raw_event = RawEvent::new(name, event, &store);
    emit_raw(&raw_event);
}

///Returns the hash of the currently executing contract
pub fn callee() -> ContractId {
    let mut result = ContractId::default();
    unsafe { external::callee(&mut result.as_bytes_mut()[0]) };
    result
}

/// Returns the caller of the contract
pub fn caller() -> ContractId {
    let mut result = ContractId::default();
    unsafe { external::caller(&mut result.as_bytes_mut()[0]) };
    result
}

/// Returns the current block height
pub fn block_height() -> u64 {
    unsafe { external::block_height() }
}

/// Deduct a specified amount of gas from the call
// pub fn gas(value: i32) {
//     unsafe { external::gas(value) }
// }

/// Return the amount of gas consumed until the point when the host call is
/// executed.
pub fn gas_consumed() -> u64 {
    unsafe { external::gas_consumed() }
}

/// Return the ammunt of gas left until the point when the host call is
/// executed.
pub fn gas_left() -> u64 {
    unsafe { external::gas_left() }
}
