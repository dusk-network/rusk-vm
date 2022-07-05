// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fmt;
use std::ops::Deref;

use dusk_hamt::{Hamt, Lookup};

use bytecheck::CheckBytes;
use microkelvin::{
    BranchRef, BranchRefMut, OffsetLen, StoreRef, StoreSerializer,
};
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{
    hash, ContractId, HostModule, Query, RawQuery, RawTransaction,
    StoreContext, Transaction,
};

use tracing::{trace, trace_span};

use crate::call_context::CallContext;
use crate::config::{Config, DEFAULT_CONFIG};
use crate::contract::Contract;
use crate::gas::GasMeter;
use crate::modules::{compile_module, HostModules};
use crate::VMError;

#[derive(Debug, Clone)]
pub struct Event {
    origin: ContractId,
    name: String,
    data: Vec<u8>,
}

impl Event {
    pub(crate) fn new(origin: ContractId, name: String, data: Vec<u8>) -> Self {
        Self { origin, name, data }
    }

    /// The Id of the smart contract originating the event.
    pub fn origin(&self) -> ContractId {
        self.origin
    }

    /// The name of the event.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// The data included with the event.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}

#[derive(Debug, Clone)]
pub struct Receipt<R> {
    ret: R,
    events: Vec<Event>,
}

impl<R> Receipt<R> {
    pub(crate) fn new(ret: R, events: Vec<Event>) -> Self {
        Self { ret, events }
    }

    /// The return of the smart contract call.
    pub fn ret(&self) -> &R {
        &self.ret
    }

    /// List of events emitted during smart contract execution, in order of
    /// emission.
    pub fn events(&self) -> &[Event] {
        &self.events
    }
}

impl<R> Deref for Receipt<R> {
    type Target = R;

    fn deref(&self) -> &R {
        &self.ret
    }
}

pub mod persist;

/// State of the contracts on the network.
#[derive(Archive, Default, Clone)]
pub struct Contracts(Hamt<ContractId, Contract, (), OffsetLen>);

impl Contracts {
    /// Returns a reference to the specified contracts state.
    pub fn get_contract(
        &self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRef<Contract>, VMError> {
        self.0
            .get(contract_id)
            .ok_or(VMError::UnknownContract(*contract_id))
    }

    /// Returns a mutable reference to the specified contracts state.
    pub fn get_contract_mut(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRefMut<Contract>, VMError> {
        self.0
            .get_mut(contract_id)
            .ok_or(VMError::UnknownContract(*contract_id))
    }

    /// Deploys a contract to the state, returning the address of the created
    /// contract or an error.
    pub fn deploy(
        &mut self,
        contract: Contract,
        config: &'static Config,
    ) -> Result<ContractId, VMError> {
        let id: ContractId = hash(contract.bytecode()).into();
        self.deploy_with_id(id, contract, config)
    }

    /// Deploys a contract with the given id to the state.
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
        config: &'static Config,
    ) -> Result<ContractId, VMError> {
        compile_module(contract.bytecode(), config)?;

        self.0.insert(id, contract);

        Ok(id)
    }
}

/// The main network state.
///
/// Use [`query`] and [`transact`] to interact with the state.
#[derive(Clone)]
pub struct NetworkState {
    contracts: Contracts,
    modules: HostModules,
    store: StoreContext,
    config: &'static Config,
}

impl NetworkState {
    /// Returns a new empty [`NetworkState`] with the default configuration.
    pub fn new(store: StoreContext) -> Self {
        Self::with_config(store, &DEFAULT_CONFIG)
    }

    /// Returns a new empty [`NetworkState`] with the given configuration.
    pub fn with_config(store: StoreContext, config: &'static Config) -> Self {
        NetworkState {
            contracts: Default::default(),
            modules: Default::default(),
            store,
            config,
        }
    }

    /// Returns the configuration of this instance.
    pub fn config(&self) -> &'static Config {
        self.config
    }

