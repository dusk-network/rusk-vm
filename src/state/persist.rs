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
use crate::VMError::PersistenceError;
use crate::{GasMeter, VMError};

/// The [`NetworkStateId`] is the persisted id of the [`NetworkState`]
#[derive(Archive, Serialize, Deserialize, Default, Clone, Debug)]
pub struct NetworkStateId {
    origin: OffsetLen,
    head: OffsetLen,
}

impl NetworkStateId {
    /// Read from the given path a [`NetworkStateId`]
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, VMError> {
        let buf = fs::read(&path)?;
        // let id: <NetworkStateId as Archive>::Archived = unsafe {
        // *archived_root::<NetworkStateId>(buf.as_slice()) };
        let id = unsafe { archived_root::<NetworkStateId>(buf.as_slice()) };
        let id: NetworkStateId = id.deserialize(&mut Infallible).unwrap();
        Ok(id)
    }

    /// Write to the given path a [`NetworkStateId`]
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), VMError> {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(self).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        fs::write(&path, bytes.as_slice())?;
        Ok(())
    }
}

impl NetworkState {
    const PERSISTENCE_ID_FILE_NAME: &'static str = "persist_id";

    /// Persists the origin contracts stored on the [`NetworkState`]
    pub fn persist(
        &self,
        store: StoreRef<OffsetLen>,
    ) -> Result<NetworkStateId, io::Error> {
        let head_stored = store.store(&self.head.0);
        let origin_stored = store.store(&self.origin.0);
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

    /// Persists the state to disk along with persistence id
    pub fn persist_to_disk<P: AsRef<Path>>(
        &self,
        store: StoreRef<OffsetLen>,
        store_path: P,
    ) -> Result<NetworkStateId, VMError> {
        let persistence_id = self.persist(store).expect("Error in persistence");

        let file_path = store_path.as_ref().join(Self::PERSISTENCE_ID_FILE_NAME);

        persistence_id.write(file_path)?;

        Ok(persistence_id)
    }

    /// Consolidate the state to disc,
    /// given the source disc path.
    pub fn consolidate_to_disk<P: AsRef<Path>>(
        source_store_path: P,
        target_store_path: P,
    ) -> Result<NetworkStateId, VMError> {
        let source_store =
            StoreRef::new(HostStore::with_file(source_store_path.as_ref())?);
        let target_store =
            StoreRef::new(HostStore::with_file(target_store_path.as_ref())?);

        let source_persistence_id_file_path =
            source_store_path.as_ref().join(Self::PERSISTENCE_ID_FILE_NAME);
        let source_persistence_id =
            NetworkStateId::read(source_persistence_id_file_path)?;

        let mut network = NetworkState::with_target_store(
            source_store.clone(),
            target_store.clone(),
        )
        .restore(source_store.clone(), source_persistence_id)?;

        let mut gas = GasMeter::with_limit(100_000_000_000);
        network.store_contract_states(&mut gas)?;
        let target_persistence_id = network.persist(target_store.clone())?;

        let target_persistence_id_file_path =
            target_store_path.as_ref().join(Self::PERSISTENCE_ID_FILE_NAME);
        target_persistence_id.write(target_persistence_id_file_path)?;

        Ok(target_persistence_id)
    }

    /// Given a [`NetworkStateId`] restores both [`Hamt`] which store
    /// contracts of the entire blockchain state.
    pub fn restore(
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
