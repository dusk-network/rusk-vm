// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::ops::{Deref, DerefMut};

use canonical::{Canon, CanonError, EncodeToVec, Sink, Source, Store};
use canonical_derive::Canon;
use dusk_abi::{HostModule, Query, Transaction};
use dusk_hamt::Map;
use tracing::{trace, trace_span};

use crate::call_context::CallContext;
use crate::contract::{Contract, ContractId};
use crate::gas::GasMeter;
use crate::modules::ModuleConfig;
use crate::modules::{compile_module, HostModules};
use crate::{Schedule, VMError};

#[cfg(feature = "persistence")]
pub mod persist;

/// State of the contracts on the network.
#[derive(Clone, Default, Canon)]
pub struct Contracts(Map<ContractId, Contract>);

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;

/// A thread-safe singleton used to register host modules shared across all
/// network states and call contexts.
pub(crate) static HOST_MODULES: Lazy<Arc<RwLock<HostModules>>> =
    Lazy::new(|| Arc::new(RwLock::new(HostModules::default())));

impl Contracts {
    /// Returns a reference to the specified contracts state.
    pub fn get_contract<'a>(
        &'a self,
        contract_id: &ContractId,
    ) -> Result<impl Deref<Target = Contract> + 'a, VMError> {
        self.0
            .get(contract_id)
            .map_err(VMError::from_store_error)
            .transpose()
            .unwrap_or(Err(VMError::UnknownContract))
    }

    /// Returns a mutable reference to the specified contracts state.
    pub fn get_contract_mut<'a>(
        &'a mut self,
        contract_id: &ContractId,
    ) -> Result<impl DerefMut<Target = Contract> + 'a, VMError> {
        self.0
            .get_mut(contract_id)
            .map_err(VMError::from_store_error)
            .transpose()
            .unwrap_or(Err(VMError::UnknownContract))
    }

    /// Deploys a contract to the state, returning the address of the created
    /// contract or an error.
    pub fn deploy(
        &mut self,
        contract: Contract,
        module_config: &ModuleConfig,
    ) -> Result<ContractId, VMError> {
        let id: ContractId = Store::hash(contract.bytecode()).into();
        self.deploy_with_id(id, contract, module_config)
    }

    /// Computes the root of the contracts tree.
    pub fn root(&self) -> [u8; 32] {
        // FIXME This is terribly slow. It should be possible to get it directly
        //  from the tree. https://github.com/dusk-network/microkelvin/issues/85
        Store::hash(&self.0.encode_to_vec())
    }

    /// Deploys a contract with the given id to the state.
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
        module_config: &ModuleConfig,
    ) -> Result<ContractId, VMError> {
        self.0
            .insert(id, contract)
            .map_err(VMError::from_store_error)?;

        let inserted_contract = self.get_contract(&id)?;
        compile_module(inserted_contract.bytecode(), module_config)?;

        Ok(id)
    }
}

/// The main network state consists of three different states: `staged`, `head`
/// and `origin`.
///
/// Any changes is applied to the `staged` state, and it's only committed to
/// the `head` state when the [commit](`Self::commit`) method is called.
///
/// Calling the [push](`Self::push`) method will turn the `head` state into the
/// new `origin`. At this point, any (`staged`) changes that are not committed
/// will be discarded.
///
/// It's possible to manually discard the `staged` state by calling the
/// [unstage](`Self::unstage`) method, as well as to reset all the three state
/// to the `origin` by calling the [reset](`Self::reset`) method.
///
/// Due to the ephemeral nature of `staged` state, only `head` and `origin`
/// are persisted on disk.
#[derive(Clone, Default)]
pub struct NetworkState {
    pub(crate) staged: Contracts,
    head: Contracts,
    origin: Contracts,
    module_config: ModuleConfig,
}

/// Custom implementation of Canon ensuring only the `head` state is encoded.
/// When restored, `head` is set to be a copy of `origin` and the modules are to
/// be set by the caller.
impl Canon for NetworkState {
    fn encode(&self, sink: &mut Sink) {
        self.origin.encode(sink);
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        let origin = Contracts::decode(source)?;
        let head = origin.clone();
        let staged = origin.clone();

        Ok(Self {
            origin,
            head,
            staged,
            module_config: ModuleConfig::new(),
        })
    }

    fn encoded_len(&self) -> usize {
        Canon::encoded_len(&self.origin)
    }
}

