// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Sink, Source};
use canonical_derive::Canon;
use dusk_hamt::Map;
use microkelvin::{
    Backend, BackendCtor, Compound, PersistError, PersistedId, Persistence,
};
use std::fs;
use std::path::Path;

use crate::state::{Contracts, NetworkState};
use crate::VMError;

/// The bytes needed to encode a single `PersistId` on disk
const PERSIST_ID_SIZE: usize = 36;

/// The [`NetworkStateId`] is the persisted id of the [`NetworkState`]
#[derive(Canon, Clone, Copy, Debug)]
pub struct NetworkStateId {
    origin: PersistedId,
    head: PersistedId,
}

impl NetworkStateId {
    /// Read from the given path a [`NetworkStateId`]
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, VMError> {
        let buf = fs::read(&path)?;
        let mut source = Source::new(&buf[..]);
        let id = NetworkStateId::decode(&mut source)
            .map_err(VMError::from_store_error)?;

        Ok(id)
    }

    /// Write to the given path a [`NetworkStateId`]
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), VMError> {
        // We need to store two ids, the origin and the head, so we allocate
        // enough buffer for both
        let mut buf = [0u8; PERSIST_ID_SIZE * 2];

        let mut sink = Sink::new(&mut buf);
        self.origin.encode(&mut sink);
        self.head.encode(&mut sink);

        fs::write(&path, &buf)?;
        Ok(())
    }
}

impl NetworkState {
    /// Persists the origin contracts stored on the [`NetworkState`] specifying
    /// a backend ctor function.
    pub async fn persist<B>(
        &self,
        ctor: &BackendCtor<B>,
    ) -> Result<NetworkStateId, PersistError>
    where
        B: 'static + Backend,
    {
        let guard = self.0.read().await;

        let head = Persistence::persist(ctor, &guard.head.0)?;
        let origin = Persistence::persist(ctor, &guard.origin.0)?;

        Ok(NetworkStateId { head, origin })
    }

    /// Given a [`NetworkStateId`] restores both [`Hamt`] which stores the
    /// contracts of the entire blockchain state.
    pub async fn restore(
        self,
        id: NetworkStateId,
    ) -> Result<Self, PersistError> {
        let mut guard = self.0.write().await;

        let map = Map::from_generic(&id.origin.restore()?)?;
        guard.origin = Contracts(map);

        let map = Map::from_generic(&id.head.restore()?)?;
        guard.head = Contracts(map);

        drop(guard);
        Ok(self)
    }
}
