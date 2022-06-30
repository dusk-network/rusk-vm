// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_hamt::Hamt;
use microkelvin::{HostStore, Ident, OffsetLen, StoreRef};
use rkyv::ser::{serializers::AllocSerializer, Serializer};
use rkyv::{archived_root, Archive, Deserialize, Infallible, Serialize};
use rusk_uplink::ContractId;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

use crate::contract::Contract;
use crate::state::{Contracts, NetworkState};
use crate::VMError;

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
    contracts: OffsetLen,
}

impl NetworkStateId {
    /// Read from the given path a [`NetworkStateId`]
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, VMError> {
        let buf = fs::read(&path).map_err(PersistError::Io)?;
        let id = unsafe { archived_root::<NetworkStateId>(buf.as_slice()) };
        let id: NetworkStateId = id.deserialize(&mut Infallible).unwrap();
        Ok(id)
    }

    /// Write to the given path a [`NetworkStateId`]
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), VMError> {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(self).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        fs::write(&path, bytes.as_slice()).map_err(PersistError::Io)?;
        Ok(())
    }
}

impl NetworkState {
    const PERSISTENCE_ID_FILE_NAME: &'static str = "persist_id";

    /// Persists the origin contracts stored on the [`NetworkState`], together
    /// with their configuration
    pub fn persist(
        &self,
        store: StoreRef<OffsetLen>,
    ) -> Result<NetworkStateId, VMError> {
        let contracts_stored = store.store(&self.contracts.0);
        store.persist().map_err(|_| {
            PersistError::Store(String::from(
                "Store persistence failed for network state",
            ))
        })?;
        Ok(NetworkStateId {
            contracts: *contracts_stored.ident().erase(),
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

    /// Given a [`NetworkStateId`] restores both [`Hamt`]s which store the
    /// contract - the entire blockchain state - together with configuration.
    pub fn restore(
        mut self,
        store: StoreRef<OffsetLen>,
        id: NetworkStateId,
    ) -> Result<Self, VMError> {
        let contracts_ident = Ident::<
            Hamt<ContractId, Contract, (), OffsetLen>,
            OffsetLen,
        >::new(id.contracts);

        let restored_contracts: &<Hamt<ContractId, Contract, (), OffsetLen> as Archive>::Archived =
            store.get::<Hamt<ContractId, Contract, (), OffsetLen>>(&contracts_ident);

        let restored_contracts: Hamt<ContractId, Contract, (), OffsetLen> =
            restored_contracts.deserialize(&mut store.clone()).unwrap();

        self.contracts = Contracts(restored_contracts);

        Ok(self)
    }

    /// Restores network state
    /// given source disk path.
    pub fn restore_from_disk<P: AsRef<Path>>(
        source_store_path: P,
    ) -> Result<Self, VMError> {
        let store = StoreRef::new(
            HostStore::with_file(source_store_path.as_ref())
                .map_err(PersistError::Io)?,
        );
        let file_path = source_store_path
            .as_ref()
            .join(Self::PERSISTENCE_ID_FILE_NAME);
        let persistence_id = NetworkStateId::read(file_path)?;
        NetworkState::new(store.clone()).restore(store, persistence_id)
    }
}
