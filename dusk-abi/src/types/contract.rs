// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

extern crate alloc;

use crate::canon_to_vec::CanonToVec;

use alloc::vec::Vec;
use canonical::{Canon, Store};
use canonical_derive::Canon;

/// Bytes representing a contract state
#[derive(Clone, Canon, Debug, Default)]
pub struct ContractState(Vec<u8>);

impl ContractState {
    /// Returns the state of the contract as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    /// Creates a state from a type implementing `Canon`
    pub fn from_canon<C, S>(c: &C, s: &S) -> Result<Self, S::Error>
    where
        C: Canon<S>,
        S: Store,
    {
        Ok(ContractState(c.encode_to_vec(s)?))
    }
}

/// Type used to identify a contract
#[derive(
    Default, Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Canon,
)]
pub struct ContractId([u8; 32]);

impl<B> From<B> for ContractId
where
    B: AsRef<[u8]>,
{
    fn from(b: B) -> Self {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(b.as_ref());
        ContractId(bytes)
    }
}

impl ContractId {
    /// Return a reserved contract id for host fn modules
    pub const fn reserved(id: u8) -> Self {
        let mut bytes = [0; 32];
        bytes[0] = id;
        ContractId(bytes)
    }

    /// Returns the contract id as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the contract id as a mutable slice
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
