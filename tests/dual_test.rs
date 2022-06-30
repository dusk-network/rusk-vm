// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fmt::Debug;

use bytecheck::CheckBytes;
use microkelvin::{HostStore, OffsetLen, StoreRef, StoreSerializer};
use rkyv::{
    validation::validators::DefaultValidator, Archive, Deserialize, Serialize,
};
use rusk_uplink::{Apply, Execute, Query, Transaction};
use rusk_vm::{Contract, ContractId, GasMeter, NetworkState};

pub struct DualTest<S> {
    store: StoreRef<OffsetLen>,
    contract_id: ContractId,
    network: NetworkState,
    state: S,
}

impl<S> DualTest<S>
where
    S: Serialize<StoreSerializer<OffsetLen>>,
{
    pub fn new(state: S, code: &[u8]) -> Self {
        let store = StoreRef::new(HostStore::new());
        let contract = Contract::new(&state, code.to_vec(), &store);
        let mut network = NetworkState::new(store.clone());
        let contract_id = network.deploy(contract).unwrap();

        DualTest {
            store,
            network,
            state,
            contract_id,
        }
    }

    pub fn execute<Q>(&mut self, q: Q) -> Q::Return
    where
        S: Execute<Q>,
        Q: Query + Clone + Serialize<StoreSerializer<OffsetLen>>,
        Q::Return: Archive + PartialEq + Debug,
        <Q::Return as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
            + Deserialize<Q::Return, StoreRef<OffsetLen>>,
    {
        let mut gas = GasMeter::with_limit(1_000_000_000);

        let a = self.state.execute(q.clone(), self.store.clone());

        let b = self
            .network
            .query(self.contract_id, 0, q, &mut gas)
            .unwrap();

        assert_eq!(a, *b, "Direct call and wasm transaction differ in result");
        a
    }

    pub fn apply<T>(&mut self, t: T) -> T::Return
    where
        S: Apply<T>,
        T: Transaction + Clone + Serialize<StoreSerializer<OffsetLen>>,
        T::Return: Archive + PartialEq + Debug,
        <T::Return as Archive>::Archived: for<'a> CheckBytes<DefaultValidator<'a>>
            + Deserialize<T::Return, StoreRef<OffsetLen>>,
    {
        let mut gas = GasMeter::with_limit(1_000_000_000);

        let a = self.state.apply(t.clone(), self.store.clone());

        let (b, network) = self
            .network
            .transact(self.contract_id, 0, t, &mut gas)
            .unwrap();
        self.network = network;

        assert_eq!(a, *b, "Direct call and wasm transaction differ in result");
        a
    }
}
