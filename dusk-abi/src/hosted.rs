// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

const BUFFER_SIZE_LIMIT: usize = 1024;

use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};

pub use crate::{ContractId, ContractState, Query, ReturnValue, Transaction};

#[doc(hidden)]
pub mod panic_include;

#[doc(hidden)]
pub mod bufwriter;

#[doc(hidden)]
pub mod debug;

// declare available host-calls
pub mod external {
    extern "C" {
        #[allow(unused)]
        pub fn debug(buffer: &u8, len: i32);

        pub fn query(target: &u8, buf: &mut u8);
        pub fn transact(target: &u8, buf: &mut u8);

        pub fn caller(buffer: &mut u8);
        pub fn callee(buffer: &mut u8);

        pub fn gas(value: i32);
        pub fn block_height() -> u64;
    }
}

/// Returns the caller of the contract
pub fn caller() -> ContractId {
    let mut result = ContractId::default();
    unsafe { external::caller(&mut result.as_bytes_mut()[0]) }
    result
}

/// Returns the hash of the currently executing contract
pub fn callee() -> ContractId {
    let mut result = ContractId::default();
    unsafe { external::callee(&mut result.as_bytes_mut()[0]) }
    result
}

/// Returns the current block height
pub fn block_height() -> u64 {
    unsafe { external::block_height() }
}

/// Deduct a specified amount of gas from the call
pub fn gas(value: i32) {
    unsafe { external::gas(value) }
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

#[cfg(test)]
mod test {
    use super::*;
    extern crate alloc;

    use alloc::vec::Vec;
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
