// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use canonical::{Canon, Ident, Store};
use canonical_derive::Canon;
use dusk_abi::{Query, Transaction};
use dusk_kelvin_map::Map;

use crate::call_context::{CallContext, Resolver};
use crate::contract::{Contract, ContractId};
use crate::gas::GasMeter;
use crate::VMError;

/// The main network state, includes the full state of contracts.
#[derive(Clone, Default, Canon)]
pub struct NetworkState<E, S>
where
    S: Store,
{
    contracts: Map<ContractId, Contract, S>,
    store: S,
    _marker: PhantomData<E>,
}

impl<E, S> NetworkState<E, S>
where
    E: Resolver<S>,
    S: Store,
{
    /// Deploys a contract to the state, returns the address of the created
    /// contract or an error
    pub fn deploy(
        &mut self,
        contract: Contract,
    ) -> Result<ContractId, S::Error> {
        let id: ContractId = S::Ident::from_bytes(contract.bytecode()).into();

        self.contracts
            .insert(id.clone(), contract)
            .expect("FIXME: error handling");
        Ok(id)
    }

    /// Returns a reference to the specified contracts state
    pub fn get_contract<'a>(
        &'a self,
        contract_id: &ContractId,
    ) -> Result<Option<impl Deref<Target = Contract> + 'a>, VMError<S>> {
        self.contracts
            .get(contract_id)
            .map_err(VMError::from_store_error)
    }

    /// Returns a reference to the specified contracts state
    pub fn get_contract_mut<'a>(
        &'a mut self,
        contract_id: &ContractId,
    ) -> Result<Option<impl DerefMut<Target = Contract> + 'a>, VMError<S>> {
        self.contracts
            .get_mut(contract_id)
            .map_err(VMError::from_store_error)
    }

    /// Returns a reference to the store backing the state
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Queryn the contract at address `target`
    pub fn query<A, R>(
        &mut self,
        target: ContractId,
        query: A,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError<S>>
    where
        A: Canon<S>,
        R: Canon<S>,
    {
        let store = self.store().clone();
        let mut context = CallContext::new(self, gas_meter, &store)
            .expect("FIXME: error handling");

        let result = context.query(
            target,
            Query::from_canon(&query, &store)
                .map_err(VMError::from_store_error)?,
        )?;

        result.cast(store).map_err(VMError::from_store_error)
    }

    /// Transact with the contract at address `target`
    pub fn transact<A, R>(
        &mut self,
        target: ContractId,
        transaction: A,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError<S>>
    where
        A: Canon<S>,
        R: Canon<S>,
    {
        let store = self.store().clone();
        let mut context = CallContext::new(self, gas_meter, &store)
            .expect("FIXME: error handling");

        let result = context.transact(
            target,
            Transaction::from_canon(&transaction, &store)
                .map_err(VMError::from_store_error)?,
        )?;

        result.cast(store).map_err(VMError::from_store_error)
    }
}
