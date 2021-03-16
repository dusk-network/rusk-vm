// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::Resolver;
use crate::ops::*;
use crate::VMError;

use wasmi::{
    self, FuncInstance, FuncRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Signature,
};

use crate::call_context::{CallContext, Invoke};

macro_rules! abi_resolver {
    ( $visibility:vis $name:ident { $( $id:expr, $op_name:expr => $op:path ),* } ) => {

        #[doc(hidden)]
        #[derive(Clone, Default)]
        $visibility struct $name;

        impl ModuleImportResolver for $name {
            fn resolve_func(&self, field_name: &str, _signature: &Signature) -> Result<FuncRef, wasmi::Error>
            where $(
                $op : AbiCall,
                )*
            {
                match field_name {
                    $(
                        $op_name => Ok(FuncInstance::alloc_host(
                            Signature::new(<$op as AbiCall>::ARGUMENTS,
                                           <$op as AbiCall>::RETURN),
                            $id,
                        ))
                    ),*

                    ,

                    _ => panic!("invalid function name {:?}", field_name)
                }
            }
        }

        impl Invoke for $name {
            fn invoke(
                context: &mut CallContext,
                index: usize,
                args: RuntimeArgs) -> Result<Option<RuntimeValue>, VMError> {

                match index {
                    $(
                        $id => <$op as AbiCall>::call(context, args)
                    ),*

                    ,

                    _ => panic!("invalid index {:?}", index)
                }
            }
        }

        impl Resolver for $name {}
    };
}

abi_resolver! {
    pub CompoundResolver {
        0, "sig" => panic::Panic,
        1, "debug" => debug::Debug,
        2, "get" => store::Get,
        3, "put" => store::Put,
        6, "query" => query::ExecuteQuery,
        7, "transact" => transact::ApplyTransaction,
        9, "callee" => callee::Callee,
        10, "gas" => gas::Gas,
        11, "block_height" => block_height::BlockHeight
    }
}
