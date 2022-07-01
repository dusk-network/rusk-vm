// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::ser::{serializers::AllocSerializer, Serializer};
use std::fs;
use std::io;
use thiserror::Error;

use crate::error::VMError;
use crate::state::NetworkState;

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

impl NetworkState {
    /// Persists the contracts in the [`NetworkState`].
    pub fn persist(&self) -> Result<(), VMError> {
        let store = &self.store;
        let contracts_stored = store.store(&self.contracts.0);
        store.persist().map_err(|_| {
            PersistError::Store(String::from(
                "Store persistence failed for network state",
            ))
        })?;

        if let Some(id_path) = &self.id_path {
            let persistence_id = *contracts_stored.ident().erase();

            let mut serializer = AllocSerializer::<0>::default();
            serializer.serialize_value(&persistence_id).unwrap();
            let bytes = serializer.into_serializer().into_inner();

            fs::write(id_path, bytes).map_err(PersistError::Io)?;
        }

        Ok(())
    }
}
