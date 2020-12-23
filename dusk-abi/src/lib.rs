//! #Dusk ABI
//!
//! ABI functionality for communicating with the host
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(lang_items)]
#![feature(panic_info_message)]

use canonical::{
    BridgeStore, ByteSink, ByteSource, Canon, Id32, Sink, Source, Store,
};
use canonical_derive::Canon;

use const_arrayvec::ArrayVec;

const BUFFER_SIZE_LIMIT: usize = 1024;

#[doc(hidden)]
pub const DEBUG_BUFFER_SIZE: usize = 1024;

#[doc(hidden)]
pub mod bufwriter;

// General types to reprent queries, transactions and return values
#[derive(Clone, Debug, Default, PartialEq)]
struct Buffer<const N: usize>(ArrayVec<u8, N>);

impl<S, const N: usize> Canon<S> for Buffer<N>
where
    S: Store,
{
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        let len = self.0.len() as u16;
        Canon::<S>::write(&len, sink)?;
        sink.copy_bytes(&self.0[..]);
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let len: u16 = Canon::<S>::read(source)?;
        Ok(Self::from_bytes(source.read_bytes(len as usize)))
    }

    fn encoded_len(&self) -> usize {
        Canon::<S>::encoded_len(&0u16) + self.0.len()
    }
}

impl<const N: usize> Buffer<N> {
    /// Creates a buffer from a type implementing `Canon`
    fn from_bytes(buffer: &[u8]) -> Self {
        let mut vec = ArrayVec::new();
        vec.try_extend_from_slice(buffer)
            .unwrap_or_else(|_| panic!("too large! {}", buffer.len()));

        Buffer(vec)
    }

    /// Creates a buffer from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        let mut buffer = [0u8; BUFFER_SIZE_LIMIT];
        let mut sink = ByteSink::new(&mut buffer, s);
        let len = Canon::<S>::encoded_len(c);
        Canon::<S>::write(c, &mut sink)?;
        Ok(Self::from_bytes(&buffer[..len]))
    }
}

/// A generic query
#[derive(Clone, Canon, Debug, Default)]
pub struct ContractState(Buffer<BUFFER_SIZE_LIMIT>);

impl ContractState {
    /// Returns the state of the contract as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &(self.0).0[..]
    }

    /// Creates a state from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(ContractState(Buffer::from_canon(c, s)?))
    }
}

/// A generic query
#[derive(Clone, Canon, Debug, Default)]
pub struct Query(Buffer<BUFFER_SIZE_LIMIT>);

impl Query {
    /// Returns the byte representation of the query
    pub fn as_bytes(&self) -> &[u8] {
        &(self.0).0[..]
    }

    /// Creates a query from a raw bytes
    pub fn from_bytes(buffer: &[u8]) -> Self {
        Query(Buffer::from_bytes(buffer))
    }

    /// Creates a query from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(Query(Buffer::from_canon(c, s)?))
    }
}

/// A generic transaction
#[derive(Clone, Canon, Debug, Default)]
pub struct Transaction(Buffer<BUFFER_SIZE_LIMIT>);

impl Transaction {
    /// Returns the byte representation of the transaction
    pub fn as_bytes(&self) -> &[u8] {
        &(self.0).0[..]
    }

    /// Creates a transaction from a raw bytes
    pub fn from_bytes(buffer: &[u8]) -> Self {
        Transaction(Buffer::from_bytes(buffer))
    }

    /// Creates a transaction from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(Transaction(Buffer::from_canon(c, s)?))
    }
}

/// A generic return value
#[derive(Clone, Canon, Debug, Default, PartialEq)]
pub struct ReturnValue(Buffer<BUFFER_SIZE_LIMIT>);

impl ReturnValue {
    /// Returns the byte representation of the transaction
    pub fn as_bytes(&self) -> &[u8] {
        &(self.0).0[..]
    }

    /// Creates a transaction from a raw bytes
    pub fn from_bytes(buffer: &[u8]) -> Self {
        ReturnValue(Buffer::from_bytes(buffer))
    }

    /// Creates a transaction from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(ReturnValue(Buffer::from_canon(c, s)?))
    }

    /// Casts the encoded return value to given type
    pub fn cast<C, S>(&self, store: S) -> Result<C, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        let mut source = ByteSource::new(self.as_bytes(), store);
        Canon::<S>::read(&mut source)
    }
}

/// Type used to identify a contract
#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Canon,
)]
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
    let mut sink = ByteSink::new(&mut buf, bs);

    query.write(&mut sink)?;

    unsafe { external::query(&target.as_bytes()[0], &mut buf[0]) }

    // read return back
    let mut source = ByteSource::new(&buf, bs);

    ReturnValue::read(&mut source)
}

/// Call another contract at address `target`
///
/// Note that you will have to specify the expected return and argument types yourself.
pub fn query<A, R>(
    target: &ContractId,
    query: &A,
) -> Result<R, <BridgeStore<Id32> as Store>::Error>
where
    A: Canon<BridgeStore<Id32>>,
    R: Canon<BridgeStore<Id32>>,
{
    let bs = BridgeStore::<Id32>::default();
    let wrapped = Query::from_canon(query, bs)?;
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
    let mut sink = ByteSink::new(&mut buf, bs);

    transaction.write(&mut sink)?;

    unsafe { external::transact(&target.as_bytes()[0], &mut buf[0]) }

    // read return back
    let mut source = ByteSource::new(&buf, bs);

    ReturnValue::read(&mut source)
}

/// Call another contract at address `target`
///
/// Note that you will have to specify the expected return and argument types yourself.
pub fn transact<A, R>(
    target: &ContractId,
    transaction: &A,
) -> Result<R, <BridgeStore<Id32> as Store>::Error>
where
    A: Canon<BridgeStore<Id32>>,
    R: Canon<BridgeStore<Id32>>,
{
    let bs = BridgeStore::<Id32>::default();
    let wrapped = Transaction::from_canon(transaction, bs)?;
    let result = transact_raw(target, &wrapped)?;
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
            Ok(Buffer::from_bytes(&vec[..]))
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
        fuzz_canon::<ReturnValue, _>(store.clone());
    }
}
