// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::config::Config;
use crate::contract::Contract;
use crate::error::VMError;
use crate::modules::compile_module;
use crate::state::hash::{hash, Hasher};

use rusk_uplink::ContractId;

use bytecheck::CheckBytes;
use dusk_hamt::{Hamt, KvPair, Lookup};
use microkelvin::{
    Annotation, BranchRef, BranchRefMut, Combine, Keyed, OffsetLen,
};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Archive,
    Serialize,
    Deserialize,
    CheckBytes,
)]
#[archive(as = "Self")]
pub struct HashAnnotation([u8; 32]);

impl Combine<HashAnnotation> for HashAnnotation {
    /// Sum each byte of the hashes together to produce a new number. This
    /// operation is commutative - meaning different tree structures will
    /// produce the same result.
    fn combine(&mut self, with: &HashAnnotation) {
        self.0.iter_mut().zip(with.0).for_each(|(b, ob)| {
            *b = b.wrapping_add(ob);
        });
    }
}

impl Annotation<KvPair<ContractId, Contract>> for HashAnnotation {
    fn from_leaf(leaf: &KvPair<ContractId, Contract>) -> Self {
        let mut hasher = Hasher::new();
        hasher
            .update(leaf.key().as_bytes())
            .update(leaf.value().state());
        Self(hasher.finalize())
    }
}

/// State of the contracts on the network.
#[derive(Archive, Default, Clone)]
pub struct Contracts(
    pub(crate) Hamt<ContractId, Contract, HashAnnotation, OffsetLen>,
);

impl Contracts {
    /// Root hash
    pub fn root(&self) -> [u8; 32] {
        HashAnnotation::from_node(&self.0).0
    }

    /// Returns a reference to the specified contracts state.
    pub fn get_contract(
        &self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRef<Contract>, VMError> {
        self.0
            .get(contract_id)
            .ok_or(VMError::UnknownContract(*contract_id))
    }

    /// Returns a mutable reference to the specified contracts state.
    pub fn get_contract_mut(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<impl BranchRefMut<Contract>, VMError> {
        self.0
            .get_mut(contract_id)
            .ok_or(VMError::UnknownContract(*contract_id))
    }

    /// Deploys a contract to the state, returning the address of the created
    /// contract or an error.
    pub fn deploy(
        &mut self,
        contract: Contract,
        config: &'static Config,
    ) -> Result<ContractId, VMError> {
        let id: ContractId = hash(contract.bytecode()).into();
        self.deploy_with_id(id, contract, config)
    }

    /// Deploys a contract with the given id to the state.
    pub fn deploy_with_id(
        &mut self,
        id: ContractId,
        contract: Contract,
        config: &'static Config,
    ) -> Result<ContractId, VMError> {
        compile_module(contract.bytecode(), config)?;

        self.0.insert(id, contract);

        Ok(id)
    }
}
