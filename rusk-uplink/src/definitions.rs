use core::fmt::Debug;

use rkyv::Archive;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
pub struct ContractId([u8; 32]);

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
