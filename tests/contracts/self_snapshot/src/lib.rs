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
use rusk_uplink::{ContractId, Query, RawTransaction, Transaction};

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SelfSnapshot {
    crossover: i32,
}

impl SelfSnapshot {
    pub fn new(init: i32) -> Self {
        SelfSnapshot { crossover: init }
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CrossoverQuery;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SetCrossoverTransaction;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SelfCallTestATransaction;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SelfCallTestBTransaction;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct UpdateAndPanicTransaction;

impl Query for CrossoverQuery {
    const NAME: &'static str = "crossover";
    type Return = i32;
}

impl Transaction for SetCrossoverTransaction {
    const NAME: &'static str = "set_crossover";
    type Return = i32;
}

impl Transaction for SelfCallTestATransaction {
    const NAME: &'static str = "self_call_test_a";
    type Return = i32;
}

impl Transaction for SelfCallTestBTransaction {
    const NAME: &'static str = "self_call_test_b";
    type Return = i32;
}

impl Transaction for UpdateAndPanicTransaction {
    const NAME: &'static str = "update_and_panic";
    type Return = ();
}


#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

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
        pub fn self_call_test_a(&mut self, update: i32) -> i32 {
            let old_value = self.crossover;

            let callee = rusk_uplink::callee();

            rusk_uplink::transact::<_, (), Self>(
                self,
                &callee,
                &(SET_CROSSOVER, update),
                0,
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
        ) -> i32 {
            self.set_crossover(self.crossover * 2);

            rusk_uplink::transact_raw(&target, raw_transaction, 0)
                .unwrap();

            self.crossover
        }

        pub fn update_and_panic(&mut self, new_value: i32) {
            let old_value = self.crossover;

            assert_eq!(self.self_call_test_a(new_value), old_value);

            let callee = rusk_uplink::callee();

            // What should self.crossover be in this case?

            // A: we live with inconsistencies and communicate them.
            // B: we update self, which then should be passed to the transaction

            assert_eq!(
                rusk_uplink::query(&callee, CrossoverQuery, 0).unwrap(),
                new_value
            );

            panic!("OH NOES")
        }
    }

    #[no_mangle]
    fn crossover(written_state: u32, _written_data: u32) -> u32 {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let state: SelfSnapshot = state.deserialize(&mut store).unwrap();

        let ret = state.crossover();

        let res: <CrossoverQuery as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<CrossoverQuery as Query>::Return as Archive>::Archived,
        >();
        buffer_len as u32
    }

    #[no_mangle]
    fn set_crossover(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let to = unsafe {
            archived_root::<i32>(&SCRATCH[..written_data as usize])
        };
        let state: SelfSnapshot = state.deserialize(&mut store).unwrap();
        let to: i32 = to.deserialize(&mut store).unwrap();

        let old = state.set_crossover(to);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<SelfSnapshot as Archive>::Archived>();

        let return_len = ser.serialize_value(&old).unwrap()
            + core::mem::size_of::<
            <<SetCrossoverTransaction as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }

    #[no_mangle]
    fn self_call_test_a(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let update = unsafe {
            archived_root::<i32>(&SCRATCH[..written_data as usize])
        };
        let state: SelfSnapshot = state.deserialize(&mut store).unwrap();
        let update: i32 = update.deserialize(&mut store).unwrap();

        let old = state.self_call_test_a(update);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<SelfSnapshot as Archive>::Archived>();

        let return_len = ser.serialize_value(&old).unwrap()
            + core::mem::size_of::<
            <<SelfCallTestATransaction as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }

    #[no_mangle]
    fn set_call_test_b(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let target_transaction_pair = unsafe {
            archived_root::<(ContractId, RawTransaction)>(&SCRATCH[..written_state as usize])
        };
        let state: SelfSnapshot = state.deserialize(&mut store).unwrap();
        let (target, transaction) = target_transaction_pair.deserialize(&mut store).unwrap();

        let old = state.self_call_test_b(target, transaction);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<SelfSnapshot as Archive>::Archived>();

        let return_len = ser.serialize_value(&old).unwrap()
            + core::mem::size_of::<
            <<SelfCallTestBTransaction as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }

    #[no_mangle]
    fn update_and_panic(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let update = unsafe {
            archived_root::<i32>(&SCRATCH[..written_data as usize])
        };
        let state: SelfSnapshot = state.deserialize(&mut store).unwrap();
        let update: i32 = update.deserialize(&mut store).unwrap();

        state.update_and_panic(update);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<SelfSnapshot as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
            <<UpdateAndPanicTransaction as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }
};
