// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_hamt::Hamt;
use microkelvin::{
    All, Compound, HostStore, Ident, Keyed, OffsetLen, StoreRef,
};
use rkyv::ser::{serializers::AllocSerializer, Serializer};
use rkyv::{archived_root, Archive, Deserialize, Infallible, Serialize};
use rusk_uplink::ContractId;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

use crate::contract::Contract;
use crate::state::{Contracts, NetworkState};
use crate::{GasMeter, VMError};

/// An error that can happen when persisting structures to disk
#[derive(Error, Debug)]
pub enum PersistError {
    /// An io-error occurred while persisting
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Store persistence error
    #[error("{0}")]
    Store(String),
}

/// The [`NetworkStateId`] is the persisted id of the [`NetworkState`]
#[derive(Archive, Serialize, Deserialize, Default, Clone, Debug)]
pub struct NetworkStateId {
    origin: OffsetLen,
    head: OffsetLen,
}

impl NetworkStateId {
    /// Read from the given path a [`NetworkStateId`]
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, VMError> {
        let buf = fs::read(&path).map_err(|e| PersistError::Io(e))?;
        let id = unsafe { archived_root::<NetworkStateId>(buf.as_slice()) };
        let id: NetworkStateId = id.deserialize(&mut Infallible).unwrap();
        Ok(id)
    }

    /// Write to the given path a [`NetworkStateId`]
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), VMError> {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(self).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        fs::write(&path, bytes.as_slice()).map_err(|e| PersistError::Io(e))?;
        Ok(())
    }
}

impl NetworkState {
    const PERSISTENCE_ID_FILE_NAME: &'static str = "persist_id";

    /// Persists the origin contracts stored on the [`NetworkState`]
    pub fn persist(
        &self,
        store: StoreRef<OffsetLen>,
    ) -> Result<NetworkStateId, VMError> {
        let head_stored = store.store(&self.head.0);
        let origin_stored = store.store(&self.origin.0);
        store.persist().map_err(|_| {
            PersistError::Store(String::from(
                "Store persistence failed for network state",
            ))
        })?;
        Ok(NetworkStateId {
            head: *head_stored.ident().erase(),
            origin: *origin_stored.ident().erase(),
        })
    }

    /// Persists the state to disk along with persistence id
    pub fn persist_to_disk<P: AsRef<Path>>(
        &self,
        store: StoreRef<OffsetLen>,
        store_path: P,
    ) -> Result<NetworkStateId, VMError> {
        let persistence_id = self.persist(store)?;

        let file_path =
            store_path.as_ref().join(Self::PERSISTENCE_ID_FILE_NAME);

        persistence_id.write(file_path)?;

        Ok(persistence_id)
    }

    /// Consolidates the state to disc,
    /// given the source disc path.
    pub fn consolidate_to_disk<P: AsRef<Path>>(
        source_store_path: P,
        target_store_path: P,
        gas_meter: &mut GasMeter,
    ) -> Result<NetworkStateId, VMError> {
        let source_store = StoreRef::new(
            HostStore::with_file(source_store_path.as_ref())
                .map_err(|e| PersistError::Io(e))?,
        );
        let target_store = StoreRef::new(
            HostStore::with_file(target_store_path.as_ref())
                .map_err(|e| PersistError::Io(e))?,
        );

        let source_persistence_id_file_path = source_store_path
            .as_ref()
            .join(Self::PERSISTENCE_ID_FILE_NAME);
        let source_persistence_id =
            NetworkStateId::read(source_persistence_id_file_path.clone())?;

        let mut network = NetworkState::with_target_store(
            source_store.clone(),
            target_store.clone(),
        )
        .restore(source_store.clone(), source_persistence_id)?;

        network.store_contract_states(gas_meter)?;
        let target_persistence_id = network.persist(target_store.clone())?;

        let target_persistence_id_file_path = target_store_path
            .as_ref()
            .join(Self::PERSISTENCE_ID_FILE_NAME);
        target_persistence_id.write(target_persistence_id_file_path)?;

        Ok(target_persistence_id)
    }

    /// Given a [`NetworkStateId`] restores both [`Hamt`] which store
    /// contracts of the entire blockchain state.
    pub fn restore(
        mut self,
        store: StoreRef<OffsetLen>,
        id: NetworkStateId,
    ) -> Result<Self, VMError> {
        let head_ident = Ident::<
            Hamt<ContractId, Contract, (), OffsetLen>,
            OffsetLen,
        >::new(id.head);
        let origin_ident = Ident::<
            Hamt<ContractId, Contract, (), OffsetLen>,
            OffsetLen,
        >::new(id.origin);

        let restored_head: &<Hamt<ContractId, Contract, (), OffsetLen> as Archive>::Archived =
            store.get::<Hamt<ContractId, Contract, (), OffsetLen>>(&head_ident);
        let restored_origin: &<Hamt<ContractId, Contract, (), OffsetLen> as Archive>::Archived =
            store.get::<Hamt<ContractId, Contract, (), OffsetLen>>(&origin_ident);

        let restored_head: Hamt<ContractId, Contract, (), OffsetLen> =
            restored_head.deserialize(&mut store.clone()).unwrap();
        let restored_origin: Hamt<ContractId, Contract, (), OffsetLen> =
            restored_origin.deserialize(&mut store.clone()).unwrap();

        self.origin = Contracts(restored_origin);

        self.head = Contracts(restored_head);

        self.staged = self.head.clone();

        Ok(self)
    }

    /// Restores network state
    /// given source disk path.
    pub fn restore_from_disk<P: AsRef<Path>>(
        source_store_path: P,
    ) -> Result<Self, VMError> {
        let store = StoreRef::new(
            HostStore::with_file(source_store_path.as_ref())
                .map_err(|e| PersistError::Io(e))?,
        );
        let file_path = source_store_path
            .as_ref()
            .join(Self::PERSISTENCE_ID_FILE_NAME);
        let persistence_id = NetworkStateId::read(file_path.clone())?;
        NetworkState::new(store.clone()).restore(store, persistence_id)
    }

    /// Store contracts' states
    pub fn store_contract_states(
        &mut self,
        gas_meter: &mut GasMeter,
    ) -> Result<(), VMError> {
        let mut contract_ids: Vec<ContractId> = vec![];

        let branch = self.head.0.walk(All).expect("Some(_)");
        for leaf in branch {
            let val = leaf.key();
            contract_ids.push(*val);
        }

        for contract_id in contract_ids {
            self.transact_store_state(contract_id, 0, gas_meter)
                .unwrap_or(());
        }

        Ok(())
    }
}
