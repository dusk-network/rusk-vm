use std::fmt::Debug;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
pub struct ContractId([u8; 32]);

pub trait Contract {
    fn code() -> &'static [u8];
}

pub trait Query {
    const NAME: &'static str;
    type Return;
}

pub trait Queryable<Q: Query> {
    fn query(&self, q: Q) -> Q::Return;
}

pub trait Transaction {
    const NAME: &'static str;
    type Return;
}

pub trait Transactable<T: Transaction> {
    fn transact(&mut self, t: T) -> T::Return;
}
