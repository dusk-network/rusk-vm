// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::modules::ModuleConfig;
use crate::VMError;
use loupe::MemoryUsage;
use std::ptr::NonNull;
use std::sync::Arc;
use wasmer::Module;
use wasmer::{
    vm::{
        self, MemoryError, MemoryStyle, TableStyle, VMMemoryDefinition,
        VMTableDefinition,
    },
    MemoryType, Pages, TableType, Tunables,
};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

/// A custom tunables that allows you to set a memory and table size limits.
#[derive(MemoryUsage)]
pub struct LimitingTunables<T: Tunables> {
    /// The maximum a linear memory is allowed to be (in Wasm pages, 64 KiB
    /// each). Since Wasmer ensures there is only none or one memory, this
    /// is practically an upper limit for the guest memory.
    memory_limit: Pages,
    /// The maximum table size
    table_size_limit: u32,
    /// The base implementation we delegate all the logic to
    base: T,
}

impl<T: Tunables> LimitingTunables<T> {
    pub fn new(base: T, memory_limit: Pages, table_size_limit: u32) -> Self {
        Self {
            memory_limit,
            table_size_limit,
            base,
        }
    }
}

impl<T: Tunables> Tunables for LimitingTunables<T> {
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        // let adjusted = self.adjust_memory(memory);
        self.base.memory_style(memory)
    }

    fn table_style(&self, table: &TableType) -> TableStyle {
        self.base.table_style(table)
    }

    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let memory_type = MemoryType {
            maximum: Some(self.memory_limit),
            ..*ty
        };
        self.base.create_host_memory(&memory_type, style)
    }

    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let memory_type = MemoryType {
            maximum: Some(self.memory_limit),
            ..*ty
        };
        self.base
            .create_vm_memory(&memory_type, style, vm_definition_location)
    }

    fn create_host_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
    ) -> Result<Arc<dyn vm::Table>, String> {
        let table_type = TableType {
            maximum: Some(self.table_size_limit),
            ..*ty
        };
        self.base.create_host_table(&table_type, style)
    }

    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<Arc<dyn vm::Table>, String> {
        let table_type = TableType {
            maximum: Some(self.table_size_limit),
            ..*ty
        };
        self.base
            .create_vm_table(&table_type, style, vm_definition_location)
    }
}

pub struct WasmerCompiler;

impl WasmerCompiler {
    /// Creates module out of bytecode
    pub fn create_module(
        bytecode: impl AsRef<[u8]>,
        _module_config: &ModuleConfig,
    ) -> Result<Module, VMError> {
        let store =
            wasmer::Store::new(&Universal::new(Cranelift::default()).engine());
        Module::new(&store, bytecode).map_err(|e| e.into())
    }
}
