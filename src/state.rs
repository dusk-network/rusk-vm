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
#[cfg(feature = "persistence")]
use microkelvin::{
    BackendCtor, Compound, DiskBackend, PersistError, PersistedId, Persistence,
};
use tracing::{trace, trace_span};

use crate::call_context::CallContext;
use crate::contract::{Contract, ContractId};
use crate::gas::GasMeter;
use crate::modules::{compile_module, HostModules};
use crate::VMError;

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
    ) -> Result<ContractId, VMError> {
        let id: ContractId = Store::hash(contract.bytecode()).into();
        self.deploy_with_id(id, contract)
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
    ) -> Result<ContractId, VMError> {
        self.0
            .insert(id, contract.instrument()?)
            .map_err(VMError::from_store_error)?;

        let inserted_contract = self.get_contract(&id)?;
        compile_module(inserted_contract.bytecode())?;

        Ok(id)
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
pub struct NetworkState {
    origin: Contracts,
    head: Contracts,
    modules: HostModules,
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

        Ok(Self {
            origin,
            head,
            modules: HostModules::default(),
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

    /// Returns the state of contracts in the `head`.
    pub(crate) fn head_mut(&mut self) -> &mut Contracts {
        &mut self.head
    }

    /// Returns a reference to the specified contracts state in the `head`
    /// state.
    pub fn get_contract<'a>(
        &'a self,
        contract_id: &ContractId,
    ) -> Result<impl Deref<Target = Contract> + 'a, VMError> {
        self.head.get_contract(contract_id)
    }

    /// Returns a mutable reference to the specified contracts state in the
    /// `origin` state.
    pub fn get_contract_mut<'a>(
        &'a mut self,
        contract_id: &ContractId,
    ) -> Result<impl DerefMut<Target = Contract> + 'a, VMError> {
        self.head.get_contract_mut(contract_id)
    }

    /// Returns a reference to the map of registered host modules
    pub fn modules(&self) -> &HostModules {
        &self.modules
    }

    #[cfg(feature = "persistence")]
    /// Persists the origin contracts stored on the [`NetworkState`] specifying
    /// a backend ctor function.
    pub fn persist(
        &self,
        ctor: fn() -> Result<DiskBackend, PersistError>,
    ) -> Result<PersistedId, PersistError> {
        Persistence::persist(&BackendCtor::new(ctor), &self.head.0)
    }

    #[cfg(feature = "persistence")]
    /// Given a [`PersistedId`] restores the [`Hamt`] which stores the contracts
    /// of the entire blockchain state.
    pub fn restore(mut self, id: PersistedId) -> Result<Self, PersistError> {
        let map = Map::from_generic(&id.restore()?)?;

        self.origin = Contracts(map);
        self.head = self.origin.clone();

        Ok(self)
    }

    /// Deploys a contract to the `head` state, returning the address of the
    /// created contract or an error.
    pub fn deploy(
        &mut self,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        self.head.deploy(contract)
    }

    /// Deploys a contract to the `head` state with the given id / address.
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        self.head.deploy_with_id(id, contract)
    }

    /// Query the contract at `target` address in the `head` state.
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

    /// Transact with the contract at `target` address in the `head` state,
    /// returning the result of the transaction.
    ///
    /// This will advance the `head` to the resultant state.
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

    /// Returns the root of the tree in the `head` state.
    pub fn root(&self) -> [u8; 32] {
        self.head.root()
    }

    /// Resets the `head` state to `origin`.
    pub fn reset(&mut self) {
        self.head = self.origin.clone();
    }

    /// Commits to the `head` state, making it the new `origin`.
    pub fn commit(&mut self) {
        self.origin = self.head.clone();
    }

    /// Register a host function handler.
    pub fn register_host_module<M>(&mut self, module: M)
    where
        M: HostModule + 'static,
    {
        self.modules.insert(module);
    }

    /// Gets the state of the given contract in the `head` state.
    pub fn get_contract_cast_state<C>(
        &self,
        contract_id: &ContractId,
    ) -> Result<C, VMError>
    where
        C: Canon,
    {
        self.head.get_contract(contract_id).map_or(
            Err(VMError::UnknownContract),
            |contract| {
                let mut source = Source::new((*contract).state().as_bytes());
                C::decode(&mut source).map_err(VMError::from_store_error)
            },
        )
    }
}
