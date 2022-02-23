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
        let id = NetworkStateId::decode(&mut source)?;

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
    pub fn persist<B>(
        &self,
        ctor: &BackendCtor<B>,
    ) -> Result<NetworkStateId, PersistError>
    where
        B: 'static + Backend,
    {
        let head = Persistence::persist(ctor, &self.head.0)?;
        let origin = Persistence::persist(ctor, &self.origin.0)?;

        Ok(NetworkStateId { head, origin })
    }

    /// Given a [`NetworkStateId`] restores both [`Hamt`] which stores the
    /// contracts of the entire blockchain state.
    pub fn restore(mut self, id: NetworkStateId) -> Result<Self, PersistError> {
        let map = Map::from_generic(&id.origin.restore()?)?;
        self.origin = Contracts(map);

        let map = Map::from_generic(&id.head.restore()?)?;
        self.head = Contracts(map);

        self.staged = self.head.clone();

        Ok(self)
    }
}
