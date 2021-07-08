// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use canonical::{Canon, CanonError, Sink, Source, Store};
use dusk_abi::{HostModule, Query, Transaction};
use dusk_hamt::Hamt;
#[cfg(feature = "persistance")]
use microkelvin::{
    BackendCtor, DiskBackend, PersistError, Persistance as Persistence,
    PersistedId,
};

use crate::call_context::CallContext;
use crate::contract::{Contract, ContractId};
use crate::gas::GasMeter;
use crate::VMError;

type BoxedHostModule = Box<dyn HostModule>;

/// The main network state, includes the full state of contracts.
#[derive(Clone, Default)]
pub struct NetworkState {
    block_height: u64,
    contracts: Hamt<ContractId, Contract, ()>,
    modules: Rc<RefCell<HashMap<ContractId, BoxedHostModule>>>,
}

// Manual implementation of `Canon` to ignore the "modules" which needs to be
// re-instantiated on program initialization.
impl Canon for NetworkState {
    fn encode(&self, sink: &mut Sink) {
        self.block_height.encode(sink);
        self.contracts.encode(sink);
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        Ok(NetworkState {
            block_height: u64::decode(source)?,
            contracts: Hamt::decode(source)?,
            modules: Rc::new(RefCell::new(HashMap::new())),
        })
    }

    fn encoded_len(&self) -> usize {
        Canon::encoded_len(&self.block_height)
            + Canon::encoded_len(&self.contracts)
    }
}

impl NetworkState {
    /// Returns a [`NetworkState`] for a specific block height
    pub fn with_block_height(block_height: u64) -> Self {
        Self {
            block_height,
            contracts: Hamt::default(),
            modules: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Mutates the `block_height` attr of the current `NetworkState` instance.
    ///
    /// That method is temporary, and will not be mantained in the API on the
    /// next versions.
    pub fn set_block_height(&mut self, block_height: u64) {
        self.block_height = block_height;
    }

    #[cfg(feature = "persistance")]
    /// Persists the contracts stored on the [`NetworkState`] specifying a
    /// backend ctor function.
    pub fn persist(
        &self,
        ctor: fn() -> Result<DiskBackend, PersistError>,
    ) -> Result<PersistedId, PersistError> {
        Persistence::persist(&BackendCtor::new(ctor), &self.contracts)
    }

    #[cfg(feature = "persistance")]
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

        self.contracts
            .insert(id, contract.instrument()?)
            .map_err(VMError::from_store_error)?;
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

    /// Returns the state's block height
    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    /// Queryn the contract at address `target`
    pub fn query<A, R>(
        &mut self,
        target: ContractId,
        query: A,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError>
    where
        A: Canon,
        R: Canon,
    {
        let mut context = CallContext::new(self, gas_meter);

        let result = context.query(target, Query::from_canon(&query))?;

        result.cast().map_err(VMError::from_store_error)
    }

    /// Transact with the contract at address `target`
    pub fn transact<A, R>(
        &mut self,
        target: ContractId,
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
        let mut context = CallContext::new(&mut fork, gas_meter);

        let (_, result) =
            context.transact(target, Transaction::from_canon(&transaction))?;

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
}
