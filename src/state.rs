// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use cached::proc_macro::cached;
use canonical::{Canon, CanonError, Sink, Source, Store};
use dusk_abi::{HostModule, Query, Transaction};
use dusk_hamt::Hamt;
#[cfg(feature = "persistence")]
use microkelvin::{
    BackendCtor, Compound, DiskBackend, PersistError, PersistedId, Persistence,
};
use wasmer::Module;

use crate::call_context::CallContext;
use crate::compiler::WasmerCompiler;
use crate::contract::{Contract, ContractId};
use crate::gas::GasMeter;
use crate::module_config::ModuleConfig;
use crate::{Schedule, VMError};

type BoxedHostModule = Box<dyn HostModule>;

/// The main network state, includes the full state of contracts.
#[derive(Clone, Default)]
pub struct NetworkState {
    contracts: Hamt<ContractId, Contract, ()>,
    modules: Rc<RefCell<HashMap<ContractId, BoxedHostModule>>>,
    module_cache: Arc<Mutex<HashMap<ContractId, Module>>>,
    module_config: ModuleConfig,
}

// Manual implementation of `Canon` to ignore the "modules" which needs to be
// re-instantiated on program initialization.
impl Canon for NetworkState {
    fn encode(&self, sink: &mut Sink) {
        self.contracts.encode(sink);
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        Ok(NetworkState {
            contracts: Hamt::decode(source)?,
            modules: Rc::new(RefCell::new(HashMap::new())),
            module_cache: Arc::new(Mutex::new(HashMap::new())),
            module_config: ModuleConfig::new(),
        })
    }

    fn encoded_len(&self) -> usize {
        Canon::encoded_len(&self.contracts)
    }
}

#[cached(size = 2048, time = 86400, result = true, sync_writes = true)]
fn get_or_create_module(bytecode: Vec<u8>) -> Result<Module, VMError> {
    let new_module = WasmerCompiler::create_module(bytecode)?;
    Ok(new_module)
}

impl NetworkState {
    /// Returns a new empty [`NetworkState`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a [`NetworkState`] based on a specific configuration file
    pub fn with_config_file(
        file_path: Option<String>,
    ) -> Result<Self, VMError> {
        let module_config = ModuleConfig::from_file(file_path)?;
        Ok(NetworkState::create(&module_config))
    }

    /// Returns a [`NetworkState`] based on a schedule
    pub fn with_schedule(schedule: &Schedule) -> Self {
        let module_config = ModuleConfig::from_schedule(schedule);
        NetworkState::create(&module_config)
    }

    fn create(module_config: &ModuleConfig) -> Self {
        Self {
            contracts: Hamt::default(),
            modules: Rc::new(RefCell::new(HashMap::new())),
            module_cache: Arc::new(Mutex::new(HashMap::new())),
            module_config: module_config.clone(),
        }
    }

    #[cfg(feature = "persistence")]
    /// Persists the contracts stored on the [`NetworkState`] specifying a
    /// backend ctor function.
    pub fn persist(
        &self,
        ctor: fn() -> Result<DiskBackend, PersistError>,
    ) -> Result<PersistedId, PersistError> {
        Persistence::persist(&BackendCtor::new(ctor), &self.contracts)
    }

    #[cfg(feature = "persistence")]
    /// Given a [`PersistedId`] restores the [`Hamt`] which stores the contracts
    /// of the entire blockchain state.
    pub fn restore(mut self, id: PersistedId) -> Result<Self, PersistError> {
        self.contracts = Hamt::from_generic(&id.restore()?)?;
        Ok(self)
    }

    /// Deploys a contract to the state, returns the address of the created
    /// contract or an error
    pub fn deploy(
        &mut self,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        let id: ContractId = Store::hash(contract.bytecode()).into();

        self.deploy_with_id(id, contract)
    }

    /// Deploys a contract to the state with the given id / address
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        self.contracts
            .insert(id, contract.instrument(&self.module_config)?)
            .map_err(VMError::from_store_error)?;
        let inserted_contract = self.get_contract(&id)?;
        self.get_module_from_cache(&id, inserted_contract.bytecode())?;
        Ok(id)
    }

    /// Returns a reference to the specified contracts state
    pub fn get_contract<'a>(
        &'a self,
        contract_id: &ContractId,
    ) -> Result<impl Deref<Target = Contract> + 'a, VMError> {
        self.contracts
            .get(contract_id)
            .map_err(VMError::from_store_error)
            .transpose()
            .unwrap_or(Err(VMError::UnknownContract))
    }

    /// Returns a reference to the specified contracts state
    pub fn get_contract_mut<'a>(
        &'a mut self,
        contract_id: &ContractId,
    ) -> Result<impl DerefMut<Target = Contract> + 'a, VMError> {
        self.contracts
            .get_mut(contract_id)
            .map_err(VMError::from_store_error)
            .transpose()
            .unwrap_or(Err(VMError::UnknownContract))
    }

    /// Returns a reference to the map of registered host modules
    pub fn modules(
        &self,
    ) -> &Rc<RefCell<HashMap<ContractId, BoxedHostModule>>> {
        &self.modules
    }

    /// Queryn the contract at address `target`
    pub fn query<A, R>(
        &mut self,
        target: ContractId,
        block_height: u64,
        query: A,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError>
    where
        A: Canon,
        R: Canon,
    {
        let mut context = CallContext::new(self, block_height);

        let result =
            context.query(target, Query::from_canon(&query), gas_meter)?;

        result.cast().map_err(VMError::from_store_error)
    }

    /// Transact with the contract at address `target`
    pub fn transact<A, R>(
        &mut self,
        target: ContractId,
        block_height: u64,
        transaction: A,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError>
    where
        A: Canon,
        R: Canon,
    {
        // Fork the current network's state
        let mut fork = self.clone();

        // Use the forked state to execute the transaction
        let mut context = CallContext::new(&mut fork, block_height);

        let (_, result) = context.transact(
            target,
            Transaction::from_canon(&transaction),
            gas_meter,
        )?;

        let ret = result.cast().map_err(VMError::from_store_error)?;

        // If we reach this point, everything went well and we can use the
        // updates made in the forked state.
        *self = fork;

        Ok(ret)
    }

    /// Register a host-fn handler
    pub fn register_host_module<M>(&mut self, module: M)
    where
        M: HostModule + 'static,
    {
        self.modules
            .borrow_mut()
            .insert(module.module_id(), Box::new(module));
    }

    /// Gets the state of the given contract
    pub fn get_contract_cast_state<C>(
        &self,
        contract_id: &ContractId,
    ) -> Result<C, VMError>
    where
        C: Canon,
    {
        self.contracts
            .get(contract_id)
            .map_err(VMError::from_store_error)?
            .map_or(Err(VMError::UnknownContract), |contract| {
                let mut source = Source::new((*contract).state().as_bytes());
                C::decode(&mut source).map_err(VMError::from_store_error)
            })
    }

    /// Retrieves module from cache possibly creating and storing a new one if
    /// not found
    pub fn get_module_from_cache(
        &self,
        _contract_id: &ContractId,
        bytecode: &[u8],
    ) -> Result<Module, VMError> {
        get_or_create_module(bytecode.to_vec())
    }
}
