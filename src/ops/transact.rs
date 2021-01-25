// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::{CallContext, Resolver};
use crate::ops::AbiCall;
use crate::VMError;

use canonical::{ByteSink, ByteSource, Canon, Store};
use dusk_abi::ContractId;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct ApplyTransaction;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for ApplyTransaction {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let &[RuntimeValue::I32(contract_id_ofs), RuntimeValue::I32(transaction_ofs)] =
            args.as_ref()
        {
            let contract_id_ofs = contract_id_ofs as usize;
            let transaction_ofs = transaction_ofs as usize;

            let (contract_id, transaction) = context
                .memory(|m| {
                    let contract_id = ContractId::from(
                        &m[contract_id_ofs..contract_id_ofs + 32],
                    );

                    let mut source =
                        ByteSource::new(&m[transaction_ofs..], context.store());

                    let transaction = Canon::<S>::read(&mut source)?;

                    Ok((contract_id, transaction))
                })
                .map_err(VMError::from_store_error)?;

            let result = context.transact(contract_id, transaction)?;

            let store = context.store().clone();

            context
                .memory_mut(|m| {
                    // write back the return value
                    let mut sink =
                        ByteSink::new(&mut m[transaction_ofs..], &store);
                    Canon::<S>::write(&result, &mut sink)
                })
                .map_err(VMError::from_store_error)?;

            Ok(None)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
