// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::Resolver;
use crate::ops::*;
use crate::VMError;
use std::collections::HashMap;
use std::sync::Arc;
use microkelvin::PersistedId;

use wasmi::{
    self, FuncInstance, FuncRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Signature,
};

use crate::call_context::{CallContext, Invoke};

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

use canonical::{Canon, CanonError, Id};

#[derive(Clone)]
pub struct MyPersistedId(Id);

#[derive(WasmerEnv, Clone)]
pub struct Env {
    pub persisted_id: MyPersistedId
}

impl HostImportsResolver {
    // pub fn register(&mut self, name: &str, store: &Store) {
    //     let f = Function::new_native(store, panic::Panic::panic);
    //     self.imports.insert(name, f);
    // }
    pub fn insert_into_namespace(namespace: &mut Exports, store: &Store, persisted_id: MyPersistedId, name: &str) {
        let env = Env{ persisted_id };
        let fun = panic::Panic::panic;
        fn say_hello_world(env: &Env) {
            println!("Hello, world!")
        }
        namespace.insert(name, Function::new_native_with_env(&store, env, say_hello_world))
    }
}
