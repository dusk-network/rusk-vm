// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::module_config::ModuleConfig;
use crate::VMError;

pub use dusk_abi::{ContractId, ContractState};

/// A representation of a contract with a state and bytecode
#[derive(Clone)]
pub struct Contract {
    code: Vec<u8>,
    data: Vec<u8>,
}

impl Contract {
    /// Create a new Contract with initial state and code
    pub fn new<State, Code>(state: State, code: Code) -> Self
    where
        Code: Into<Vec<u8>>,
    {
        Contract {
            data: todo!(),
            code: code.into(),
        }
    }

    /// Returns a reference to the contract bytecode
    pub fn bytecode(&self) -> &[u8] {
        &self.code
    }

    /// Returns a reference to the contract state
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns a mutable reference to the contract state
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub(crate) fn instrument(mut self) -> Result<Self, VMError> {
        self.code = ModuleConfig::new()
            .with_grow_cost()
            .with_forbidden_floats()
            .with_metering()
            .with_table_size_limit()
            .apply(&self.code[..])?;

        Ok(self)
    }
}
