// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

extern crate alloc;

use crate::canon_to_vec::CanonToVec;

use alloc::vec::Vec;
use canonical::{ByteSource, Canon, Store};
use canonical_derive::Canon;

/// A generic query
#[derive(Clone, Canon, Debug, Default)]
pub struct Query(Vec<u8>);

impl Query {
    /// Returns the byte representation of the query
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    /// Creates a query from a raw bytes
    pub fn from_slice(buffer: &[u8]) -> Self {
        Query(buffer.to_vec())
    }

    /// Creates a query from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: &S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(Query(c.encode_to_vec(s)?))
    }

    /// Casts the generict query to given type
    pub fn cast<C, S>(&self, store: S) -> Result<C, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        let mut source = ByteSource::new(self.as_bytes(), &store);
        Canon::<S>::read(&mut source)
    }
}
