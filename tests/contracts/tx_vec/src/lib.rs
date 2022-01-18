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
use rusk_uplink::{Apply, ContractId, Execute, Query, Transaction, transaction_state_arg_fun};
use rusk_uplink::{get_state, get_state_and_arg, q_return, t_return, query_state_arg_fun};
use rusk_uplink::StoreContext;
extern crate alloc;
use alloc::boxed::Box;

// #[macro_use]
// extern crate rusk_uplink;


#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVec {
    value: u8,
}

impl TxVec {
    pub fn new(value: u8) -> Self {
        TxVec { value }
    }
}

use rusk_uplink::{AbiStore, RawTransaction};
impl TxVec {
    pub fn read_value(&self) -> u8 {
        self.value
    }

    pub fn sum(&mut self, values: impl AsRef<[u8]>) {
        let values: &[u8] = &Box::from(values.as_ref());
        self.value +=
            values.into_iter().fold(0u8, |s, v| s.wrapping_add(*v));
    }

    pub fn delegate_sum(
        &mut self,
        target: &ContractId,
        data: impl AsRef<[u8]>,
        store: StoreContext,
    ) -> () {
        let tx_vec_sum = TxVecSum::new(data);
        let raw_transaction = RawTransaction::new(tx_vec_sum);
        let ret = rusk_uplink::transact_raw(
            self,
            target,
            &raw_transaction,
            0,
            store,
        )
            .unwrap();
        self.value = *ret.cast::<u8>().unwrap();
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVecReadValue;

impl Query for TxVecReadValue {
    const NAME: &'static str = "read_value";
    type Return = u8;
}

impl Execute<TxVecReadValue> for TxVec {
    fn execute(
        &self,
        _: &TxVecReadValue,
        _: StoreContext,
    ) -> <TxVecReadValue as Query>::Return {
        self.read_value()
    }
}

impl Apply<TxVecSum> for TxVec {
    fn apply(
        &mut self,
        s: &TxVecSum,
    ) -> <TxVecSum as Transaction>::Return {
        self.sum(&s.values);
        self.value
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

#[cfg(target_family = "wasm")]
const _: () = {

    #[no_mangle]
    static mut SCRATCH: [u8; 8192] = [0u8; 8192];

    // #[no_mangle]
    // fn read_value(written_state: u32, _written_data: u32) -> u32 {
    //     let slf: TxVec = unsafe { get_state(written_state, &SCRATCH) };
    //
    //     let ret: <TxVecReadValue as Query>::Return = slf.read_value();
    //
    //     unsafe { q_return(&ret, &mut SCRATCH) }
    // }
    query_state_arg_fun!(read_value, TxVec, TxVecReadValue);

    // #[no_mangle]
    // fn sum(written_state: u32, written_data: u32) -> [u32; 2] {
    //     let (mut slf, de_arg): (TxVec, TxVecSum) = unsafe { get_state_and_arg(written_state, written_data, &SCRATCH) };
    //
    //     let res = slf.apply(&de_arg);
    //
    //     unsafe { t_return(&slf, &res, &mut SCRATCH)}
    // }
    transaction_state_arg_fun!(sum, TxVec, TxVecSum);

    #[no_mangle]
    fn delegate_sum(written_state: u32, written_data: u32) -> [u32; 2] {
        let (mut slf, de_arg): (TxVec, TxVecDelegateSum) = unsafe { get_state_and_arg(written_state, written_data, &SCRATCH) };

        let store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
        slf.delegate_sum(&de_arg.contract_id, de_arg.data, store);

        unsafe { t_return(&slf, &(), &mut SCRATCH)}
    }
};
