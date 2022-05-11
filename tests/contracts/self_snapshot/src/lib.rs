// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use rkyv::{AlignedVec, Archive, Deserialize, Serialize};
use rusk_uplink::{
    Apply, ContractId, Execute, Query, RawTransaction, StoreContext,
    Transaction,
};
extern crate alloc;
use alloc::boxed::Box;
use rusk_uplink_derive::{apply, execute, init, query, state, transaction};

#[state]
pub struct SelfSnapshot {
    crossover: i32,
}

#[init]
fn init() {}

impl SelfSnapshot {
    pub fn crossover(&self) -> i32 {
        self.crossover
    }

    pub fn set_crossover(&mut self, to: i32) -> i32 {
        let old_val = self.crossover;
        rusk_uplink::debug!(
            "setting crossover from {:?} to {:?}",
            self.crossover,
            to
        );
        self.crossover = to;
        old_val
    }

    // updates crossover and returns the old value
    pub fn self_call_test_a(
        &mut self,
        update: i32,
        store: StoreContext,
    ) -> i32 {
        let old_value = self.crossover;
        let callee = rusk_uplink::callee();
        rusk_uplink::transact(
            self,
            &callee,
            SetCrossoverTransaction::new(update),
            0,
            store,
        )
        .unwrap();
        assert_eq!(self.crossover, update);
        old_value
    }

    // updates crossover and returns the old value
    pub fn self_call_test_b(
        &mut self,
        target: ContractId,
        raw_transaction: &RawTransaction,
        store: StoreContext,
    ) -> i32 {
        self.set_crossover(self.crossover * 2);
        rusk_uplink::transact_raw(self, &target, raw_transaction, 0, store)
            .unwrap();
        self.crossover
    }

    pub fn update_and_panic(&mut self, new_value: i32, store: StoreContext) {
        let old_value = self.crossover;

        assert_eq!(self.self_call_test_a(new_value, store.clone()), old_value);

        let callee = rusk_uplink::callee();

        // What should self.crossover be in this case?

        // A: we live with inconsistencies and communicate them.
        // B: we update self, which then should be passed to the transaction

        assert_eq!(
            rusk_uplink::query(&callee, CrossoverQuery, 0, store).unwrap(),
            new_value
        );

        panic!("OH NOES")
    }
}

#[query]
pub struct CrossoverQuery;

impl Query for CrossoverQuery {
    const NAME: &'static str = "crossover";
    type Return = i32;
}

#[transaction]
pub struct SetCrossoverTransaction {
    crossover: i32,
}

impl Transaction for SetCrossoverTransaction {
    const NAME: &'static str = "set_crossover";
    type Return = i32;
}

#[transaction]
pub struct SelfCallTestATransaction {
    update: i32,
}

impl Transaction for SelfCallTestATransaction {
    const NAME: &'static str = "self_call_test_a";
    type Return = i32;
}

#[transaction(new = false)]
pub struct SelfCallTestBTransaction {
    contract_id: ContractId,
    tx_data: Box<[u8]>,
    tx_name: Box<str>,
}

impl SelfCallTestBTransaction {
    pub fn new(
        contract_id: ContractId,
        data: impl AsRef<[u8]>,
        name: impl AsRef<str>,
    ) -> Self {
        let tx_data = Box::from(data.as_ref());
        let tx_name = Box::from(name.as_ref());
        Self {
            contract_id,
            tx_data,
            tx_name,
        }
    }
}

impl Transaction for SelfCallTestBTransaction {
    const NAME: &'static str = "self_call_test_b";
    type Return = i32;
}

#[transaction]
pub struct UpdateAndPanicTransaction {
    update: i32,
}

impl Transaction for UpdateAndPanicTransaction {
    const NAME: &'static str = "update_and_panic";
    type Return = ();
}

#[execute(name = "crossover")]
impl Execute<CrossoverQuery> for SelfSnapshot {
    fn execute(&self, _: CrossoverQuery, _: StoreContext) -> i32 {
        self.crossover
    }
}

#[apply(name = "set_crossover")]
impl Apply<SetCrossoverTransaction> for SelfSnapshot {
    fn apply(&mut self, to: SetCrossoverTransaction, _: StoreContext) -> i32 {
        self.set_crossover(to.crossover)
    }
}

#[apply(name = "self_call_test_a")]
impl Apply<SelfCallTestATransaction> for SelfSnapshot {
    fn apply(
        &mut self,
        update: SelfCallTestATransaction,
        store: StoreContext,
    ) -> i32 {
        self.self_call_test_a(update.update, store)
    }
}

#[apply(name = "self_call_test_b")]
impl Apply<SelfCallTestBTransaction> for SelfSnapshot {
    fn apply(
        &mut self,
        arg: SelfCallTestBTransaction,
        store: StoreContext,
    ) -> i32 {
        let mut tx_data = AlignedVec::new();
        tx_data.extend_from_slice(arg.tx_data.as_ref());
        let raw_transaction = RawTransaction::from(tx_data, &arg.tx_name);
        self.self_call_test_b(arg.contract_id, &raw_transaction, store)
    }
}

#[apply(name = "update_and_panic")]
impl Apply<UpdateAndPanicTransaction> for SelfSnapshot {
    fn apply(
        &mut self,
        update_and_panic: UpdateAndPanicTransaction,
        store: StoreContext,
    ) {
        self.update_and_panic(update_and_panic.update, store);
    }
}
