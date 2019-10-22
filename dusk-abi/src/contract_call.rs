use crate::{encoding, Error, CALL_DATA_SIZE};
use core::marker::PhantomData;
use core::ops::Deref;
use serde::{Deserialize, Serialize};

// R is the return type of the Call
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

    pub fn nil() -> Self {
        ContractCall {
            data: [0u8; CALL_DATA_SIZE],
            len: 0,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn data(&self) -> &[u8; CALL_DATA_SIZE] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8; CALL_DATA_SIZE] {
        &mut self.data
    }

    pub fn into_data(self) -> [u8; CALL_DATA_SIZE] {
        self.data
    }
}

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
