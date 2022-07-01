// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::config::{Config, DEFAULT_CONFIG};
use crate::contract::Contract;
use crate::modules::{HostModule, HostModules};
use crate::state::{Contracts, NetworkState};

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dusk_hamt::Hamt;
use microkelvin::{HostStore, Ident, OffsetLen};
use rkyv::{archived_root, Archive, Deserialize, Infallible};
use rusk_uplink::{ContractId, StoreContext};

/// Builder for a [`NetworkState`].
pub struct NetworkStateBuilder {
    store_and_contracts: Option<(StoreContext, Contracts)>,
    modules: HostModules,
    id_path: Option<PathBuf>,
    config: &'static Config,
}

impl NetworkStateBuilder {
    const PERSISTENCE_ID_FILE_NAME: &'static str = "persist_id";

    /// Create a new [`NetworkState`] builder.
    pub fn new() -> Self {
        NetworkStateBuilder::default()
    }

    /// Set the configuration for the network state.
    pub fn config(self, config: &'static Config) -> Self {
        Self {
            store_and_contracts: self.store_and_contracts,
            modules: self.modules,
            id_path: self.id_path,
            config,
        }
    }

    /// Set the directory to store the state. If not set,
    pub fn store_dir<P: AsRef<Path>>(self, dir: P) -> io::Result<Self> {
        let dir = dir.as_ref();

        let id_path = dir.join(Self::PERSISTENCE_ID_FILE_NAME);

        let store = StoreContext::new(HostStore::with_file(dir)?);
        let contracts = match id_path.exists() && id_path.is_file() {
            true => {
                let buf = fs::read(&id_path)?;
                let persist_id =
                    unsafe { archived_root::<OffsetLen>(buf.as_slice()) };
                let persist_id =
                    persist_id.deserialize(&mut Infallible).unwrap();

                let contracts_ident = Ident::<
                    Hamt<ContractId, Contract, (), OffsetLen>,
                    OffsetLen,
                >::new(persist_id);

                let contracts: &<Hamt<ContractId, Contract, (), OffsetLen> as Archive>::Archived =
                        store.get::<Hamt<ContractId, Contract, (), OffsetLen>>(&contracts_ident);

                Contracts(contracts.deserialize(&mut store.clone()).unwrap())
            }
            false => Contracts::default(),
        };

        Ok(Self {
            store_and_contracts: Some((store, contracts)),
            modules: self.modules,
            id_path: Some(id_path),
            config: self.config,
        })
    }

    /// Use the given host module.
    pub fn module<M>(self, module: M) -> Self
    where
        M: 'static + HostModule,
    {
        let mut modules = self.modules;
        modules.insert(module);

        Self {
            store_and_contracts: self.store_and_contracts,
            modules,
            id_path: self.id_path,
            config: self.config,
        }
    }

    /// Build the [`NetworkState`].
    pub fn build(self) -> NetworkState {
        let (store, contracts) =
            self.store_and_contracts.unwrap_or_else(|| {
                let store = StoreContext::new(HostStore::new());
                let contracts = Contracts::default();
                (store, contracts)
            });

        NetworkState {
            contracts,
            modules: self.modules,
            store,
            id_path: self.id_path,
            config: self.config,
        }
    }
}

impl Default for NetworkStateBuilder {
    fn default() -> Self {
        Self {
            store_and_contracts: None,
            modules: HostModules::default(),
            id_path: None,
            config: &DEFAULT_CONFIG,
        }
    }
}
