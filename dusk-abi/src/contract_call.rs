use core::marker::PhantomData;
use core::mem;

use dataview::Pod;

use crate::CALL_DATA_SIZE;

/// Type describing a contract call, includes
/// the `R` parameter to specify return type
pub struct ContractCall<R> {
    data: [u8; CALL_DATA_SIZE],
    len: usize,
    _marker: PhantomData<R>,
}

impl<R> Clone for ContractCall<R> {
    fn clone(&self) -> Self {
        let mut data = [0u8; CALL_DATA_SIZE];
        data.copy_from_slice(&self.data);
        ContractCall {
            data,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

impl<R> ContractCall<R> {
    /// Create a new ContractCall with given arguments `C`
    pub fn new<C: Pod>(call: C) -> Self {
        let mut data = [0u8; CALL_DATA_SIZE];

        data[0..].copy_from_slice(call.as_bytes());

        let len = mem::size_of::<C>();
        ContractCall {
            data,
            len,
            _marker: PhantomData,
        }
    }

    /// Create a ContractCall from raw bytes
    pub fn new_raw(raw: &[u8]) -> Self {
        let mut data = [0u8; CALL_DATA_SIZE];
        let len = raw.len();
        data[0..len].copy_from_slice(raw);
        ContractCall {
            data,
            len,
            _marker: PhantomData,
        }
    }

    /// Create an empty contract call
    pub fn nil() -> Self {
        ContractCall {
            data: [0u8; CALL_DATA_SIZE],
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Returns the length of the argument data
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the call is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the call data segment
    pub fn data(&self) -> &[u8; CALL_DATA_SIZE] {
        &self.data
    }

    /// Returns a mutable reference to the call data segment
    pub fn data_mut(&mut self) -> &mut [u8; CALL_DATA_SIZE] {
        &mut self.data
    }

    /// Consume the `ContractCall` and return the call data
    pub fn into_data(self) -> [u8; CALL_DATA_SIZE] {
        self.data
    }
}
