// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::MaybeArchived;
use rkyv::{Archive, Deserialize, Serialize};

pub use rusk_uplink::{ContractId, ContractState};

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
    {
        Contract {
            state: todo!(),
            code: code.into(),
        }
    }

    /// Returns a reference to the contract bytecode
    pub fn bytecode(&self) -> &[u8] {
        &self.code
    }

    /// Returns a reference to the contract state
    pub fn state(&self) -> &Vec<u8> {
        &self.state
    }

    /// Returns a mutable reference to the contract state
    pub fn state_mut(&mut self) -> &mut Vec<u8> {
        &mut self.state
    }


}
