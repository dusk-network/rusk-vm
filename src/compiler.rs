// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use wasmer::{CompileError, Module};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

pub struct WasmerCompiler;

impl WasmerCompiler {
    /// Creates module out of bytecode
    pub fn create_module(
        bytecode: impl AsRef<[u8]>,
    ) -> Result<Module, CompileError> {
        let store =
            wasmer::Store::new(&Universal::new(Cranelift::default()).engine());
        Module::new(&store, bytecode)
    }
}
