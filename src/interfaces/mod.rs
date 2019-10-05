use std::marker::PhantomData;

use dusk_abi::{Error, MAX_CALL_DATA_SIZE};
use serde::Serialize;

pub use self::default_account::DefaultAccount;

mod default_account;

// C and R are the types of the Call, and Return respectively
pub struct ContractCall<C, R> {
    data: [u8; MAX_CALL_DATA_SIZE],
    _marker: PhantomData<(C, R)>,
}

impl<C: Serialize, R> ContractCall<C, R> {
    pub fn new(call: C) -> Result<Self, Error> {
        let mut data = [0u8; MAX_CALL_DATA_SIZE];
        dusk_abi::encoding::encode(&call, &mut data)?;
        Ok(ContractCall {
            data,
            _marker: PhantomData,
        })
    }

    pub fn nil() -> Self {
        ContractCall {
            data: [0u8; MAX_CALL_DATA_SIZE],
            _marker: PhantomData,
        }
    }

    pub fn data(&self) -> &[u8; MAX_CALL_DATA_SIZE] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8; MAX_CALL_DATA_SIZE] {
        &mut self.data
    }
}
