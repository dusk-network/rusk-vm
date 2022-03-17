// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::{StoreRef, OffsetLen};
use rkyv::{Archive, Deserialize, Serialize, archived_root, Infallible};
use rkyv::ser::{Serializer, serializers::AllocSerializer};
use std::fs;
use std::path::Path;
use rusk_uplink::StoreContext;

use crate::state::{Contracts, NetworkState};
use crate::VMError;
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
