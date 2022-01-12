// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::convert::Infallible;
use core::hint::unreachable_unchecked;
use rkyv::{ser::Serializer, Archive, Fallible, Serialize};

use parking_lot::RwLock;

use std::sync::Arc;

use microkelvin::{Ident, Offset, Storage, Store, Stored};
use std::ffi::c_void;

pub trait UnwrapInfallible<T> {
    fn unwrap_infallible(self) -> T;
}

impl<T> UnwrapInfallible<T> for Result<T, core::convert::Infallible> {
    fn unwrap_infallible(self) -> T {
        match self {
            Ok(t) => t,
            Err(_) => unsafe {
                // safe, since the error type cannot be instantiated
                unreachable_unchecked()
            },
        }
    }
}

/// Storage that uses raw external memory store data
#[derive(Debug)]
pub struct RawStorage {
    bytes: *mut c_void,
    length: usize,
    written: usize,
}

impl Fallible for RawStorage {
    type Error = Infallible;
}

impl RawStorage {
    /// Creates a new empty `PageStorage`
    pub fn new(bytes: &mut [u8]) -> RawStorage {
        RawStorage {
            bytes: bytes as *mut _ as *mut c_void,
            length: bytes.len(),
            written: 0,
        }
    }

    fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.bytes as *mut u8, self.length)
        }
    }

    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.bytes as *mut u8, self.length)
        }
    }
}

impl Serializer for RawStorage {
    fn pos(&self) -> usize {
        self.written
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        let space_left = self.length - self.written;
        if space_left < bytes.len() {
            unreachable!() // todo don't want to change Storage trait atm as
                           // Storage is making it Infallible
        } else {
            self.as_slice_mut().copy_from_slice(bytes);
            self.written += bytes.len();
            Ok(())
        }
    }
}

impl<'a> Storage<Offset> for RawStorage {
    fn put<T: Serialize<RawStorage>>(&mut self, t: &T) -> Offset {
        self.serialize_value(t).unwrap_infallible();
        Offset::new(self.pos() as u64)
    }

    fn get<T: Archive>(&self, ofs: &Offset) -> &T::Archived {
        let ofs = ofs.0;
        let size = core::mem::size_of::<T::Archived>();
        assert!(ofs <= self.length as u64);
        let start_pos = (ofs as usize)
            .checked_sub(size)
            .expect("Offset larger than size");
        let slice = &self.as_slice()[start_pos..][..size];
        unsafe { rkyv::archived_root::<T>(slice) }
    }
}

/// Store that utilises a reference-counted PageStorage
#[derive(Clone)]
pub struct HostRawStore {
    /// Inner storage representation
    pub inner: Arc<RwLock<RawStorage>>,
}

impl HostRawStore {
    /// Creates a new HostStore
    pub fn new(bytes: &mut [u8]) -> Self {
        HostRawStore {
            inner: Arc::new(RwLock::new(RawStorage::new(bytes))),
        }
    }
}

impl Fallible for HostRawStore {
    type Error = Infallible;
}

impl Store for HostRawStore {
    type Identifier = Offset;
    type Storage = RawStorage;

    fn put<T>(&self, t: &T) -> Stored<T, Self>
    where
        T: Serialize<Self::Storage>,
    {
        Stored::new(self.clone(), Ident::new(self.inner.write().put::<T>(t)))
    }

    fn get_raw<T>(&self, id: &Ident<Self::Identifier, T>) -> &T::Archived
    where
        T: Archive,
    {
        let guard = self.inner.read();
        let reference = guard.get::<T>(&id.erase());
        let extended: &T::Archived = unsafe { core::mem::transmute(reference) };
        extended
    }
}
