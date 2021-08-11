use std::collections::HashMap as Map;
use std::fmt::Debug;

use rkyv::ser::serializers::BufferSerializer;
use rkyv::{Archive, Infallible, Serialize};
use wasmer::{imports, Instance, Module, Store, Value};

/// Default wasm memory pages are 64k large
pub const PAGE_SIZE: usize = 64 * 1024;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ContractId([u8; 32]);

#[derive(Debug)]
struct ContractInstance {
    pub data: Vec<u8>,
    pub code: Vec<u8>,
}

impl ContractInstance {
    fn id(&self) -> ContractId {
        ContractId(Default::default())
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
        let initial_memory = vec![0u8; PAGE_SIZE];
        let mut data = contract.serialize(&mut Infallible);

        let instance = ContractInstance {
            code: C::code().into(),
            data: initial_memory,
        };

        let id = instance.id();

        self.map.insert(id, instance);
        id
    }

    pub fn query<A, R>(&self, id: ContractId, method: &'static str, arg: A) -> Result<R, Error>
    where
        A: Debug,
    {
        if let Some(contract) = self.map.get(&id) {
            let module = Module::new(&self.wasmer_store, &contract.code).unwrap();
            let import_object = imports! {};
            let instance = Instance::new(&module, &import_object).unwrap();

            let check = instance.exports.get_function("check_hair").unwrap();

            let result = check.call(&[Value::I32(42)]);

            assert_eq!(result[0], Value::I32(43))
        } else {
            Err(Error::UnknownContract)
        }
    }
}

pub trait Contract {
    fn code() -> &'static [u8];
}
