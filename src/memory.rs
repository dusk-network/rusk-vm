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
    /// Check that the given offset and length fits into the memory bounds. If not,
    /// it will try to grow the memory.
    fn check_bounds(memory: &Memory, offset: u64, len: usize) -> Result<(), VMError> {
        if memory.data_size() < offset + len as u64 {
            let cur_pages = memory.size().0;
            let capacity = cur_pages as usize * wasmer::WASM_PAGE_SIZE;
            let missing = offset as usize + len - capacity;
            // Ceiling division
            let req_pages = ((missing + wasmer::WASM_PAGE_SIZE - 1)
                / wasmer::WASM_PAGE_SIZE) as u32;
            memory.grow(req_pages).map_err(|_|VMError::MemoryNotFound)?; // todo this grow will probably need to go away
        }
        Ok(())
    }

    /// Read bytes from memory at the given offset and length
    pub fn read_memory_bytes(
        memory: &Memory,
        offset: u64,
        len: usize,
    ) -> Result<Vec<u8>, VMError> {
        Self::check_bounds(memory, offset, len)?;
        let offset = offset as usize;
        let vec: Vec<_> = memory.view()[offset..(offset + len)]
            .iter()
            .map(|cell| cell.get())
            .collect();
        Ok(vec)
    }

    /// Write bytes into memory at the given offset
    pub fn write_memory_bytes(
        memory: &Memory,
        offset: u64,
        bytes: impl AsRef<[u8]>,
    ) -> Result<(), VMError> {
        let slice = bytes.as_ref();
        let len = slice.len();
        Self::check_bounds(memory, offset, len as _)?;
        let offset = offset as usize;
        memory.view()[offset..(offset + len)]
            .iter()
            .zip(slice.iter())
            .for_each(|(cell, v)| cell.set(*v));
        Ok(())
    }
}

// impl WasmerMemory {
//     pub fn new(memory_ref: &wasmer::Memory) -> WasmerMemoryRef {
//         WasmerMemoryRef { memory_ref }
//     }
//     pub fn with_direct_access<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
//         let buf = unsafe { self.memory_ref.data_unchecked() };
//         f(buf)
//     }
//
//     pub fn with_direct_access_mut<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> R {
//         let buf = unsafe { self.memory_ref.data_unchecked_mut() };
//         f(buf)
//     }
//
// }
