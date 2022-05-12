// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use microkelvin::{MaybeArchived, OffsetLen, StoreSerializer};
use rkyv::{ser::Serializer, Archive, Deserialize, Serialize};

use rusk_uplink::StoreContext;
pub use rusk_uplink::{ContractId, ContractState};

/// A representation of a contract with a state and bytecode
#[derive(Archive, Clone, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Contract {
    // put the below behind link
    state: Vec<u8>,
    // put the below behind link
    code: Vec<u8>,
}

/// Trait for wrapping archived/memory Contracts
pub trait ContractRef {
    /// Get the bytecode of the contract
    fn bytecode(&self) -> &[u8];
    /// Get the state of the contract
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
    pub fn new<State, Code>(
        state: &State,
        code: Code,
        store: &StoreContext,
    ) -> Self
    where
        State: Serialize<StoreSerializer<OffsetLen>>,
        Code: Into<Vec<u8>>,
    {
        let mut ser = store.serializer();
        ser.serialize_value(state).unwrap();

        let state_vec = ser.spill_bytes(|bytes| Vec::from(bytes));

        Contract {
            state: state_vec,
            code: code.into(),
        }
    }

    /// Upgrade the contract state
    pub fn set_state(&mut self, state: &[u8]) {
        self.state.truncate(0);
        self.state.extend_from_slice(state);
    }
}
