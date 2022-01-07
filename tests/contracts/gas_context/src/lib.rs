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
use rusk_uplink::{Query, Transaction};
extern crate alloc;
use alloc::boxed::Box;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasContextData {
    after_call_gas_limits: Box<[u64]>,
    call_gas_limits: Box<[u64]>,
}

impl GasContextData {
    pub fn new() -> GasContextData {
        GasContextData {
            after_call_gas_limits: Box::from([]),
            call_gas_limits: Box::from([]),
        }
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
struct QCompute {
    value: u64,
}

impl QCompute {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

impl Query for QCompute {
    const NAME: &'static str = "q_compute";
    type Return = u64;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
struct TCompute {
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

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
struct SetGasLimits {
    limits: Box<[u64]>
}

impl SetGasLimits {
    pub fn new(limits: impl AsRef<[u64]>) -> Self {
        let limits = Box::from(limits.as_ref());
        Self { limits }
    }
}

impl Transaction for SetGasLimits {
    const NAME: &'static str = "set_gas_limits";
    type Return = ();
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
struct ReadGasLimits;

impl Query for ReadGasLimits {
    const NAME: &'static str = "read_gas_limits";
    type Return = Box<[u64]>;
}


#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    impl GasContextData {
        pub fn compute_with_transact(&mut self, n: u64) -> u64 {
            if n < 1 {
                0
            } else {
                let callee = rusk_uplink::callee();
                let call_limit = *self
                    .call_gas_limits
                    .get(n as usize - 1)
                    .expect("Call limit out of bounds");
                rusk_uplink::transact(
                    self,
                    &callee,
                    &(COMPUTE, n - 1),
                    call_limit,
                )
                .unwrap();
                self.after_call_gas_limits.insert(0, dusk_abi::gas_left());
                n
            }
        }
        pub fn compute_with_query(&mut self, n: u64) -> u64 {
            if n < 1 {
                0
            } else {
                let callee = rusk_uplink::callee();
                let call_limit = *self
                    .call_gas_limits
                    .get(n as usize - 1)
                    .expect("Call limit out of bounds");
                dusk_abi::query(
                    &callee,
                    &(COMPUTE, n - 1),
                    call_limit,
                )
                .unwrap();
                self.after_call_gas_limits.insert(0, dusk_abi::gas_left());
                n
            }
        }
    }

    fn read_gas_limits(written_state: u32, _written_data: u32) -> u32 {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<GasContextData>(&SCRATCH[..written_state as usize])
        };
        let mut state: GasContextData = state.deserialize(&mut store).unwrap();

        let ret = slf.after_call_gas_limits;

        let res: <ReadGasLimits as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<ReadGasLimits as Query>::Return as Archive>::Archived,
        >();
        buffer_len as u32
    }

    fn q_compute(written_state: u32, written_data: u32) -> u32 {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<GasContextData>(&SCRATCH[..written_state as usize])
        };
        let input = unsafe {
            archived_root::<u64>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let state: GasContextData = state.deserialize(&mut store).unwrap();
        let input: u64 = input.deserialize(&mut store).unwrap();

        let ret: u64 = state.compute_with_transact(input);

        let res: <QCompute as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<QCompute as Query>::Return as Archive>::Archived,
        >();
        buffer_len as u32
    }

    fn t_compute(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<GasContextData>(&SCRATCH[..written_state as usize])
        };
        let input = unsafe {
            archived_root::<u64>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let state: GasContextData = state.deserialize(&mut store).unwrap();
        let input: u64 = input.deserialize(&mut store).unwrap();

        let ret: u64 = state.compute_with_query(input);

        let res: <QCompute as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<GasContextData as Archive>::Archived>();
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<QCompute as Query>::Return as Archive>::Archived,
        >();
        [state_len as u32, return_len as u32]
    }

    fn set_gas_limits(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<GasContextData>(&SCRATCH[..written_state as usize])
        };
        let limits = unsafe {
            archived_root::<SetGasLimits>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };
        let mut state: GasContextData = state.deserialize(&mut store).unwrap();
        let limits: SetGasLimits = to.deserialize(&mut store).unwrap();

        slf.call_gas_limits = limits.limits;

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<GasContextData as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
            <<SetGasLimits as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }

};
