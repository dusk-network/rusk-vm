// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::ops::AbiCall;
use crate::VMError;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct ExecuteQuery;

impl AbiCall for ExecuteQuery {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(contract_id_ofs), RuntimeValue::I32(query_ofs)] =
            *args.as_ref()
        {
            let contract_id_ofs = contract_id_ofs as usize;
            let query_ofs = query_ofs as usize;

            // let (contract_id,) = context.memory(|m| {
            //     // let contract_id =
            //     //     ContractId::from(&m[contract_id_ofs..contract_id_ofs +
            //     // 32]);

            //     // let mut source = todo!();
            //     // let query = todo!();

            //     // Ok((contract_id, query))
            //     todo!()
            // });

            // let result = context.query(contract_id)?;

            // context.memory_mut(|m| {
            //     // write back the return value
            //     //let mut sink = Sink::new(&mut m[query_ofs..]);
            //     //result.encode(&mut sink);
            //     todo!();
            // });

            todo!();

            Ok(None)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
