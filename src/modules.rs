// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::WasmerCompiler;
use crate::config::{config_hash, Config};
use crate::state::hash::hash;
use crate::VMError;

use cached::cached_key_result;
use cached::TimedSizedCache;
use thiserror::Error;
use tracing::trace;
use wasmer::Module;

pub use rusk_uplink::{ContractId, ContractState};

pub trait HostModule {
    fn execute(&self);

    fn module_id(&self) -> ContractId;
}

type BoxedHostModule = Box<dyn HostModule>;

/// Compiles a module with the specified bytecode or retrieves it from
pub fn compile_module(
    bytecode: &[u8],
    config: &'static Config,
) -> Result<Module, VMError> {
    get_or_create_module(bytecode, config)
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct ModuleCacheKey {
    hash: [u8; 32],
    config_hash: u64,
}

// The `cached` crate is used to generate a cache for calls to this function.
// This is done to prevent modules from being compiled over and over again,
// saving some CPU cycles.
cached_key_result! {
    COMPUTE: TimedSizedCache<ModuleCacheKey, Module>
        = TimedSizedCache::with_size_and_lifespan(2048, 86400);
    Key = {
        ModuleCacheKey{ hash: hash(bytecode), config_hash: config_hash(config) }
    };

    fn get_or_create_module(bytecode: &[u8], config: &'static Config) -> Result<Module, VMError> = {
        trace!("Compiling module");
        WasmerCompiler::create_module(bytecode, config)
    }
}

/// A cheaply cloneable store for host modules.
#[derive(Clone, Default)]
pub struct HostModules(Rc<RefCell<HashMap<ContractId, BoxedHostModule>>>);

/// A `Ref` to a particular host module.
pub struct HostModuleRef<'a> {
    map_ref: Ref<'a, HashMap<ContractId, BoxedHostModule>>,
    id: &'a ContractId,
}

impl<'a> HostModuleRef<'a> {
    pub fn get(&self) -> Option<&BoxedHostModule> {
        self.map_ref.get(self.id)
    }
}

impl HostModules {
    /// Insert a new module into the store.
    pub fn insert<M>(&mut self, module: M)
    where
        M: 'static + HostModule,
    {
        self.0
            .borrow_mut()
            .insert(module.module_id(), Box::new(module));
    }

    /// Get a reference to a particular module from the store.
    pub fn get_module_ref<'a>(
        &'a self,
        id: &'a ContractId,
    ) -> HostModuleRef<'a> {
        HostModuleRef {
            map_ref: self.0.borrow(),
            id,
        }
    }
}

#[derive(Error, Debug)]
pub enum InstrumentationError {
    #[error("gas metering injection")]
    GasMeteringInjection,
    #[error("stack height injection")]
    StackHeightInjection,
    #[error("multiple tables")]
    MultipleTables,
    #[error("max table size")]
    MaxTableSize,
    #[error("invalid bytecode")]
    InvalidByteCode,
    #[error("invalid instruction type")]
    InvalidInstructionType,
}
