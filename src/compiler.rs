// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::compiler_config::CompilerConfigProvider;
use crate::modules::ModuleConfig;
use crate::VMError;
use wasmer::Module;
use wasmer_engine_universal::Universal;

pub struct WasmerCompiler;

impl WasmerCompiler {
    /// Creates module out of bytecode
    pub fn create_module(
        bytecode: impl AsRef<[u8]>,
        module_config: &ModuleConfig,
    ) -> Result<Module, VMError> {
        let compiler_config =
            CompilerConfigProvider::from_config(module_config)?;
        let store =
            wasmer::Store::new(&Universal::new(compiler_config).engine());
        Module::new(&store, bytecode).map_err(VMError::WasmerCompileError)
    }
}
