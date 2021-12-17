// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::{MaybeArchived, Store};
use rkyv::{Archive, Deserialize, Serialize};

pub use rusk_uplink::{ContractId, ContractState};
use rusk_uplink::{HostRawStore, RawStorage};

/// A representation of a contract with a state and bytecode
#[derive(Archive, Clone, Serialize, Deserialize)]
pub struct Contract {
    state: Vec<u8>,
    code: Vec<u8>,
}

pub trait ContractRef {
    fn bytecode(&self) -> &[u8];
    fn state(&self) -> &[u8];
}

impl<'a, T> ContractRef for MaybeArchived<'a, T>
where
    T: Archive + ContractRef,
    T::Archived: ContractRef,
{
    fn bytecode(&self) -> &[u8] {
        todo!()
    }

    fn state(&self) -> &[u8] {
        todo!()
    }
}

impl ContractRef for Contract {
    fn bytecode(&self) -> &[u8] {
        &self.code[..]
    }

    fn state(&self) -> &[u8] {
        &self.state[..]
    }
}

impl ContractRef for ArchivedContract {
    fn bytecode(&self) -> &[u8] {
        &self.code[..]
    }

    fn state(&self) -> &[u8] {
        &self.state[..]
    }
}

impl Contract {
    /// Create a new Contract with initial state and code
    pub fn new<State, Code>(state: State, code: Code) -> Self
    where
        Code: Into<Vec<u8>>,
        State: Archive + Serialize<RawStorage>,
    {
        let size = core::mem::size_of::<State::Archived>();
        let mut vec = Vec::with_capacity(size);
        vec.resize_with(size, || 0);
        let storage = HostRawStore::new(vec.as_mut_slice());
        storage.put(&state);
        Contract {
            state: vec,
            code: code.into(),
        }
    }

    /// Returns a mutable reference to the contract state
    pub fn state_mut(&mut self) -> &mut Vec<u8> {
        &mut self.state
    }
}
