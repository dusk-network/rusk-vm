// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::VMError;
use wasmer::{LazyInit, Memory};

pub struct WasmerMemory {
    pub inner: LazyInit<Memory>,
}

impl WasmerMemory {
    pub fn new() -> WasmerMemory {
        WasmerMemory {
            inner: LazyInit::new(),
        }
    }
    /// Initializes the object with exported memory
    pub fn init(
        &mut self,
        exports: &wasmer::Exports,
    ) -> std::result::Result<(), VMError> {
        let memory = exports.get_memory("memory")?;
        self.inner.initialize(memory.clone());
        Ok(())
    }

    /// Read bytes from memory at a given offset and length
    pub fn read(&self, offset: u64, length: usize) -> Result<&[u8], VMError> {
        let offset = offset as usize;
        Ok(unsafe {
            &self.inner.get_unchecked().data_unchecked()
                [offset..(offset + length)]
        })
    }

    /// Write bytes into memory at a given offset
    pub fn write(
        &self,
        offset: u64,
        bytes: impl AsRef<[u8]>,
    ) -> Result<(), VMError> {
        let offset = offset as usize;
        let slice = bytes.as_ref();
        unsafe {
            self.inner.get_unchecked().data_unchecked_mut()
                [offset..(offset + slice.len())]
                .copy_from_slice(slice);
        }
        Ok(())
    }

    /// Write bytes into memory at a given offset
    pub fn with_mut_slice_from<F, R>(&mut self, offset: usize, closure: F) -> R
    where
        F: Fn(&mut [u8]) -> R,
    {
        unsafe {
            let slice =
                &mut self.inner.get_unchecked().data_unchecked_mut()[offset..];
            closure(slice)
        }
    }

    /// Get a reference into memory
    pub fn with_slice_from<F, R>(&self, ofs: usize, mut closure: F) -> R
    where
        F: FnMut(&[u8]) -> R,
    {
        unsafe {
            let slice = self.inner.get_unchecked().data_unchecked();
            closure(&slice[ofs..])
        }
    }
}
