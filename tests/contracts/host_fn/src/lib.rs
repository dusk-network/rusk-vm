// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const PLZ_CALL: u8 = 0;

// transaction ids
pub const MUTATO: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct HostFnTest {}

impl HostFnTest {
    pub fn new() -> Self {
        HostFnTest {}
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};
    use dusk_abi::{ContractId, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl HostFnTest {
        pub fn please_call_host_fn_ok(&self) -> i32 {
            const HOST_MODULE_ID: ContractId = ContractId::reserved(0);
            const TEST_HOST_CALL: u8 = 0;

            let argument: u128 = 1994;

            dusk_abi::query(&HOST_MODULE_ID, &(TEST_HOST_CALL, argument))
                .unwrap()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(&bytes[..], &bs);

        // read self.
        let slf: HostFnTest = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            PLZ_CALL => {
                let ret = slf.please_call_host_fn_ok();

                let r = {
                    // return value
                    let wrapped_return = ReturnValue::from_canon(&ret, &bs)?;

                    let mut sink = ByteSink::new(&mut bytes[..], &bs);

                    Canon::<BS>::write(&wrapped_return, &mut sink)
                };

                r
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = query(bytes);
    }

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(bytes, &bs);

        // read self.
        let mut _slf: HostFnTest = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let tid: u8 = Canon::<BS>::read(&mut source)?;
        match tid {
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }
}
