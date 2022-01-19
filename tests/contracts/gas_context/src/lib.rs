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
use rusk_uplink::{Query, Apply, Execute, Transaction, StoreContext};
extern crate alloc;
use alloc::vec::Vec;
use alloc::boxed::Box;

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
            rusk_uplink::debug!("after limits = {:?}", self.after_call_gas_limits);
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
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::{AbiStore, StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    use rusk_uplink::q_return_store;
    rusk_uplink::query_state_arg_fun_store!(read_gas_limits, GasContextData, ReadGasLimits);

    use rusk_uplink::{get_state_and_arg, t_return_store};
    rusk_uplink::transaction_state_arg_fun_store!(t_compute, GasContextData, TCompute);

    #[no_mangle]
    fn set_gas_limits(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<GasContextData>(&SCRATCH[..written_state as usize])
        };
        let limits = unsafe {
            archived_root::<SetGasLimits>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };
        let mut state: GasContextData = state.deserialize(&mut store).unwrap();
        let limits: SetGasLimits = limits.deserialize(&mut store).unwrap();

        rusk_uplink::debug!("setting limits to {:?}", limits.limits);
        state.call_gas_limits = limits.limits;

        let mut ser = store.serializer();

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<GasContextData as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
                <<SetGasLimits as Transaction>::Return as Archive>::Archived,
            >();

        [state_len as u32, return_len as u32]
    }
};
