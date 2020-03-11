use crate::{encoding, Error, CALL_DATA_SIZE};
use core::marker::PhantomData;
use core::ops::Deref;
use serde::{Deserialize, Serialize};

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
    pub fn new<C: Serialize + core::fmt::Debug>(
        call: C,
    ) -> Result<Self, Error> {
        let mut data = [0u8; CALL_DATA_SIZE];
        let len = encoding::encode(&call, &mut data)?.len();
        Ok(ContractCall {
            data,
            len,
            _marker: PhantomData,
        })
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

/// A struct representing the return of a contract
pub struct ContractReturn<R> {
    #[allow(unused)]
    data: [u8; CALL_DATA_SIZE],
    val: R,
}

impl<R> From<ContractCall<R>> for ContractReturn<R>
where
    R: for<'de> Deserialize<'de>,
{
    fn from(from: ContractCall<R>) -> Self {
        let data = from.data;
        let val = encoding::decode(&data).unwrap();

        ContractReturn { data, val }
    }
}

impl<R> Deref for ContractReturn<R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
