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
use rusk_uplink::{ContractId, Query, Transaction, Apply, Execute, StoreContext};
extern crate alloc;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CallerState {
    target_address: ContractId,
}

impl CallerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, address: ContractId) {
        self.target_address = address;
    }
}

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct CallerQuery;

impl Query for CallerQuery {
    const NAME: &'static str = "call";
    type Return = ([u8; 32], [u8; 32], [u8; 32]);
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct CallerTransaction {
    target_id: ContractId,
}

impl CallerTransaction {
    pub fn new(target_id: ContractId) -> Self {
        Self { target_id }
    }
}

impl Transaction for CallerTransaction {
    const NAME: &'static str = "set_target";
    type Return = ();
}

impl Apply<CallerTransaction> for CallerState {
    fn apply(
        &mut self,
        target: &CallerTransaction,
        _: StoreContext,
    ) -> <CallerTransaction as Transaction>::Return {
        self.set_target(target.target_id);
        rusk_uplink::debug!(
            "setting state.set_target to: {:?}",
            target.target_id
        );
    }
}


#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Callee1Query {
    sender: ContractId,
}

impl Query for Callee1Query {
    const NAME: &'static str = "call";
    type Return = ([u8; 32], [u8; 32], [u8; 32]);
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::{StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    #[no_mangle]
    fn call(written_state: u32, _written_data: u32) -> u32 {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<CallerState>(&SCRATCH[..written_state as usize])
        };

        let state: CallerState = (state).deserialize(&mut store).unwrap();

        rusk_uplink::debug!(
            "caller: calling state target 'call' with param: callee"
        );
        let call_data = Callee1Query {
            sender: rusk_uplink::callee(),
        };
        let ret = rusk_uplink::query::<Callee1Query>(
            &state.target_address,
            call_data,
            0,
            store,
        )
        .unwrap();

        let res: <Callee1Query as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<Callee1Query as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }

    transaction_state_arg_fun!(set_target, CallerState, CallerTransaction);
};
