// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::Canon;
use canonical_derive::Canon;

pub use dusk_abi::{ContractId, ContractState};

/// A representation of a contract with a state and bytecode
#[derive(Clone, Canon)]
pub struct Contract {
    state: ContractState,
    code: Vec<u8>,
}

impl Contract {
    /// Create a new Contract with initial state and code
    pub fn new<State, Code>(state: State, code: Code) -> Self
    where
        State: Canon,
        Code: Into<Vec<u8>>,
    {
        Contract {
            state: ContractState::from_canon(&state),
            code: code.into(),
        }
    }

    /// Returns a reference to the contract bytecode
    pub fn bytecode(&self) -> &[u8] {
        &self.code
    }

    /// Returns a reference to the contract state
    pub fn state(&self) -> &ContractState {
        &self.state
    }

    /// Returns a mutable reference to the contract state
    pub fn state_mut(&mut self) -> &mut ContractState {
        &mut self.state
    }
}
