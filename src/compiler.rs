// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::compiler_config::CompilerConfigProvider;
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
    BaseTunables, MemoryType, Pages, TableType, Target, Tunables,
};
use wasmer_engine_universal::Universal;

/// A custom tunables that allows you to set a memory limit.
///
/// After adjusting the memory limits, it delegates all other logic
/// to the base tunables.
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

    /// Takes an input memory type as requested by the guest and sets
    /// a maximum if missing. The resulting memory type is final if
    /// valid. However, this can produce invalid types, such that
    /// validate_memory must be called before creating the memory.
    fn adjust_memory(&self, requested: &MemoryType) -> MemoryType {
        let mut adjusted = *requested;
        if requested.maximum.is_none() {
            adjusted.maximum = Some(self.memory_limit);
        }
        adjusted
    }

    /// Ensures the a given memory type does not exceed the memory limit.
    /// Call this after adjusting the memory.
    fn validate_memory(&self, ty: &MemoryType) -> Result<(), MemoryError> {
        if ty.minimum > self.memory_limit {
            return Err(MemoryError::Generic(
                "Minimum exceeds the allowed memory limit".to_string(),
            ));
        }

        if let Some(max) = ty.maximum {
            if max > self.memory_limit {
                return Err(MemoryError::Generic(
                    "Maximum exceeds the allowed memory limit".to_string(),
                ));
            }
        } else {
            return Err(MemoryError::Generic("Maximum unset".to_string()));
        }

        Ok(())
    }
}

impl<T: Tunables> Tunables for LimitingTunables<T> {
    /// Construct a `MemoryStyle` for the provided `MemoryType`
    ///
    /// Delegated to base.
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        let adjusted = self.adjust_memory(memory);
        self.base.memory_style(&adjusted)
    }

    /// Construct a `TableStyle` for the provided `TableType`
    ///
    /// Delegated to base.
    fn table_style(&self, table: &TableType) -> TableStyle {
        self.base.table_style(table)
    }

    /// Create a memory owned by the host given a [`MemoryType`] and a
    /// [`MemoryStyle`].
    ///
    /// The requested memory type is validated, adjusted to the limited and then
    /// passed to base.
    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let adjusted = self.adjust_memory(ty);
        self.validate_memory(&adjusted)?;
        self.base.create_host_memory(&adjusted, style)
    }

    /// Create a memory owned by the VM given a [`MemoryType`] and a
    /// [`MemoryStyle`].
    ///
    /// Delegated to base.
    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let adjusted = self.adjust_memory(ty);
        self.validate_memory(&adjusted)?;
        self.base
            .create_vm_memory(&adjusted, style, vm_definition_location)
    }

    /// Create a table owned by the host given a [`TableType`] and a
    /// [`TableStyle`].
    ///
    /// Delegated to base.
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

    /// Create a table owned by the VM given a [`TableType`] and a
    /// [`TableStyle`].
    ///
    /// Delegated to base.
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
        module_config: &ModuleConfig,
    ) -> Result<Module, VMError> {
        let compiler_config =
            CompilerConfigProvider::from_config(module_config)?;
        let base = BaseTunables::for_target(&Target::default());
        let tunables = LimitingTunables::new(
            base,
            Pages(module_config.max_memory_pages),
            module_config.max_table_size,
        );
        let store = wasmer::Store::new_with_tunables(
            &Universal::new(compiler_config).engine(),
            tunables,
        );
        Module::new(&store, bytecode).map_err(VMError::WasmerCompileError)
    }
}
