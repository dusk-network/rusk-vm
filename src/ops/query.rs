// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::ops::AbiCall;
use crate::VMError;

use canonical::{ByteSink, ByteSource, Canon, Store};
use dusk_abi::ContractId;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct ExecuteQuery;

impl<S: Store> AbiCall<S> for ExecuteQuery {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let [RuntimeValue::I32(contract_id_ofs), RuntimeValue::I32(query_ofs)] =
            *args.as_ref()
        {
            let contract_id_ofs = contract_id_ofs as usize;
            let query_ofs = query_ofs as usize;

            let (contract_id, query) = context
                .memory(|m| {
                    let contract_id = ContractId::from(
                        &m[contract_id_ofs..contract_id_ofs + 32],
                    );

                    let mut source =
                        ByteSource::new(&m[query_ofs..], context.store());

                    let query = Canon::<S>::read(&mut source)?;

                    Ok((contract_id, query))
                })
                .map_err(VMError::from_store_error)?;

            let result = context.query(contract_id, query)?;

            let store = context.store().clone();

            context
                .memory_mut(|m| {
                    // write back the return value
                    let mut sink = ByteSink::new(&mut m[query_ofs..], &store);
                    Canon::<S>::write(&result, &mut sink)
                })
                .map_err(VMError::from_store_error)?;

            Ok(None)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
