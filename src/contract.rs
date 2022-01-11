// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::convert::Infallible;

use bytecheck::CheckBytes;
use microkelvin::{
    HostStore, Ident, MaybeArchived, OffsetLen, Store, Stored, UnwrapInfallible,
};
use rkyv::{
    ser::serializers::AllocSerializer, ser::Serializer, AlignedVec, Archive,
    Deserialize, Fallible, Serialize,
};

pub use rusk_uplink::{ContractId, ContractState};

/// A representation of a contract with a state and bytecode
#[derive(Archive, Clone, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
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
        match self {
            MaybeArchived::Memory(m) => m.bytecode(),
            MaybeArchived::Archived(a) => a.bytecode(),
        }
    }

    fn state(&self) -> &[u8] {
        match self {
            MaybeArchived::Memory(m) => m.state(),
            MaybeArchived::Archived(a) => a.state(),
        }
    }
}

impl ContractRef for Contract {
    fn bytecode(&self) -> &[u8] {
        &self.code[..]
    }

    fn state(&self) -> &[u8] {
        &self.state
    }
}

impl ContractRef for ArchivedContract {
    fn bytecode(&self) -> &[u8] {
        &self.code[..]
    }

    fn state(&self) -> &[u8] {
        &self.state
    }
}

impl Contract {
    /// Create a new Contract with initial state and code
    pub fn new<State, Code>(state: State, code: Code) -> Self
    where
        State: Into<Vec<u8>>,
        Code: Into<Vec<u8>>,
    {
        Contract {
            state: state.into(),
            code: code.into(),
        }
    }

    /// Upgrade the contract state
    pub fn set_state(&mut self, state: Vec<u8>) {
        self.state = state
    }
}
