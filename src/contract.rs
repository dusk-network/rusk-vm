// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use microkelvin::{
    All, Branch, Child, ChildMut, Compound, Link, MaybeArchived, MaybeStored,
    Nth, OffsetLen, StoreSerializer,
};
use rkyv::{ser::Serializer, Archive, Deserialize, Serialize};

use crate::linked_list::LinkedList;
use rusk_uplink::StoreContext;
pub use rusk_uplink::{ContractId, ContractState};

#[derive(Archive, Clone, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct ContractData(Vec<u8>);

impl AsRef<[u8]> for ContractData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for ArchivedContractData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<A, I> Compound<A, I> for ContractData {
    type Leaf = Vec<u8>;

    fn child(&self, ofs: usize) -> Child<Self, A, I> {
        match ofs {
            0 => Child::Leaf(&self.0),
            _ => Child::End,
        }
    }

    fn child_mut(&mut self, ofs: usize) -> ChildMut<Self, A, I> {
        match ofs {
            0 => ChildMut::Leaf(&mut self.0),
            _ => ChildMut::End,
        }
    }
}

/// A representation of a contract with a state and bytecode
#[derive(Archive, Clone, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Contract {
    state: Link<ContractData, (), OffsetLen>,
    code: Link<LinkedList<ContractData, (), OffsetLen>, (), OffsetLen>,
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

        let state = Link::new(ContractData(state_vec));
        let mut code_list = LinkedList::<ContractData, (), OffsetLen>::new();
        code_list.push(ContractData(code.into()));

        Contract {
            state,
            code: Link::new(code_list),
        }
    }

    /// Update the contract's state
    pub fn set_state(&mut self, state: &[u8]) {
        let s = self.state.inner_mut();
        s.0.truncate(0);
        s.0.extend_from_slice(state);
    }

    /// Returns a slice to the contract's bytecode
    pub fn bytecode(&self) -> Vec<u8> {
        let v = match self.code.inner() {
            MaybeStored::Memory(m) => {
                let vector = m.walk(All).expect("Some(Branch)");
                let bytes: &[u8] = match vector.leaf() {
                    MaybeArchived::Memory(v) => v.as_ref(),
                    MaybeArchived::Archived(v) => v.as_ref(),
                };
                bytes.to_vec()
            }
            MaybeStored::Stored(s) => {
                Vec::new()
            },
        };
        v
    }

    /// Returns a slice to the contract's state
    pub fn state(&self) -> &[u8] {
        match self.state.inner() {
            MaybeStored::Memory(m) => m.as_ref(),
            MaybeStored::Stored(s) => s.inner().as_ref(),
        }
    }
}

impl ArchivedContract {
    /// Returns the identity of the contract's bytecode in the store
    pub fn bytecode(&self, store: &StoreContext) -> Vec<u8> {
        let all = Branch::walk_with_store(
            MaybeArchived::<LinkedList<ContractData, (), OffsetLen>>::Archived(
                &store.get(self.code.ident()),
            ),
            All,
            store.clone(),
        )
        .expect("invalid branch");
        let bytes = match all.leaf() {
            MaybeArchived::Memory(v) => v.as_ref(),
            MaybeArchived::Archived(v) => v.as_ref(),
        };
        if bytes.len() > 65536 {
            println!("xarchived xcontract, {} bytes", bytes.len());
        }
        bytes.to_vec()
    }

    /// Returns the identity of the contract's state in the store
    pub fn state<'a>(&self, store: &'a StoreContext) -> &'a [u8] {
        &store.get(self.state.ident()).0
    }
}
