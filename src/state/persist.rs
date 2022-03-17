// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::{StoreRef, OffsetLen, Ident};
use rkyv::{Archive, Deserialize, Serialize, archived_root, Infallible};
use rkyv::ser::{Serializer, serializers::AllocSerializer};
use std::fs;
use std::path::Path;
use rusk_uplink::StoreContext;
use std::io;
use dusk_hamt::Hamt;
use rusk_uplink::{ContractId};

use crate::state::{Contracts, NetworkState};
use crate::VMError;
use crate::contract::Contract;
use bytecheck::CheckBytes;

/// The bytes needed to encode a single `PersistId` on disk
const PERSIST_ID_SIZE: usize = 128;


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
        // let id: <NetworkStateId as Archive>::Archived = unsafe { *archived_root::<NetworkStateId>(buf.as_slice()) };
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
    /// Persists the origin contracts stored on the [`NetworkState`]
    pub fn persist(
        &self,
        store: StoreRef<OffsetLen>,
    ) -> Result<NetworkStateId, io::Error>
    {
        let head_stored = store.store(&self.head.0);
        let head_ident = head_stored.ident();
        let origin_stored = store.store(&self.origin.0);
        let origin_ident = origin_stored.ident();
        store.persist();
        Ok(NetworkStateId { head: *head_stored.ident().erase(), origin: *origin_stored.ident().erase() })
    }

    /// Given a [`NetworkStateId`] restores both [`Hamt`] which store
    /// contracts of the entire blockchain state.
    pub fn restore(mut self, store: StoreRef<OffsetLen>, id: NetworkStateId) -> Result<Self, io::Error> {

        let head_ident = Ident::<Hamt<ContractId, Contract, (), OffsetLen>, OffsetLen>::new(id.head);
        let origin_ident = Ident::<Hamt<ContractId, Contract, (), OffsetLen>, OffsetLen>::new(id.origin);

        let restored_head: &<Hamt<ContractId, Contract, (), OffsetLen> as Archive>::Archived =
            store.get::<Hamt<ContractId, Contract, (), OffsetLen>>(&head_ident);
        let restored_head: Hamt<ContractId, Contract, (), OffsetLen> =
            restored_head.deserialize(&mut store.clone()).unwrap();

        let restored_origin: &<Hamt<ContractId, Contract, (), OffsetLen> as Archive>::Archived =
            store.get::<Hamt<ContractId, Contract, (), OffsetLen>>(&origin_ident);
        let restored_origin: Hamt<ContractId, Contract, (), OffsetLen> =
            restored_origin.deserialize(&mut store.clone()).unwrap();

        self.origin = Contracts(restored_origin);

        self.head = Contracts(restored_head);

        self.staged = self.head.clone();

        Ok(self)
    }
}
