// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use canonical::{Canon, EncodeToVec, Source, Store};
use canonical_derive::Canon;
use dusk_abi::{HostModule, Query, Transaction};
use dusk_hamt::Map;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
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

/// A read-only reference to a contract.
pub struct ContractRef<'id, 'guard> {
    contract_id: &'id ContractId,
    guard: RwLockReadGuard<'guard, NetworkStateInner>,
}

impl<'id, 'guard> ContractRef<'id, 'guard> {
    /// Gets the reference to the contract in the `head` state.
    pub fn get(&self) -> Result<impl Deref<Target = Contract> + '_, VMError> {
        self.guard.head.get_contract(self.contract_id)
    }

    /// Gets the reference to the contract in the `origin` state.
    pub fn get_origin(
        &self,
    ) -> Result<impl Deref<Target = Contract> + '_, VMError> {
        self.guard.origin.get_contract(self.contract_id)
    }
}

/// A mutable reference to a contract.
pub struct ContractMutRef<'id, 'guard> {
    contract_id: &'id ContractId,
    guard: RwLockWriteGuard<'guard, NetworkStateInner>,
}

impl<'id, 'guard> ContractMutRef<'id, 'guard> {
    /// Gets the reference to the contract in the `head` state.
    pub fn get(&self) -> Result<impl Deref<Target = Contract> + '_, VMError> {
        self.guard.head.get_contract(self.contract_id)
    }

    /// Gets the reference to the contract in the `origin` state.
    pub fn get_origin(
        &self,
    ) -> Result<impl Deref<Target = Contract> + '_, VMError> {
        self.guard.origin.get_contract(self.contract_id)
    }

    /// Gets the mutable reference to the contract in the `head` state.
    pub fn get_mut(
        &mut self,
    ) -> Result<impl DerefMut<Target = Contract> + '_, VMError> {
        self.guard.head.get_contract_mut(self.contract_id)
    }

    /// Gets the reference to the contract in the `origin` state.
    pub fn get_origin_mut(
        &mut self,
    ) -> Result<impl DerefMut<Target = Contract> + '_, VMError> {
        self.guard.origin.get_contract_mut(self.contract_id)
    }
}

/// The main network state.
///
/// It keeps two different states, the `origin` and the `head`. The `origin` is
/// the starting state, and `head` is origin with all the received transactions
/// applied.
///
/// It is possible to either [commit](`Self::commit`) to the `head` state,
/// turning it into the new `origin`, or [reset](`Self::reset`) it back to
/// `origin`.
#[derive(Clone, Default)]
pub struct NetworkState(Arc<RwLock<NetworkStateInner>>);

#[derive(Clone, Default)]
struct NetworkStateInner {
    origin: Contracts,
    head: Contracts,
    modules: HostModules,
    module_config: ModuleConfig,
}

impl From<NetworkStateInner> for NetworkState {
    fn from(nsi: NetworkStateInner) -> Self {
        Self(Arc::new(RwLock::new(nsi)))
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
        NetworkStateInner {
            module_config,
            ..NetworkStateInner::default()
        }
        .into()
    }

    /// Returns a reference to the specified contracts state in the `head`
    /// state.
    pub async fn get_contract<'id, 'guard>(
        &'guard self,
        contract_id: &'id ContractId,
    ) -> ContractRef<'id, 'guard> {
        ContractRef {
            guard: self.0.read().await,
            contract_id,
        }
    }

    /// Returns a mutable reference to the specified contracts state in the
    /// `head` state.
    pub async fn get_contract_mut<'id, 'guard>(
        &'guard mut self,
        contract_id: &'id ContractId,
    ) -> ContractMutRef<'id, 'guard> {
        ContractMutRef {
            guard: self.0.write().await,
            contract_id,
        }
    }

    /// Returns a reference to the map of registered host modules
    pub async fn modules(&self) -> HostModules {
        let guard = self.0.read().await;
        guard.modules.clone()
    }

    /// Deploys a contract to the `head` state, returning the address of the
    /// created contract or an error.
    pub async fn deploy(
        &mut self,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        let mut guard = self.0.write().await;
        let config = guard.module_config.clone();
        guard.head.deploy(contract, &config)
    }

    /// Deploys a contract to the `head` state with the given id / address.
    pub async fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        let mut guard = self.0.write().await;
        let config = guard.module_config.clone();
        guard.head.deploy_with_id(id, contract, &config)
    }

    /// Query the contract at `target` address in the `head` state.
    pub async fn query<A, R>(
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

        let mut guard = self.0.write().await;

        let modules = guard.modules.clone();
        let config = guard.module_config.clone();

        let mut context =
            CallContext::new(&mut guard.head, config, modules, block_height);

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

    /// Transact with the contract at `target` address in the `head` state,
    /// returning the result of the transaction.
    ///
    /// This will advance the `head` to the resultant state.
    pub async fn transact<A, R>(
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

        let mut guard = self.0.write().await;

        // Fork the current head state
        let mut fork = guard.head.clone();

        let modules = guard.modules.clone();
        let config = guard.module_config.clone();

        // Use the forked state to execute the transaction
        let mut context =
            CallContext::new(&mut fork, config, modules, block_height);

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
        guard.head = fork;

        Ok(ret)
    }

    /// Returns the root of the tree in the `head` state.
    pub async fn root(&self) -> [u8; 32] {
        let guard = self.0.read().await;
        guard.head.root()
    }

    /// Resets the `head` state to `origin`.
    pub async fn reset(&mut self) {
        let mut guard = self.0.write().await;
        guard.head = guard.origin.clone();
    }

    /// Commits to the `head` state, making it the new `origin`.
    pub async fn commit(&mut self) {
        let mut guard = self.0.write().await;
        guard.origin = guard.head.clone();
    }

    /// Register a host function handler.
    pub async fn register_host_module<M>(&mut self, module: M)
    where
        M: HostModule + 'static,
    {
        let mut guard = self.0.write().await;
        guard.modules.insert(module);
    }

    /// Gets the state of the given contract in the `head` state.
    pub async fn get_contract_cast_state<C>(
        &self,
        contract_id: &ContractId,
    ) -> Result<C, VMError>
    where
        C: Canon,
    {
        let guard = self.0.read().await;
        guard.head.get_contract(contract_id).map_or(
            Err(VMError::UnknownContract),
            |contract| {
                let mut source = Source::new((*contract).state().as_bytes());
                C::decode(&mut source).map_err(VMError::from_store_error)
            },
        )
    }

    /// Gets module config
    pub async fn get_module_config(&self) -> ModuleConfig {
        let guard = self.0.read().await;
        guard.module_config.clone()
    }
}
