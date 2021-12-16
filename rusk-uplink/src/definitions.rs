use core::fmt::Debug;

use rkyv::{Archive, Serialize, Deserialize};
use crate::AbiStore;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default, Archive)]
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


pub trait Execute<Q>
where
    Q: Query,
{
    fn execute(&self, q: &Q) -> Q::Return;
}

pub trait Apply<T>
where
    T: Transaction,
{
    fn apply(&mut self, t: &T) -> T::Return;
}

pub trait Query: Archive {
    const NAME: &'static str;

    type Return;
}

pub trait Transaction: Archive {
    const NAME: &'static str;

    type Return;
}

#[derive(Debug, Default, Archive, Serialize, Deserialize)]
pub struct ContractState(Vec<u8>);

impl ContractState {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

pub trait HostModule {
    fn execute(&self) -> Result<ReturnValue, ()>; // todo this is not the final shape of it
}

#[derive(Debug, Default, Archive, Serialize, Deserialize)]
pub struct ReturnValue;

#[derive(Debug)]
pub struct StoreError;