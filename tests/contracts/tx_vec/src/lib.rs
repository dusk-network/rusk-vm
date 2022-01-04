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

use rkyv::{AlignedVec, Archive, Deserialize, Serialize};
use rusk_uplink::{ContractId, Query, ReturnValue, Transaction, RawTransaction};
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

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVecReadValue;

impl Query for TxVecReadValue {
    const NAME: &'static str = "read_value";
    type Return = u8;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVecSum;

impl Transaction for TxVecSum {
    const NAME: &'static str = "sum";
    type Return = ();
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct TxVecDelegateSum;

impl Transaction for TxVecDelegateSum {
    const NAME: &'static str = "delegate_sum";
    type Return = ();
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct SumParam {
    values: Box<[u8]>,
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct DelegateSumParam {
    contract_id: ContractId,
    data: Box<[u8]>,
    name: Box<str>,
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::AbiStore;

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
            name: impl AsRef<str>,
        ) -> ReturnValue {
            let name = Box::from(name.as_ref());
            let mut tx_data = AlignedVec::new();
            tx_data.extend_from_slice(data.as_ref());
            let raw_transaction = RawTransaction::from(tx_data, &name);
            rusk_uplink::transact_raw(target, &raw_transaction, 0).unwrap()
        }
    }

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    #[no_mangle]
    fn read_value(written: u32) -> u32 {
        let mut store = AbiStore;

        let slf =
            unsafe { archived_root::<TxVec>(&SCRATCH[..written as usize]) };

        let mut slf: TxVec = (slf).deserialize(&mut store).unwrap();
        let ret = slf.read_value();

        let res: <TxVecReadValue as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<TxVecReadValue as Query>::Return as Archive>::Archived,
        >();
        buffer_len as u32
    }

    fn sum(written: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let (slf, arg) =
            unsafe { archived_root::<(TxVec, SumParam)>(&SCRATCH[..written as usize]) };

        let mut slf: TxVec = (slf).deserialize(&mut store).unwrap();
        let mut de_arg: SumParam = (arg).deserialize(&mut store).unwrap();

        slf.sum(de_arg.values);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&slf).unwrap()
            + core::mem::size_of::<<TxVec as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
            <<TxVecDelegateSum as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }

    fn delegate_sum(written: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let (slf, arg) = unsafe {
            archived_root::<(TxVec, DelegateSumParam)>(
                &SCRATCH[..written as usize]
            )
        };

        let mut slf: TxVec = (slf).deserialize(&mut store).unwrap();
        let mut de_arg: DelegateSumParam = (arg).deserialize(&mut store).unwrap();

        slf.delegate_sum(&de_arg.contract_id, de_arg.data, de_arg.name);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&slf).unwrap()
            + core::mem::size_of::<<TxVec as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
            <<TxVecDelegateSum as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }
};
