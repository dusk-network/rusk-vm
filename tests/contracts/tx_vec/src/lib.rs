// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(
    core_intrinsics,
    lang_items,
    alloc_error_handler,
    option_result_unwrap_unchecked
)]

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::StoreContext;
use rusk_uplink::{
    Apply, ContractId, Execute, Query, RawTransaction, Transaction,
};
use rusk_uplink_derive::{query, transaction};

extern crate alloc;
use alloc::boxed::Box;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVec {
    value: u8,
}

impl TxVec {
    pub fn new(value: u8) -> Self {
        TxVec { value }
    }
}

impl TxVec {
    pub fn read_value(&self) -> u8 {
        self.value
    }

    pub fn sum(&mut self, values: impl AsRef<[u8]>) {
        let values: &[u8] = &Box::from(values.as_ref());
        self.value += values.into_iter().fold(0u8, |s, v| s.wrapping_add(*v));
    }

    pub fn delegate_sum(
        &mut self,
        target: &ContractId,
        data: impl AsRef<[u8]>,
        store: StoreContext,
    ) {
        let tx_vec_sum = TxVecSum::new(data);
        let raw_transaction = RawTransaction::new(tx_vec_sum, &store);
        let ret =
            rusk_uplink::transact_raw(self, target, &raw_transaction, 0, store)
                .unwrap();
        self.value = *ret.cast::<u8>().unwrap();
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVecReadValue;

#[query(name="read_value")]
impl Execute<TxVecReadValue> for TxVec {
    fn execute(
        &self,
        _: TxVecReadValue,
        _: StoreContext,
    ) -> u8 {
        self.read_value()
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVecSum {
    values: Box<[u8]>,
}

impl TxVecSum {
    pub fn new(v: impl AsRef<[u8]>) -> Self {
        Self {
            values: Box::from(v.as_ref()),
        }
    }
}

impl Transaction for TxVecSum {
    const NAME: &'static str = "sum";
    type Return = u8;
}

impl Apply<TxVecSum> for TxVec {
    fn apply(
        &mut self,
        s: TxVecSum,
        _: StoreContext,
    ) -> <TxVecSum as Transaction>::Return {
        self.sum(&s.values);
        self.value
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVecDelegateSum {
    contract_id: ContractId,
    data: Box<[u8]>,
}

impl TxVecDelegateSum {
    pub fn new(contract_id: ContractId, data: impl AsRef<[u8]>) -> Self {
        let data = Box::from(data.as_ref());
        Self { contract_id, data }
    }
}

impl Transaction for TxVecDelegateSum {
    const NAME: &'static str = "delegate_sum";
    type Return = ();
}

impl Apply<TxVecDelegateSum> for TxVec {
    fn apply(
        &mut self,
        s: TxVecDelegateSum,
        store: StoreContext,
    ) -> <TxVecDelegateSum as Transaction>::Return {
        self.delegate_sum(&s.contract_id, &s.data, store)
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    scratch_memory!(8192);

    t_handler!(_sum, TxVec, TxVecSum);

    t_handler!(_delegate_sum, TxVec, TxVecDelegateSum);
};
