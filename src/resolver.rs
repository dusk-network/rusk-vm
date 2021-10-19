// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::Resolver;
use crate::ops::*;
use crate::{GasMeter, VMError};
use microkelvin::PersistedId;

use wasmi::{
    self, FuncInstance, FuncRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Signature,
};

use crate::call_context::{CallContext, Invoke, StackFrame};

use wasmer::{Function, Store, Exports, WasmerEnv};

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
        4, "hash" => store::Hash,
        6, "query" => query::ExecuteQuery,
        7, "transact" => transact::ApplyTransaction,
        9, "callee" => call_stack::Callee,
        10, "caller" => call_stack::Caller,
        11, "gas" => gas::Gas,
        12, "gas_consumed" => gas::GasConsumed,
        13, "gas_left" => gas::GasLeft,
        14, "block_height" => block_height::BlockHeight
    }
}

pub struct HostImportsResolver {
    // imports: HashMap<&'static str, Function>
}

// impl HostImportsResolver {
//     pub fn new() -> Self {
//         let imports: HashMap<&'static str, Box<dyn AbiCall>> = [
//             ("sig", panic::Panic),
//             ("debug", debug::Debug),
//             ("get", store::Get),
//             ("put", store::Put),
//             ("hash", store::Hash),
//             ("query", query::ExecuteQuery),
//             ("transact", transact::ApplyTransaction),
//             ("callee", call_stack::Callee),
//             ("caller", call_stack::Caller),
//             ("gas", gas::Gas),
//             ("gas_consumed", gas::GasConsumed),
//             ("gas_left", gas::GasLeft),
//             ("block_height", block_height::BlockHeight)
//         ].iter().cloned().collect();
//         HostImportsResolver{ imports }
//     }
//     pub fn resolve(&self, name: &str) -> Option<Box<dyn AbiCall>> {
//         self.imports.get(name).map(|p| *p)
//     }
// }

use std::ffi::c_void;

#[derive(Clone)]
pub struct ImportReference(pub *mut c_void);
unsafe impl Send for ImportReference {}
unsafe impl Sync for ImportReference {}


#[derive(WasmerEnv, Clone)]
pub struct Env {
    pub context: ImportReference
}

impl HostImportsResolver {
    pub fn insert_into_namespace(namespace: &mut Exports, store: &Store, env: Env) {
        namespace.insert("sig", Function::new_native_with_env(&store, env.clone(), panic::Panic::panic));
        namespace.insert("debug", Function::new_native_with_env(&store, env.clone(), debug::Debug::debug));
        namespace.insert("block_height", Function::new_native_with_env(&store, env.clone(), block_height::BlockHeight::block_height));
        namespace.insert("transact", Function::new_native_with_env(&store, env.clone(), transact::ApplyTransaction::transact));
        namespace.insert("query", Function::new_native_with_env(&store, env.clone(), query::ExecuteQuery::query));
        namespace.insert("callee", Function::new_native_with_env(&store, env.clone(), call_stack::Callee::callee));
        namespace.insert("caller", Function::new_native_with_env(&store, env.clone(), call_stack::Caller::caller));
        namespace.insert("get", Function::new_native_with_env(&store, env.clone(), store::Get::get));
        namespace.insert("put", Function::new_native_with_env(&store, env.clone(), store::Put::put));
        namespace.insert("hash", Function::new_native_with_env(&store, env.clone(), store::Hash::hash));
        namespace.insert("gas", Function::new_native_with_env(&store, env.clone(), gas::Gas::gas));
        namespace.insert("gas_consumed", Function::new_native_with_env(&store, env.clone(), gas::GasConsumed::gas_consumed));
        namespace.insert("gas_left", Function::new_native_with_env(&store, env.clone(), gas::GasLeft::gas_left));
    }
}
