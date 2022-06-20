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
use std::io::ErrorKind;
use std::path::Path;

use crate::contract::Contract;
use crate::state::{Contracts, NetworkState};
use crate::{GasMeter, VMError};

/// The [`NetworkStateId`] is the persisted id of the [`NetworkState`]
#[derive(Archive, Serialize, Deserialize, Default, Clone, Debug)]
pub struct NetworkStateId {
    origin: OffsetLen,
    head: OffsetLen,
}

impl NetworkStateId {
    /// Read from the given path a [`NetworkStateId`]
    pub fn read<P>(path: P) -> Result<Self, VMError>
    where
        P: AsRef<Path>,
    {
        let buf = fs::read(&path)?;
        let id = unsafe { archived_root::<NetworkStateId>(buf.as_slice()) };
        let id: NetworkStateId = id.deserialize(&mut Infallible).unwrap();
        Ok(id)
    }

    /// Write to the given path a [`NetworkStateId`]
    pub fn write<P>(&self, path: P) -> Result<(), VMError>
    where
        P: AsRef<Path>,
    {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(self).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        fs::write(&path, bytes.as_slice())?;
        Ok(())
    }
}

impl NetworkState {
    const PERSISTENCE_ID_FILE_NAME: &'static str = "persist_id";

    /// Compact the state to disk
    pub(in crate::state) fn compact<P>(
        from_path: P,
        to_path: P,
        gas_meter: &mut GasMeter,
    ) -> Result<(), VMError>
    where
        P: AsRef<Path>,
    {
        let source_store =
            StoreRef::new(HostStore::with_file(from_path.as_ref())?);
        let target_store =
            StoreRef::new(HostStore::with_file(to_path.as_ref())?);

        let source_persistence_id_file_path =
            from_path.as_ref().join(Self::PERSISTENCE_ID_FILE_NAME);
        let source_persistence_id =
            NetworkStateId::read(source_persistence_id_file_path)?;

        let mut network = NetworkState::with_target_store(
            source_store.clone(),
            target_store.clone(),
        )
        .restore_from_store(source_store.clone(), source_persistence_id)?;

        network.store_contract_states(gas_meter)?;
        let target_persistence_id = network.persist_to_store(target_store)?;

        let target_persistence_id_file_path =
            to_path.as_ref().join(Self::PERSISTENCE_ID_FILE_NAME);
        target_persistence_id.write(target_persistence_id_file_path)?;

        Ok(())
    }

    /// Persists the contracts stored on the [`NetworkState`]
    pub(in crate::state) fn persist_to_store(
        &self,
        store: StoreRef<OffsetLen>,
    ) -> Result<NetworkStateId, io::Error> {
        let head_stored = store.store(&self.head.0);
        let origin_stored = store.store(&self.origin.0);
        println!(
            "head_stored offslen={:?}",
            head_stored.ident().clone().erase()
        );
        println!(
            "origin_stored offslen={:?}",
            origin_stored.ident().clone().erase()
        );
        store.persist().map_err(|_| {
            io::Error::new(
                ErrorKind::Other,
                VMError::PersistenceError(String::from("network state")),
            )
        })?;
        Ok(NetworkStateId {
            head: *head_stored.ident().erase(),
            origin: *origin_stored.ident().erase(),
        })
    }

    /// Persists network state to disk
    pub(in crate::state) fn persist_to_disk<P>(
        &self,
        path: P,
    ) -> Result<(), VMError>
    where
        P: AsRef<Path>,
    {
        let persistence_id = self
            .persist_to_store(self.store.clone())
            .expect("Error in persistence");

        let file_path =
            path.as_ref().join(NetworkState::PERSISTENCE_ID_FILE_NAME);

        persistence_id.write(file_path)?;
        Ok(())
    }

    /// Given a [`NetworkStateId`] restores both [`Hamt`] which store
    /// contracts of the entire blockchain state.
    pub(in crate::state) fn restore_from_store(
        mut self,
        store: StoreRef<OffsetLen>,
        id: NetworkStateId,
    ) -> Result<Self, io::Error> {
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
    pub(in crate::state) fn restore_from_disk<P>(
        source_store_path: P,
    ) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let store =
            StoreRef::new(HostStore::with_file(source_store_path.as_ref())?);
        let file_path = source_store_path
            .as_ref()
            .join(Self::PERSISTENCE_ID_FILE_NAME);
        let persistence_id = NetworkStateId::read(file_path).map_err(|_| {
            io::Error::new(
                ErrorKind::Other,
                VMError::PersistenceError(String::from("network state")),
            )
        })?;
        NetworkState::new(store.clone())
            .restore_from_store(store, persistence_id)
    }

    /// Creates network state
    /// given source disk path.
    pub(in crate::state) fn create_from_disk<P>(
        source_store_path: P,
    ) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let store =
            StoreRef::new(HostStore::with_file(source_store_path.as_ref())?);
        Ok(NetworkState::new(store))
    }

    /// Store contracts' states
    pub(in crate::state) fn store_contract_states(
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
