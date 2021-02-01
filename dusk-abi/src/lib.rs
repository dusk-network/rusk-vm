// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! #Dusk ABI
//!
//! ABI functionality for communicating with the host
#![warn(missing_docs)]
#![no_std]

// re-export WeeAlloc

pub use wee_alloc::WeeAlloc;

extern crate alloc;

use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};
use canonical_derive::Canon;

use alloc::vec::Vec;

const BUFFER_SIZE_LIMIT: usize = 1024;

#[doc(hidden)]
pub const DEBUG_BUFFER_SIZE: usize = 1024;

#[doc(hidden)]
pub mod bufwriter;

trait CanonToVec<S>
where
    S: Store,
{
    fn encode_to_vec(&self, store: &S) -> Result<Vec<u8>, S::Error>;
}

impl<T, S> CanonToVec<S> for T
where
    T: Canon<S>,
    S: Store,
{
    fn encode_to_vec(&self, store: &S) -> Result<Vec<u8>, S::Error> {
        let len = Canon::<S>::encoded_len(self);

        let mut vec = Vec::new();
        vec.resize_with(len, || 0);
        let mut sink = ByteSink::new(&mut vec[..], store);

        Canon::<S>::write(self, &mut sink)?;
        Ok(vec)
    }
}

/// A generic query
#[derive(Clone, Canon, Debug, Default)]
pub struct ContractState(Vec<u8>);

impl ContractState {
    /// Returns the state of the contract as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    /// Creates a state from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: &S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(ContractState(c.encode_to_vec(s)?))
    }
}

/// A generic query
#[derive(Clone, Canon, Debug, Default)]
pub struct Query(Vec<u8>);

impl Query {
    /// Returns the byte representation of the query
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    /// Creates a query from a raw bytes
    pub fn from_slice(buffer: &[u8]) -> Self {
        Query(buffer.to_vec())
    }

    /// Creates a query from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: &S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(Query(c.encode_to_vec(s)?))
    }
}

/// A generic transaction
#[derive(Clone, Canon, Debug, Default)]
pub struct Transaction(Vec<u8>);

impl Transaction {
    /// Returns the byte representation of the transaction
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    /// Creates a transaction from a raw bytes
    pub fn from_slice(buffer: &[u8]) -> Self {
        Transaction(buffer.to_vec())
    }

    /// Creates a transaction from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: &S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(Transaction(c.encode_to_vec(s)?))
    }
}

/// A generic return value
#[derive(Clone, Canon, Debug, Default, PartialEq)]
pub struct ReturnValue(Vec<u8>);

impl ReturnValue {
    /// Returns the byte representation of the transaction
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    /// Creates a transaction from a raw bytes
    pub fn from_slice(buffer: &[u8]) -> Self {
        ReturnValue(buffer.to_vec())
    }

    /// Creates a transaction from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: &S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(ReturnValue(c.encode_to_vec(s)?))
    }

    /// Casts the encoded return value to given type
    pub fn cast<C, S>(&self, store: S) -> Result<C, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        let mut source = ByteSource::new(self.as_bytes(), &store);
        Canon::<S>::read(&mut source)
    }
}

/// Type used to identify a contract
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Canon)]
pub struct ContractId([u8; 32]);

impl<B> From<B> for ContractId
where
    B: AsRef<[u8]>,
{
    fn from(b: B) -> Self {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(b.as_ref());
        ContractId(bytes)
    }
}

