// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::ops::Deref;

use canonical::{Canon, CanonError};
use canonical_derive::Canon;
use microkelvin::{Child, ChildMut, Compound, GenericTree, Link, LinkCompound};

pub use dusk_abi::{ContractId, ContractState};

#[derive(Clone, Canon)]
pub struct ContractCode(Vec<u8>);

impl Deref for ContractCode {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ContractCode {
    /// Create a new Contract with initial state and code
    pub fn new<Code>(code: Code) -> Self
    where
        Code: Into<Vec<u8>>,
    {
        ContractCode(code.into())
    }
}

impl Compound<()> for ContractCode {
    type Leaf = Vec<u8>;

    fn child(&self, ofs: usize) -> Child<Self, ()> {
        if ofs == 0 {
            Child::Leaf(&self.0)
        } else {
            Child::EndOfNode
        }
    }

    fn child_mut(&mut self, ofs: usize) -> ChildMut<Self, ()> {
        if ofs == 0 {
            ChildMut::Leaf(&mut self.0)
        } else {
            ChildMut::EndOfNode
        }
    }

    fn from_generic(_tree: &GenericTree) -> Result<Self, CanonError> {
        todo!("deprecated");
    }
}

/// A representation of a contract with a state and bytecode
#[derive(Clone, Canon)]
pub struct Contract {
    state: ContractState,
    code: Link<ContractCode, ()>,
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
            code: Link::new(ContractCode::new(code)),
        }
    }

    /// Returns a reference to the contract bytecode
    pub fn bytecode(
        &self,
    ) -> Result<LinkCompound<ContractCode, ()>, CanonError> {
        self.code.inner()
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
