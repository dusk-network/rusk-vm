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

use rkyv::{Archive, Deserialize, Serialize, AlignedVec};
use rusk_uplink::{ContractId, Query, Transaction, Apply, Execute, StoreContext, RawTransaction};
extern crate alloc;
use alloc::boxed::Box;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SelfSnapshot {
    crossover: i32,
}

impl SelfSnapshot {
    pub fn new(init: i32) -> Self {
        SelfSnapshot { crossover: init }
    }

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

    pub fn update_and_panic(
        &mut self,
        new_value: i32,
        store: StoreContext,
    ) {
        let old_value = self.crossover;

        assert_eq!(
            self.self_call_test_a(new_value, store.clone()),
            old_value
        );

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

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CrossoverQuery;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SetCrossoverTransaction {
    crossover: i32,
}

impl SetCrossoverTransaction {
    pub fn new(crossover: i32) -> Self {
        Self { crossover }
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SelfCallTestATransaction {
    update: i32,
}

impl SelfCallTestATransaction {
    pub fn new(update: i32) -> Self {
        Self { update }
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct UpdateAndPanicTransaction {
    update: i32,
}

impl UpdateAndPanicTransaction {
    pub fn new(update: i32) -> Self {
        Self { update }
    }
}

impl Query for CrossoverQuery {
    const NAME: &'static str = "crossover";
    type Return = i32;
}

impl Execute<CrossoverQuery> for SelfSnapshot {
    fn execute(
        &self,
        _: &CrossoverQuery,
        _: StoreContext,
    ) -> <CrossoverQuery as Query>::Return {
        self.crossover
    }
}

impl Transaction for SetCrossoverTransaction {
    const NAME: &'static str = "set_crossover";
    type Return = i32;
}

impl Apply<SetCrossoverTransaction> for SelfSnapshot {
    fn apply(
        &mut self,
        to: &SetCrossoverTransaction,
        _: StoreContext,
    ) -> <SetCrossoverTransaction as Transaction>::Return {
        self.set_crossover(to.crossover)
    }
}

impl Transaction for SelfCallTestATransaction {
    const NAME: &'static str = "self_call_test_a";
    type Return = i32;
}

impl Apply<SelfCallTestATransaction> for SelfSnapshot {
    fn apply(
        &mut self,
        update: &SelfCallTestATransaction,
        store: StoreContext,
    ) -> <SelfCallTestATransaction as Transaction>::Return {
        self.self_call_test_a(update.update, store)
    }
}

impl Transaction for SelfCallTestBTransaction {
    const NAME: &'static str = "self_call_test_b";
    type Return = i32;
}

impl Apply<SelfCallTestBTransaction> for SelfSnapshot {
    fn apply(
        &mut self,
        arg: &SelfCallTestBTransaction,
        store: StoreContext,
    ) -> <SelfCallTestBTransaction as Transaction>::Return {
        let mut tx_data = AlignedVec::new();
        tx_data.extend_from_slice(arg.tx_data.as_ref());
        let raw_transaction = RawTransaction::from(tx_data, &arg.tx_name);
        self.self_call_test_b(arg.contract_id, &raw_transaction, store)
    }
}

impl Transaction for UpdateAndPanicTransaction {
    const NAME: &'static str = "update_and_panic";
    type Return = ();
}

impl Apply<UpdateAndPanicTransaction> for SelfSnapshot {
    fn apply(
        &mut self,
        update_and_panic: &UpdateAndPanicTransaction,
        store: StoreContext,
    ) -> <UpdateAndPanicTransaction as Transaction>::Return {
        self.update_and_panic(update_and_panic.update, store);
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rkyv::{archived_root, AlignedVec};
    use rusk_uplink::{AbiStore, StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    #[no_mangle]
    fn crossover(written_state: u32, _written_data: u32) -> u32 {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let state: SelfSnapshot = state.deserialize(&mut store).unwrap();

        let ret = state.execute(&CrossoverQuery, store);

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
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let to = unsafe {
            archived_root::<i32>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };
        let mut state: SelfSnapshot = state.deserialize(&mut store).unwrap();
        let to: i32 = to.deserialize(&mut store).unwrap();

        let old = state.apply(&SetCrossoverTransaction::new(to), store);

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
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let update = unsafe {
            archived_root::<i32>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };
        let mut state: SelfSnapshot = state.deserialize(&mut store).unwrap();
        let update: i32 = update.deserialize(&mut store).unwrap();

        let old = state.apply(&SelfCallTestATransaction::new(update), store);

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
    fn self_call_test_b(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let arg = unsafe {
            archived_root::<SelfCallTestBTransaction>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };
        let mut state: SelfSnapshot = state.deserialize(&mut store).unwrap();
        let arg: SelfCallTestBTransaction =
            arg.deserialize(&mut store).unwrap();

        let old = state.apply(&arg, store);

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
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<SelfSnapshot>(&SCRATCH[..written_state as usize])
        };
        let update = unsafe {
            archived_root::<i32>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };
        let mut state: SelfSnapshot = state.deserialize(&mut store).unwrap();
        let update: i32 = update.deserialize(&mut store).unwrap();

        state.apply(&UpdateAndPanicTransaction::new(update), store);

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
