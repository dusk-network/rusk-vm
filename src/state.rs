// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use canonical::{ByteSource, Canon, Ident, Sink, Source, Store};
use dusk_abi::{HostModule, Query, Transaction};
use dusk_kelvin_map::Map;

use crate::call_context::CallContext;
use crate::contract::{Contract, ContractId, ContractInstrumenter};
use crate::gas::GasMeter;
use crate::VMError;

type BoxedHostModule<S> = Box<dyn HostModule<S>>;

/// The main network state, includes the full state of contracts.
#[derive(Clone, Default)]
pub struct NetworkState<S>
where
    S: Store,
{
    block_height: u64,
    contracts: Map<ContractId, Contract, S>,
    modules: Rc<RefCell<HashMap<ContractId, BoxedHostModule<S>>>>,
    store: S,
}

// Manual implementation of `Canon` to ignore the "modules" which needs to be
// re-instantiated on program initialization.
impl<S> Canon<S> for NetworkState<S>
where
    S: Store,
{
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        self.block_height.write(sink)?;
        self.contracts.write(sink)
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let block_height = u64::read(source)?;
        let contracts = Map::read(source)?;
        Ok(NetworkState {
            block_height,
            contracts,
            store: source.store().clone(),
            modules: Rc::new(RefCell::new(HashMap::new())),
        })
    }

    fn encoded_len(&self) -> usize {
        Canon::<S>::encoded_len(&self.block_height)
            + Canon::<S>::encoded_len(&self.contracts)
    }
}

impl<S> NetworkState<S>
where
    S: Store,
{
    /// Returns a [`NetworkState`] for a specific block height
    pub fn with_block_height(block_height: u64) -> Self {
        Self {
            block_height,
            contracts: Map::default(),
            modules: Rc::new(RefCell::new(HashMap::new())),
            store: S::default(),
        }
    }

    /// Deploys a contract to the state, returns the address of the created
    /// contract or an error.
    /// Before the deployment happens the contract's bytecode is instrumented
    /// and then stored into the NetworkState
    pub fn deploy(
        &mut self,
        mut contract: Contract,
    ) -> Result<ContractId, VMError<S>> {
        let schedule = crate::Schedule::default();
        let mut instrumenter =
            ContractInstrumenter::new(contract.bytecode(), &schedule)?;

        // Apply instrumentation & validate the module.
        instrumenter.apply_module_config()?;

        let id: ContractId =
            S::Ident::from_bytes(instrumenter.bytecode()?.as_ref()).into();

        // Assign to the Contract that we're going to store the instrumented
        // bytecode.
        contract.code = instrumenter.bytecode()?.clone();

        // FIXME: This shoul check wether the contract is already deployed.
        let _ = self
            .contracts
            .insert(id.clone(), contract)
            .map_err(|e| VMError::StoreError(e))?;
        Ok(id)
    }

    /// Returns a reference to the specified contracts state
    pub fn get_contract<'a>(
        &'a self,
        contract_id: &ContractId,
    ) -> Result<impl Deref<Target = Contract> + 'a, VMError<S>> {
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
    ) -> Result<impl DerefMut<Target = Contract> + 'a, VMError<S>> {
        self.contracts
            .get_mut(contract_id)
            .map_err(VMError::from_store_error)
            .transpose()
            .unwrap_or(Err(VMError::UnknownContract))
    }

    /// Returns a reference to the map of registered host modules
    pub fn modules(
        &self,
    ) -> &Rc<RefCell<HashMap<ContractId, BoxedHostModule<S>>>> {
        &self.modules
    }

    /// Returns a reference to the store backing the state
    pub fn store(&self) -> &S {
        &self.store
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
    ) -> Result<R, VMError<S>>
    where
        A: Canon<S>,
        R: Canon<S>,
    {
        let store = self.store().clone();
        let mut context = CallContext::new(self, gas_meter, &store)?;

        let result = context.query(
            target,
            Query::from_canon(&query, &store)
                .map_err(VMError::from_store_error)?,
        )?;

        result.cast(store).map_err(VMError::from_store_error)
    }

    /// Transact with the contract at address `target`
    pub fn transact<A, R>(
        &mut self,
        target: ContractId,
        transaction: A,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError<S>>
    where
        A: Canon<S>,
        R: Canon<S>,
    {
        let store = self.store().clone();

        // Fork the current network's state
        let mut fork = self.clone();

        // Use the forked state to execute the transaction
        let mut context = CallContext::new(&mut fork, gas_meter, &store)?;

        let (_, result) = context.transact(
            target,
            Transaction::from_canon(&transaction, &store)
                .map_err(VMError::from_store_error)?,
        )?;

        let ret = result.cast(store).map_err(VMError::from_store_error)?;

        // If we reach this point, everything went well and we can use the
        // updates made in the forked state.
        *self = fork;

        Ok(ret)
    }

    /// Register a host-fn handler
    pub fn register_host_module<M>(&mut self, module: M)
    where
        M: HostModule<S> + 'static,
    {
        self.modules
            .borrow_mut()
            .insert(module.module_id(), Box::new(module));
    }

    /// Gets the state of the given contract
    pub fn get_contract_cast_state<C>(
        &self,
        contract_id: &ContractId,
    ) -> Result<C, VMError<S>>
    where
        C: Canon<S>,
    {
        self.contracts
            .get(contract_id)
            .map_err(VMError::from_store_error)?
            .map_or(Err(VMError::UnknownContract), |contract| {
                let mut source = ByteSource::new(
                    (*contract).state().as_bytes(),
                    &self.store,
                );
                Canon::<S>::read(&mut source).map_err(VMError::from_store_error)
            })
    }
}