    /// Returns a reference to the specified contracts state in the state.
    pub fn get_contract(
        &self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRef<Contract>, VMError> {
        self.contracts.get_contract(contract_id)
    }

    /// Returns a mutable reference to the specified contracts state in the
    /// state.
    pub fn get_contract_mut(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRefMut<Contract>, VMError> {
        self.contracts.get_contract_mut(contract_id)
    }

    /// Returns a reference to the map of registered host modules
    pub fn modules(&self) -> &HostModules {
        &self.modules
    }

    /// Deploys a contract to the state, returning the address of the
    /// created contract or an error.
    pub fn deploy(
        &mut self,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        self.contracts.deploy(contract, self.config)
    }

    /// Deploys a contract to the state with the given id / address.
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        self.contracts.deploy_with_id(id, contract, self.config)
    }

    /// Query the contract at `target` address in the state, returning the query
    /// receipt.
    pub fn query<Q>(
        &self,
        target: ContractId,
        block_height: u64,
        query: Q,
        gas_meter: &mut GasMeter,
    ) -> Result<Receipt<Q::Return>, VMError>
    where
        Q: Query + Serialize<StoreSerializer<OffsetLen>>,
        Q::Return: Archive,
        <Q::Return as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
            + Deserialize<Q::Return, StoreRef<OffsetLen>>,
    {
        let _span = trace_span!(
            "outer query",
            block_height = ?block_height,
            target = ?target,
            gas_limit = ?gas_meter.limit()
        );

        let mut state = self.clone();
        let store = self.store.clone();

        let mut context =
            CallContext::new(&mut state, block_height, self.store.clone());

        let result = match context.query(
            target,
            RawQuery::new(query, &store),
            gas_meter,
        ) {
            Ok(result) => {
                trace!("query was successful");
                Ok(result)
            }
            Err(e) => {
                trace!("query returned an error: {}", e);
                Err(e)
            }
        }?;

        let cast = result
            .cast::<Q::Return>()
            .map_err(|_| VMError::InvalidData)?;

        let events = context.take_events();
        let ret: Q::Return = cast
            .deserialize(&mut self.store.clone())
            .expect("Infallible");

        Ok(Receipt::new(ret, events))
    }

    /// Transact with the contract at `target` address in the state, returning
    /// the transaction receipt and the resultant state.
    pub fn transact<T>(
        &self,
        target: ContractId,
        block_height: u64,
        transaction: T,
        gas_meter: &mut GasMeter,
    ) -> Result<(Receipt<T::Return>, NetworkState), VMError>
    where
        T: Transaction + Serialize<StoreSerializer<OffsetLen>>,
        T::Return: Archive,
        <T::Return as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
            + Deserialize<T::Return, StoreRef<OffsetLen>>,
    {
        let _span = trace_span!(
            "outer transact",
            block_height = ?block_height,
            target = ?target,
            gas_limit = ?gas_meter.limit(),
        );

        // Fork the current network's state
        let mut fork = self.clone();

        // Use the forked state to execute the transaction
        let mut context =
            CallContext::new(&mut fork, block_height, self.store.clone());

        let result = match context.transact(
            target,
            RawTransaction::new(transaction, &self.store),
            gas_meter,
        ) {
            Ok(result) => {
                trace!("query was successful");
                Ok(result)
            }
            Err(e) => {
                trace!("query returned an error: {}", e);
                Err(e)
            }
        }?;

        let cast = result
            .cast::<T::Return>()
            .map_err(|_| VMError::InvalidData)?;

        let events = context.take_events();
        let ret: T::Return = cast
            .deserialize(&mut self.store.clone())
            .expect("Infallible");

        Ok((Receipt::new(ret, events), fork))
    }

    /// Returns the root of the contracts tree.
    pub fn root(&self) -> [u8; 32] {
        todo!()
    }

    /// Register a host function handler.
    pub fn register_host_module<M>(&mut self, module: M)
    where
        M: HostModule + 'static,
    {
        self.modules.insert(module);
    }

    /// Gets the state of the given contract.
    pub fn get_contract_cast_state<C>(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<C, VMError> {
        self.contracts.get_contract(contract_id).map_or(
            Err(VMError::UnknownContract(*contract_id)),
            |_contract| {
                // let mut source = Source::new((*contract).state().as_bytes());
                // C::decode(&mut source).map_err(VMError::from_store_error)
                todo!()
            },
        )
    }
}

impl fmt::Debug for NetworkState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetworkState")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
