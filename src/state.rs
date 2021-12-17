// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_hamt::{Hamt, Lookup};
use microkelvin::{BranchRef, BranchRefMut, Store, Stored, Ident, Offset, HostStore};

use rusk_uplink::{ContractId, HostModule, hash_mocker, HostRawStore};
use tracing::{trace, trace_span};

use crate::call_context::CallContext;
use crate::contract::{Contract, ContractRef};
use crate::gas::GasMeter;
use crate::modules::ModuleConfig;
use crate::modules::{compile_module, HostModules};
use crate::{Schedule, VMError};
use core::convert::Infallible;
use rkyv::{Archive, Serialize, Fallible};

// #[derive(Clone)]
// struct BogusStore;
//
// struct BogusStorage;
// impl Fallible for BogusStorage {
//     type Error = Infallible;
// }
//
// impl microkelvin::Store for BogusStore {
//     type Identifier = Offset;
//     type Storage = BogusStorage;
//
//     fn put<T>(&self, t: &T) -> Stored<T, Self>
//         where
//             T: Serialize<Self::Storage> {
//         Stored::new(self.clone(), Ident::new(Offset::new(1)))
//     }
//
//     /// Gets a reference to an archived value
//     fn get_raw<T>(&self, ident: &Ident<Self::Identifier, T>) -> &T::Archived
//         where
//             T: Archive {
//         let extended: &T::Archived =
//             unsafe { core::mem::transmute(0) };
//         extended
//     }
// }
//
// impl Fallible for BogusStore {
//     type Error = Infallible;
// }



/// State of the contracts on the network.
#[derive(Archive, Default, Clone)]
pub struct Contracts(Hamt<ContractId, Contract, (), HostStore>);

impl Contracts {
    /// Returns a reference to the specified contracts state.
    pub fn get_contract<'a>(
        &'a self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRef<'a, Contract>, VMError> {
        self.0.get(contract_id).ok_or(VMError::UnknownContract)
    }

    /// Returns a mutable reference to the specified contracts state.
    pub fn get_contract_mut<'a>(
        &'a mut self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRefMut<'a, Contract>, VMError> {
        self.0.get_mut(contract_id).ok_or(VMError::UnknownContract)
    }

    /// Deploys a contract to the state, returning the address of the created
    /// contract or an error.
    pub fn deploy(
        &mut self,
        contract: Contract,
        module_config: &ModuleConfig,
    ) -> Result<ContractId, VMError> {
        let id: ContractId = hash_mocker(contract.bytecode()).into();
        self.deploy_with_id(id, contract, module_config)
    }

    /// Deploys a contract with the given id to the state.
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
        module_config: &ModuleConfig,
    ) -> Result<ContractId, VMError> {
        self.0.insert(id, contract);

        let inserted_contract = self.get_contract(&id)?;
        compile_module(inserted_contract.leaf().bytecode(), module_config)?;

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
#[derive(Clone)]
pub struct NetworkState {
    origin: Contracts,
    head: Contracts,
    modules: HostModules,
    module_config: ModuleConfig,
    store: HostStore,
}

impl NetworkState {
    /// Returns a new empty [`NetworkState`].
    pub fn new(store: HostStore) -> Self {
        NetworkState {
            store,
            origin: Default::default(),
            head: Default::default(),
            modules: Default::default(),
            module_config: Default::default(),
        }
    }

    /// Returns a [`NetworkState`] based on a schedule
    pub fn with_schedule(store: HostStore, schedule: &Schedule) -> Self {
        let module_config = ModuleConfig::from_schedule(schedule);
        Self {
            store,
            module_config,
            origin: Default::default(),
            head: Default::default(),
            modules: Default::default(),
        }
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
    ) -> Result<impl BranchRef<'a, Contract>, VMError> {
        self.head.get_contract(contract_id)
    }

    /// Returns a mutable reference to the specified contracts state in the
    /// `origin` state.
    pub fn get_contract_mut<'a>(
        &'a mut self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRefMut<'a, Contract>, VMError> {
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
        self.head.deploy(contract, &self.module_config)
    }

    /// Deploys a contract to the `head` state with the given id / address.
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
    ) -> Result<ContractId, VMError> {
        self.head.deploy_with_id(id, contract, &self.module_config)
    }

    /// Query the contract at `target` address in the `head` state.
    pub fn query<A, R>(
        &mut self,
        target: ContractId,
        block_height: u64,
        query: A,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError> {
        let _span = trace_span!(
            "outer query",
            block_height = ?block_height,
            target = ?target,
            gas_limit = ?gas_meter.limit()
        );

        let mut context =
            CallContext::new(self, block_height);

        // let result = match context.query(target, todo!(), gas_meter) {
        //     Ok(result) => {
        //         trace!("query was successful");
        //         Ok(result)
        //     }
        //     Err(e) => {
        //         trace!("query returned an error: {}", e);
        //         Err(e)
        //     }
        // }

        todo!()
        // result.cast().map_err(|e| {
        //     trace!("failed casting to result type: {:?}", e);
        //     VMError::from_store_error(e)
        // })
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
    ) -> Result<R, VMError> {
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
            CallContext::new(&mut fork, block_height);

        // let result = match context.transact(target, todo!(), gas_meter) {
        //     Ok((_, result)) => {
        //         trace!("transact was successful");
        //         Ok(result)
        //     }
        //     Err(e) => {
        //         trace!("transact returned an error: {}", e);
        //         Err(e)
        //     }
        // }?;

        // let ret = result.cast().map_err(|e| {
        //     trace!("failed casting to result type: {:?}", e);
        //     VMError::from_store_error(e)
        // })?;

        todo!();

        // If we reach this point, everything went well and we can use the
        // updates made in the forked state.
        *self = fork;

        Ok(todo!())
    }

    /// Returns the root of the tree in the `head` state.
    pub fn root(&self) -> [u8; 32] {
        todo!()
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
        &mut self,
        contract_id: &ContractId,
    ) -> Result<C, VMError> {
        self.head.get_contract(contract_id).map_or(
            Err(VMError::UnknownContract),
            |contract| {
                // let mut source = Source::new((*contract).state().as_bytes());
                // C::decode(&mut source).map_err(VMError::from_store_error)
                todo!()
            },
        )
    }

    /// Gets module config
    pub fn get_module_config(&self) -> &ModuleConfig {
        &self.module_config
    }
}
