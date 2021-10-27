// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use wasmer::{Memory, LazyInit};
use crate::VMError;


pub struct WasmerMemory {
    pub inner: LazyInit<Memory>,
}

impl WasmerMemory {
    pub fn init_env_memory(
        &mut self,
        exports: &wasmer::Exports,
    ) -> std::result::Result<(), VMError> {
        let memory = exports.get_memory("memory")?;
        self.inner.initialize(memory.clone());
        Ok(())
    }

    /// Read bytes from memory at the given offset
    pub fn read_memory_bytes(
        memory: &Memory,
        offset: u64,
    ) -> Result<&[u8], VMError> {
        let offset = offset as usize;
        Ok(unsafe { &memory.data_unchecked()[offset..] })
    }

    /// Read bytes from memory at the given offset and length
    pub fn read_memory_bytes_with_length(
        memory: &Memory,
        offset: u64,
        length: usize,
    ) -> Result<&[u8], VMError> {
        let offset = offset as usize;
        Ok(unsafe { &memory.data_unchecked()[offset..(offset+length)] })
    }

    /// Write bytes into memory at the given offset
    pub fn write_memory_bytes(
        &self,
        offset: u64,
        bytes: impl AsRef<[u8]>,
    ) -> Result<(), VMError> {
        let offset = offset as usize;
        let slice = bytes.as_ref();
        Ok(unsafe { self.inner.get_unchecked().data_unchecked_mut()[offset..(offset + slice.len())].copy_from_slice(slice) })
    }
}
