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
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};
extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasContextData {
    after_call_gas_limits: Vec<u64>,
    call_gas_limits: Vec<u64>,
}

impl GasContextData {
    pub fn new() -> GasContextData {
        GasContextData {
            after_call_gas_limits: Vec::new(),
            call_gas_limits: Vec::new(),
        }
    }
    pub fn compute_with_transact(
        &mut self,
        n: u64,
        store: StoreContext,
    ) -> u64 {
        if n < 1 {
            0
        } else {
            let callee = rusk_uplink::callee();
            let call_limit = *self
                .call_gas_limits
                .get(n as usize - 1)
                .expect("Call limit out of bounds");
            rusk_uplink::debug!("call TCompute with limit {}", call_limit);
            rusk_uplink::debug!("limits = {:?}", self.call_gas_limits);
            rusk_uplink::transact(
                self,
                &callee,
                TCompute::new(n - 1),
                call_limit,
                store,
            )
            .unwrap();
            self.after_call_gas_limits
                .insert(0, rusk_uplink::gas_left());
            rusk_uplink::debug!(
                "after limits = {:?}",
                self.after_call_gas_limits
            );
            n
        }
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TCompute {
    value: u64,
}

impl TCompute {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

impl Transaction for TCompute {
    const NAME: &'static str = "t_compute";
    type Return = u64;
}

impl Apply<TCompute> for GasContextData {
    fn apply(
        &mut self,
        input: &TCompute,
        store: StoreContext,
    ) -> <TCompute as Transaction>::Return {
        self.compute_with_transact(input.value, store)
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SetGasLimits {
    limits: Vec<u64>,
}

impl SetGasLimits {
    pub fn new(limits: impl AsRef<[u64]>) -> Self {
        let limits = Vec::from(limits.as_ref());
        Self { limits }
    }
}

impl Transaction for SetGasLimits {
    const NAME: &'static str = "set_gas_limits";
    type Return = ();
}

impl Apply<SetGasLimits> for GasContextData {
    fn apply(
        &mut self,
        limits: &SetGasLimits,
        _: StoreContext,
    ) -> <SetGasLimits as Transaction>::Return {
        self.call_gas_limits = limits.limits.to_vec();
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct ReadGasLimits;

impl Query for ReadGasLimits {
    const NAME: &'static str = "read_gas_limits";
    type Return = Box<[u64]>;
}

impl Execute<ReadGasLimits> for GasContextData {
    fn execute(
        &self,
        _: &ReadGasLimits,
        _: StoreContext,
    ) -> <ReadGasLimits as Query>::Return {
        Box::from(&self.after_call_gas_limits[..])
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    q_handler_store_ser!(
        read_gas_limits,
        GasContextData,
        ReadGasLimits
    );

    t_handler_store_ser!(t_compute, GasContextData, TCompute);

    t_handler_store_ser!(
        set_gas_limits,
        GasContextData,
        SetGasLimits
    );
};
