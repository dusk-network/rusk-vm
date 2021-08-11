use std::collections::HashMap as Map;
use std::fmt::Debug;

use crate::definitions::*;

//use rkyv::ser::serializers::BufferSerializer;
use rkyv::{Archive, Infallible, Serialize};
use wasmer::{imports, Instance, Module, Store, Value};

/// Default wasm memory pages are 64k large
pub const PAGE_SIZE: usize = 64 * 1024;

/// Pages of memory, enforced to be a multiple of `PAGE_SIZE` in length
#[derive(Debug)]
pub struct Pages(Vec<u8>);

impl Pages {
    fn new() -> Self {
        Pages(vec![0u8; PAGE_SIZE])
    }
}

#[derive(Debug)]
struct ContractInstance {
    pub data: Pages,
    pub code: Vec<u8>,
}

impl ContractInstance {
    fn id(&self) -> ContractId {
        Default::default()
    }
}

#[derive(Debug)]
pub enum Error {
    UnknownContract,
}

#[derive(Debug, Default)]
pub struct State {
    map: Map<ContractId, ContractInstance>,
    wasmer_store: Store,
}

impl State {
    pub fn deploy<C>(&mut self, contract: C) -> ContractId
    where
        C: Archive + Contract + Serialize<Infallible>,
    {
        let initial_memory = Pages::new();
        let _data = contract.serialize(&mut Infallible);

        let instance = ContractInstance {
            code: C::code().into(),
            data: initial_memory,
        };

        let id = instance.id();

        self.map.insert(id, instance);
        id
    }

    pub fn query<Q>(
        &self,
        id: ContractId,
        _query: Q,
    ) -> Result<Q::Return, Error>
    where
        Q: Debug + Query,
    {
        if let Some(contract) = self.map.get(&id) {
            let module =
                Module::new(&self.wasmer_store, &contract.code).unwrap();
            let import_object = imports! {};
            let instance = Instance::new(&module, &import_object).unwrap();

            let check = instance.exports.get_function(Q::NAME).unwrap();

            let result = check.call(&[Value::I32(42)]);

            dbg!(result);

            todo!()
        } else {
            Err(Error::UnknownContract)
        }
    }

    pub fn transact<T>(
        &mut self,
        id: ContractId,
        _transaction: T,
    ) -> Result<T::Return, Error>
    where
        T: Debug + Transaction,
    {
        if let Some(contract) = self.map.get(&id) {
            let module =
                Module::new(&self.wasmer_store, &contract.code).unwrap();
            let import_object = imports! {};
            let instance = Instance::new(&module, &import_object).unwrap();

            let check = instance.exports.get_function(T::NAME).unwrap();

            let result = check.call(&[Value::I32(42)]);

            dbg!(result);

            todo!()
        } else {
            Err(Error::UnknownContract)
        }
    }
}