impl ContractId {
    /// Returns the contract id as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// declare available host-calls
mod external {
    extern "C" {
        #[allow(unused)]
        pub fn caller(buffer: &mut u8);
        pub fn self_id(buffer: &mut u8);

        pub fn debug(buffer: &u8, len: i32);

        pub fn query(target: &u8, buf: &mut u8);
        pub fn transact(target: &u8, buf: &mut u8);

        pub fn gas(value: i32);
    }
}

/// Returns the caller of the contract
pub fn caller() -> ContractId {
    let mut result = ContractId::default();
    unsafe { external::caller(&mut result.as_bytes_mut()[0]) }
    result
}

/// Returns the hash of the currently executing contract
pub fn self_id() -> ContractId {
    let mut result = ContractId::default();
    unsafe { external::self_id(&mut result.as_bytes_mut()[0]) }
    result
}

/// Call another contract at address `target`
pub fn query_raw(
    target: &ContractId,
    query: &Query,
) -> Result<ReturnValue, <BridgeStore<Id32> as Store>::Error> {
    let bs = BridgeStore::<Id32>::default();

    let mut buf = [0u8; BUFFER_SIZE_LIMIT];
    let mut sink = ByteSink::new(&mut buf, &bs);

    query.write(&mut sink)?;

    unsafe { external::query(&target.as_bytes()[0], &mut buf[0]) }

    // read return back
    let mut source = ByteSource::new(&buf, &bs);

    ReturnValue::read(&mut source)
}

/// Call another contract at address `target`
///
/// Note that you will have to specify the expected return and argument types
/// yourself.
pub fn query<A, R>(
    target: &ContractId,
    query: &A,
) -> Result<R, <BridgeStore<Id32> as Store>::Error>
where
    A: Canon<BridgeStore<Id32>>,
    R: Canon<BridgeStore<Id32>>,
{
    let bs = BridgeStore::<Id32>::default();
    let wrapped = Query::from_canon(query, &bs)?;
    let result = query_raw(target, &wrapped)?;
    result.cast(bs)
}

/// Call another contract at address `target`
pub fn transact_raw(
    target: &ContractId,
    transaction: &Transaction,
) -> Result<ReturnValue, <BridgeStore<Id32> as Store>::Error> {
    let bs = BridgeStore::<Id32>::default();

    let mut buf = [0u8; BUFFER_SIZE_LIMIT];
    let mut sink = ByteSink::new(&mut buf, &bs);

    transaction.write(&mut sink)?;

    unsafe { external::transact(&target.as_bytes()[0], &mut buf[0]) }

    // read return back
    let mut source = ByteSource::new(&buf, &bs);

    ReturnValue::read(&mut source)
}

/// Call another contract at address `target`
///
/// Note that you will have to specify the expected return and argument types
/// yourself.
pub fn transact<A, R>(
    target: &ContractId,
    transaction: &A,
) -> Result<R, <BridgeStore<Id32> as Store>::Error>
where
    A: Canon<BridgeStore<Id32>>,
    R: Canon<BridgeStore<Id32>>,
{
    let bs = BridgeStore::<Id32>::default();
    let wrapped = Transaction::from_canon(transaction, &bs)?;
    let result = transact_raw(&target, &wrapped)?;
    result.cast(bs)
}

/// Deduct a specified amount of gas from the call
pub fn gas(value: i32) {
    unsafe { external::gas(value) }
}

#[doc(hidden)]
pub fn _debug(buf: &[u8]) {
    let len = buf.len() as i32;
    unsafe { external::debug(&buf[0], len) }
}

/// Macro to format and send debug output to the host
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {
        use core::fmt::Write as _;
        let mut buffer = [0u8; $crate::DEBUG_BUFFER_SIZE];
        let len = {
            let mut bw = $crate::bufwriter::BufWriter::new(&mut buffer);
            write!(bw, $($tt)*).unwrap();
            bw.ofs()
        };
        $crate::_debug(&buffer[0..len])
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use canonical_fuzz::*;
    use canonical_host::MemStore;

    use arbitrary::{Arbitrary, Unstructured};

    impl Arbitrary for Buffer<BUFFER_SIZE_LIMIT> {
        fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
            let mut vec = Vec::arbitrary(u)?;
            vec.truncate(BUFFER_SIZE_LIMIT);
            Ok(Buffer::from_slice(&vec[..]))
        }
    }

    impl Arbitrary for ReturnValue {
        fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
            Ok(ReturnValue(Buffer::arbitrary(u)?))
        }
    }

    #[test]
    fn fuzz_buffer() {
        let store = MemStore::new();
        fuzz_canon::<Buffer<BUFFER_SIZE_LIMIT>, _>(store.clone());
        fuzz_canon::<ReturnValue, _>(store);
    }
}
