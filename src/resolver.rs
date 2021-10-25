// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::ops::*;

use wasmer::{Function, Store, Exports, WasmerEnv};
use std::ffi::c_void;

#[derive(Clone)]
pub struct ImportReference(pub *mut c_void);
unsafe impl Send for ImportReference {}
unsafe impl Sync for ImportReference {}

#[derive(WasmerEnv, Clone)]
pub struct Env {
    pub context: ImportReference
}

pub struct HostImportsResolver;

impl HostImportsResolver {
    // pub fn insert_into_namespace(namespace: &mut Exports, store: &Store, env: Env) {
    //     namespace.insert("sig", Function::new_native_with_env(&store, env.clone(), panic::Panic::panic));
    //     namespace.insert("debug", Function::new_native_with_env(&store, env.clone(), debug::Debug::debug));
    //     namespace.insert("block_height", Function::new_native_with_env(&store, env.clone(), block_height::BlockHeight::block_height));
    //     namespace.insert("transact", Function::new_native_with_env(&store, env.clone(), transact::ApplyTransaction::transact));
    //     namespace.insert("query", Function::new_native_with_env(&store, env.clone(), query::ExecuteQuery::query));
    //     namespace.insert("callee", Function::new_native_with_env(&store, env.clone(), call_stack::Callee::callee));
    //     namespace.insert("caller", Function::new_native_with_env(&store, env.clone(), call_stack::Caller::caller));
    //     namespace.insert("get", Function::new_native_with_env(&store, env.clone(), store::Get::get));
    //     namespace.insert("put", Function::new_native_with_env(&store, env.clone(), store::Put::put));
    //     namespace.insert("hash", Function::new_native_with_env(&store, env.clone(), store::Hash::hash));
    //     namespace.insert("gas", Function::new_native_with_env(&store, env.clone(), gas::Gas::gas));
    //     namespace.insert("gas_consumed", Function::new_native_with_env(&store, env.clone(), gas::GasConsumed::gas_consumed));
    //     namespace.insert("gas_left", Function::new_native_with_env(&store, env.clone(), gas::GasLeft::gas_left));
    // }
    pub fn insert_into_namespace(namespace: &mut Exports, store: &Store, env: Env, names: &Vec<String>) {
        if names.contains(&"sig".to_string()) {
            println!("inserting sig");
            namespace.insert("sig", Function::new_native_with_env(&store, env.clone(), panic::Panic::panic));
        };
        if names.contains(&"debug".to_string()) {
            println!("inserting debug");
            namespace.insert("debug", Function::new_native_with_env(&store, env.clone(), debug::Debug::debug));
        };
        if names.contains(&"block_height".to_string()) {
            println!("inserting block_height");
            namespace.insert("block_height", Function::new_native_with_env(&store, env.clone(), block_height::BlockHeight::block_height));
        };
        if names.contains(&"transact".to_string()) {
            println!("inserting transact");
            namespace.insert("transact", Function::new_native_with_env(&store, env.clone(), transact::ApplyTransaction::transact));
        };
        if names.contains(&"query".to_string()) {
            println!("inserting query");
            namespace.insert("query", Function::new_native_with_env(&store, env.clone(), query::ExecuteQuery::query));
        };
        if names.contains(&"callee".to_string()) {
            println!("inserting callee");
            namespace.insert("callee", Function::new_native_with_env(&store, env.clone(), call_stack::Callee::callee));
        };
        if names.contains(&"caller".to_string()) {
            println!("inserting caller");
            namespace.insert("caller", Function::new_native_with_env(&store, env.clone(), call_stack::Caller::caller));
        };
        if names.contains(&"get".to_string()) {
            println!("inserting get");
            namespace.insert("get", Function::new_native_with_env(&store, env.clone(), store::Get::get));
        };
        if names.contains(&"put".to_string()) {
            println!("inserting put");
            namespace.insert("put", Function::new_native_with_env(&store, env.clone(), store::Put::put));
        };
        if names.contains(&"hash".to_string()) {
            println!("inserting hash");
            namespace.insert("hash", Function::new_native_with_env(&store, env.clone(), store::Hash::hash));
        };
        if names.contains(&"gas".to_string()) {
            println!("inserting gas");
            namespace.insert("gas", Function::new_native_with_env(&store, env.clone(), gas::Gas::gas));
        };
        if names.contains(&"gas_consumed".to_string()) {
            println!("inserting gas_consumed");
            namespace.insert("gas_consumed", Function::new_native_with_env(&store, env.clone(), gas::GasConsumed::gas_consumed));
        };
        if names.contains(&"gas_left".to_string()) {
            println!("inserting gas_left");
            namespace.insert("gas_left", Function::new_native_with_env(&store, env.clone(), gas::GasLeft::gas_left));
        };
    }
}