impl NetworkState {
    /// Returns a new empty [`NetworkState`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a [`NetworkState`] based on a schedule
    pub fn with_schedule(schedule: &Schedule) -> Self {
        let module_config = ModuleConfig::from_schedule(schedule);
        Self {
            module_config,
            ..Self::default()
        }
    }

    /// Returns a reference to the specified contracts state
    pub fn get_contract<'a>(
        &'a self,
        contract_id: &ContractId,
    ) -> Result<impl Deref<Target = Contract> + 'a, VMError> {
        self.staged.get_contract(contract_id)
    }

    /// Returns a mutable reference to the specified contracts state
    pub fn get_contract_mut<'a>(
        &'a mut self,
        contract_id: &ContractId,
    ) -> Result<impl DerefMut<Target = Contract> + 'a, VMError> {
        self.staged.get_contract_mut(contract_id)
    }

    /// Deploys a contract, returning the address of the created contract or
    /// an error.
    pub fn deploy(
        &mut self,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        self.staged.deploy(contract, &self.module_config)
    }

    /// Deploys a contract with the given id / address.
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        self.staged
            .deploy_with_id(id, contract, &self.module_config)
    }

    /// Query the contract at `target` address
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
        let _span = trace_span!(
            "outer query",
            block_height = ?block_height,
            target = ?target,
            gas_limit = ?gas_meter.limit()
        );

        let mut context = CallContext::new(self, block_height);

        let result =
            match context.query(target, Query::from_canon(&query), gas_meter) {
                Ok(result) => {
                    trace!("query was successful");
                    Ok(result)
                }
                Err(e) => {
                    trace!("query returned an error: {}", e);
                    Err(e)
                }
            }?;

        result.cast().map_err(|e| {
            trace!("failed casting to result type: {:?}", e);
            VMError::from_store_error(e)
        })
    }

    /// Transact with the contract at `target` address returning the result of
    /// the transaction.
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
        let _span = trace_span!(
            "outer transact",
            block_height = ?block_height,
            target = ?target,
            gas_limit = ?gas_meter.limit(),
        );

        // Fork the current network's state
        let mut fork = self.clone();

        // Use the forked state to execute the transaction
        let mut context = CallContext::new(&mut fork, block_height);

        let result = match context.transact(
            target,
            Transaction::from_canon(&transaction),
            gas_meter,
        ) {
            Ok((_, result)) => {
                trace!("transact was successful");
                Ok(result)
            }
            Err(e) => {
                trace!("transact returned an error: {}", e);
                Err(e)
            }
        }?;

        let ret = result.cast().map_err(|e| {
            trace!("failed casting to result type: {:?}", e);
            VMError::from_store_error(e)
        })?;

        // If we reach this point, everything went well and we can use the
        // updates made in the forked state.
        *self = fork;

        Ok(ret)
    }

    /// Returns the root of the tree
    pub fn root(&self) -> [u8; 32] {
        self.staged.root()
    }

    /// Resets the state to `origin`
    pub fn reset(&mut self) {
        self.staged = self.origin.clone();
        self.head = self.origin.clone();
    }

    /// Unstage the state
    pub fn unstage(&mut self) {
        self.staged = self.head.clone();
    }

    /// Commits the `staged` state, making it the new `head`.
    pub fn commit(&mut self) {
        self.head = self.staged.clone();
    }

    /// Pushes the `head` state, making it the new `origin`.
    /// Anything in the `staged` that wasn't [commit](Self::commit)ed is
    /// lost.
    pub fn push(&mut self) {
        self.origin = self.head.clone();
        self.staged = self.head.clone();
    }

    /// Register a host function handler.
    pub fn register_host_module<M>(module: M)
    where
        M: HostModule + 'static + Sync + Send,
    {
        HOST_MODULES.write().insert(module);
    }

    /// Gets the state of the given contract in the `head` state.
    pub fn get_contract_cast_state<C>(
        &self,
        contract_id: &ContractId,
    ) -> Result<C, VMError>
    where
        C: Canon,
    {
        self.staged.get_contract(contract_id).map_or(
            Err(VMError::UnknownContract),
            |contract| {
                let mut source = Source::new((*contract).state().as_bytes());
                C::decode(&mut source).map_err(VMError::from_store_error)
            },
        )
    }

    /// Gets module config
    pub fn get_module_config(&self) -> &ModuleConfig {
        &self.module_config
    }
}
