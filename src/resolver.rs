// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::env::Env;
use crate::ops::*;

use wasmer::{Exports, Function, Store};

pub struct HostImportsResolver;

impl HostImportsResolver {
    pub fn insert_into_namespace(
        namespace: &mut Exports,
        store: &Store,
        env: Env,
        names: &[String],
    ) {
        for name in names {
            match name.as_str() {
                "sig" => namespace.insert(
                    "sig",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        panic::Panic::panic,
                    ),
                ),
                "debug" => namespace.insert(
                    "debug",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        debug::Debug::debug,
                    ),
                ),
                "block_height" => namespace.insert(
                    "block_height",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        block_height::BlockHeight::block_height,
                    ),
                ),
                "transact" => namespace.insert(
                    "transact",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        transact::ApplyTransaction::transact,
                    ),
                ),
                "query" => namespace.insert(
                    "query",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        query::ExecuteQuery::query,
                    ),
                ),
                "emit" => namespace.insert(
                    "emit",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        emit::Emit::emit,
                    ),
                ),
                "callee" => namespace.insert(
                    "callee",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        call_stack::Callee::callee,
                    ),
                ),
                "caller" => namespace.insert(
                    "caller",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        call_stack::Caller::caller,
                    ),
                ),
                "_get" => namespace.insert(
                    "_get",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        store::Get::get,
                    ),
                ),
                "_put" => namespace.insert(
                    "_put",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        store::Put::put,
                    ),
                ),
                "hash" => namespace.insert(
                    "hash",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        store::Hash::hash,
                    ),
                ),
                "gas_consumed" => namespace.insert(
                    "gas_consumed",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        gas::GasConsumed::gas_consumed,
                    ),
                ),
                "gas_left" => namespace.insert(
                    "gas_left",
                    Function::new_native_with_env(
                        store,
                        env.clone(),
                        gas::GasLeft::gas_left,
                    ),
                ),
                _ => {
                    debug_assert!(false, "unknown wasm module import {}", name)
                }
            }
        }
    }
}
