// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, CanonError, Sink, Source};
use dusk_abi::{ContractState, Query, ReturnValue, Transaction};

use wasmi::{
    Externals, ImportsBuilder, MemoryRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Trap, TrapKind,
};

use crate::contract::ContractId;
use crate::gas::GasMeter;
use crate::state::NetworkState;
use crate::VMError;

pub trait Resolver: Invoke + ModuleImportResolver + Clone + Default {}

pub use crate::resolver::CompoundResolver as StandardABI;
use crate::resolver::{HostImportsResolver, Env, ImportReference};

use wasmer::{imports, wat2wasm, Function, Instance, Module, NativeFunc, Store, ImportObject, Exports, Memory, LazyInit};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

use microkelvin::{
    BackendCtor, Compound, DiskBackend, Keyed, PersistError, Persistence, PersistedId
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc,Mutex};
use std::ffi::c_void;



#[derive(Debug)]
enum Argument {
    Query(Query),
    Transaction(Transaction),
}

pub struct StackFrame {
    callee: ContractId,
    argument: Argument,
    ret: ReturnValue,
    memory: WasmerMemory,
}

impl std::fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(arg: {:?} return: {:?})", self.argument, self.ret)
    }
}

impl StackFrame {
    fn new_query(callee: ContractId, memory: WasmerMemory, query: Query) -> StackFrame {
        StackFrame {
            callee,
            memory,
            argument: Argument::Query(query),
            ret: Default::default(),
        }
    }

    fn new_transaction(
        callee: ContractId,
        memory: WasmerMemory,
        transaction: Transaction,
    ) -> StackFrame {
        StackFrame {
            callee,
            memory,
            argument: Argument::Transaction(transaction),
            ret: Default::default(),
        }
    }

    fn memory<R, C: FnOnce(&[u8]) -> R>(&self, closure: C) -> R {
        self.memory.with_direct_access(closure)
    }

    fn memory_mut<R, C: FnOnce(&mut [u8]) -> R>(&mut self, closure: C) -> R {
        self.memory.with_direct_access_mut(closure)
    }
}

pub trait Invoke: Sized {
    fn invoke(
        context: &mut CallContext,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError>;
}

/// Check that the given offset and length fits into the memory bounds. If not,
/// it will try to grow the memory.
fn check_bounds(memory: &Memory, offset: u64, len: usize) -> Result<(), VMError> {
    if memory.data_size() < offset + len as u64 {
        let cur_pages = memory.size().0;
        let capacity = cur_pages as usize * wasmer::WASM_PAGE_SIZE;
        let missing = offset as usize + len - capacity;
        // Ceiling division
        let req_pages = ((missing + wasmer::WASM_PAGE_SIZE - 1)
            / wasmer::WASM_PAGE_SIZE) as u32;
        memory.grow(req_pages).map_err(VMError::MemoryNotFound)?; // todo this grow will probably need to go away
    }
    Ok(())
}

/// Read bytes from memory at the given offset and length
fn read_memory_bytes(
    memory: &Memory,
    offset: u64,
    len: usize,
) -> Result<Vec<u8>, VMError> {
    check_bounds(memory, offset, len)?;
    let offset = offset as usize;
    let vec: Vec<_> = memory.view()[offset..(offset + len)]
        .iter()
        .map(|cell| cell.get())
        .collect();
    Ok(vec)
}

/// Write bytes into memory at the given offset
fn write_memory_bytes(
    memory: &Memory,
    offset: u64,
    bytes: impl AsRef<[u8]>,
) -> Result<(), VMError> {
    let slice = bytes.as_ref();
    let len = slice.len();
    check_bounds(memory, offset, len as _)?;
    let offset = offset as usize;
    memory.view()[offset..(offset + len)]
        .iter()
        .zip(slice.iter())
        .for_each(|(cell, v)| cell.set(*v));
    Ok(())
}

pub struct WasmerMemory {
    inner: LazyInit<Memory>,
}

impl WasmerMemory {
    pub fn init_env_memory(
        &mut self,
        exports: &wasmer::Exports,
    ) -> std::result::Result<(), VMError> {
        let memory = exports.get_memory("memory")?;
        self.inner.initialize(memory.clone());
        Ok(())
    }
}


// impl WasmerMemory {
//     pub fn new(memory_ref: &wasmer::Memory) -> WasmerMemoryRef {
//         WasmerMemoryRef { memory_ref }
//     }
//     pub fn with_direct_access<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
//         let buf = unsafe { self.memory_ref.data_unchecked() };
//         f(buf)
//     }
//
//     pub fn with_direct_access_mut<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> R {
//         let buf = unsafe { self.memory_ref.data_unchecked_mut() };
//         f(buf)
//     }
//
// }

pub struct CallContext<'a> {
    state: &'a mut NetworkState,
    stack: Vec<StackFrame>,
    gas_meter: &'a mut GasMeter,
}

impl<'a> CallContext<'a> {
    pub fn new(
        state: &'a mut NetworkState,
        gas_meter: &'a mut GasMeter,
    ) -> Self {
        CallContext {
            state,
            stack: vec![],
            gas_meter,
        }
    }

    pub fn create_env(&mut self) -> Env {
        Env {
            context: ImportReference(self as *mut _ as *mut c_void)
        }
    }

    pub fn query(
        &mut self,
        target: ContractId,
        query: Query,
    ) -> Result<ReturnValue, VMError> {

        //let instance;
        let wasmer_instance: Instance;

        if let Some(module) = self.state.modules().borrow().get(&target) {
             // is this a reserved module call?
             return module.execute(query).map_err(VMError::from_store_error);
        } else {
            let contract = self.state.get_contract(&target)?;

            //let module = wasmi::Module::from_buffer(contract.bytecode())?;
            // WASMER
            let wasmer_store = Store::new(&Universal::new(Cranelift::default()).engine());
            let wasmer_module = Module::new(&wasmer_store, contract.bytecode()).expect("wasmer module created");


            //instance = wasmi::ModuleInstance::new(&module, &imports)?.assert_no_start();


            // WASMER
            let wasmer_import_names: Vec<String> = wasmer_module.imports().map(|i| i.name().to_string()).collect();
            println!("import names for contract id {:?} = {:?}", target, wasmer_import_names);
            let mut wasmer_import_object = ImportObject::new();
            // WASMER env namespace
            let mut env_namespace = Exports::new();

            HostImportsResolver::insert_into_namespace(&mut env_namespace, &wasmer_store, self.create_env());
            wasmer_import_object.register("env", env_namespace);

            // match instance.export_by_name("memory") {
            //     Some(wasmi::ExternVal::Memory(memref)) => {
            //         // write contract state and argument to memory
            //         memref
            //             .with_direct_access_mut(|m| {
            //                 let mut sink = Sink::new(&mut *m);
            //                 // copy the raw bytes only, since the
            //                 // contract
            //                 // can infer
            //                 // it's own state and argument lengths
            //                 sink.copy_bytes(contract.state().as_bytes());
            //                 sink.copy_bytes(query.as_bytes());
            //                 Ok(())
            //             })
            //             .map_err(VMError::from_store_error)?;
            //
            //         self.stack
            //             .push(StackFrame::new_query(target, memref, query));
            //     }
            //     _ => return Err(VMError::MemoryNotFound),
            // }

            // WASMER
            wasmer_instance = Instance::new(&wasmer_module, &wasmer_import_object).expect("wasmer module created");

            let mut wasmer_memory = WasmerMemory { inner: LazyInit::new() };
            wasmer_memory.init_env_memory(&wasmer_instance.exports)?;
            unsafe { write_memory_bytes(wasmer_memory.inner.get_unchecked(), 0, contract.state().as_bytes()) };
            unsafe { write_memory_bytes(wasmer_memory.inner.get_unchecked(), contract.state().as_bytes().len() as u64, query.as_bytes()) };

            self.stack
                .push(StackFrame::new_query(target, wasmer_memory, query));
        }

        // Perform the query call
        //instance.invoke_export("q", &[wasmi::RuntimeValue::I32(0)], self)?;

        // WASMER
        let wasmer_run_func: NativeFunc<i32, ()> = wasmer_instance.exports.get_native_function("q").expect("wasmer invoked function q");
        wasmer_run_func.call(0);

        // match instance.export_by_name("memory") {
        //     Some(wasmi::ExternVal::Memory(memref)) => memref
        //         .with_direct_access_mut(|m| {
        //             let mut source = Source::new(&m[..]);
        //             let result = ReturnValue::decode(&mut source)?;
        //
        //             self.stack.pop();
        //             Ok(result)
        //         })
        //         .map_err(VMError::from_store_error),
        //     _ => Err(VMError::MemoryNotFound),
        // }

        // WASMER
        let mut wasmer_memory = WasmerMemory { inner: LazyInit::new() };
        wasmer_memory.init_env_memory(&wasmer_instance.exports)?;
        let read_buffer = unsafe { read_memory_bytes(wasmer_memory.inner.get_unchecked(), 0, wasmer_memory.inner.get_unchecked().data_size() as usize)? };
        let mut source = Source::new(&read_buffer);
        let result = ReturnValue::decode(&mut source).expect("query result decoded");
        self.stack.pop();
        Ok(result)
    }

    pub fn transact(
        &mut self,
        target: ContractId,
        transaction: Transaction,
    ) -> Result<(ContractState, ReturnValue), VMError> {

        //let instance;
        let wasmer_instance;

        {
            let contract = self.state.get_contract(&target)?;
            // let module = wasmi::Module::from_buffer(contract.bytecode())?;

            // WASMER
            let wasmer_store = Store::new(&Universal::new(Cranelift::default()).engine());
            let wasmer_module = Module::new(&wasmer_store, contract.bytecode()).expect("wasmer module created");



            // instance = wasmi::ModuleInstance::new(&module, &imports)?
            //     .assert_no_start();
            // WASMER
            let wasmer_import_names: Vec<String> = wasmer_module.imports().map(|i| i.name().to_string()).collect();
            println!("import names for contract id {:?} = {:?}", target, wasmer_import_names);
            let mut wasmer_import_object = ImportObject::new();
            // WASMER env namespace
            let mut env_namespace = Exports::new();
            HostImportsResolver::insert_into_namespace(&mut env_namespace, &wasmer_store, self.create_env());

            wasmer_import_object.register("env", env_namespace);

            // WASMER
            wasmer_instance = Instance::new(&wasmer_module, &wasmer_import_object).expect("wasmer module created");


            // match instance.export_by_name("memory") {
            //     Some(wasmi::ExternVal::Memory(memref)) => {
            //         // write contract state and argument to memory
            //
            //         memref.with_direct_access_mut(|m| {
            //             let mut sink = Sink::new(&mut *m);
            //             // copy the raw bytes only, since the contract can
            //             // infer it's own state and argument lengths.
            //             sink.copy_bytes(contract.state().as_bytes());
            //             sink.copy_bytes(transaction.as_bytes());
            //         });
            //
            //         self.stack.push(StackFrame::new_transaction(
            //             target,
            //             memref,
            //             transaction,
            //         ));
            //     }
            //     _ => return Err(VMError::MemoryNotFound),
            // }

            // WASMER
            let mut wasmer_memory = WasmerMemory { inner: LazyInit::new() };
            wasmer_memory.init_env_memory(&wasmer_instance.exports)?;
            unsafe { write_memory_bytes(wasmer_memory.inner.get_unchecked(), 0, contract.state().as_bytes()) };
            unsafe { write_memory_bytes(wasmer_memory.inner.get_unchecked(), contract.state().as_bytes().len() as u64, transaction.as_bytes()) };

            self.stack.push(StackFrame::new_transaction(
                target,
                wasmer_memory,
                transaction,
            ));
        }
        // Perform the transact call
        // instance.invoke_export("t", &[wasmi::RuntimeValue::I32(0)], self)?;

        // WASMER
        let wasmer_run_func: NativeFunc<(i32), ()> = wasmer_instance.exports.get_native_function("t").expect("wasmer invoked function t");
        wasmer_run_func.call(0);


        let ret = {
            let mut contract = self.state.get_contract_mut(&target)?;

            // match instance.export_by_name("memory") {
            //     Some(wasmi::ExternVal::Memory(memref)) => {
            //         memref
            //             .with_direct_access_mut(|m| {
            //                 let mut source = Source::new(&m[..]);
            //
            //                 // read new state
            //                 let state = ContractState::decode(&mut source)?;
            //
            //                 // update new self state
            //                 *(*contract).state_mut() = state;
            //
            //                 // read return value
            //                 ReturnValue::decode(&mut source)
            //             })
            //             .map_err(VMError::from_store_error)
            //     }
            //     _ => return Err(VMError::MemoryNotFound),
            // }

            // WASMER
            let mut wasmer_memory = WasmerMemory { inner: LazyInit::new() };
            wasmer_memory.init_env_memory(&wasmer_instance.exports)?;
            let read_buffer = unsafe { read_memory_bytes(wasmer_memory.inner.get_unchecked(), 0, wasmer_memory.inner.get_unchecked().data_size() as usize)? };
            let mut source = Source::new(&read_buffer);
            let state = ContractState::decode(&mut source).expect("query result decoded");
            *(*contract).state_mut() = state;
            ReturnValue::decode(&mut source)
        };

        let state = if self.stack.len() > 1 {
            self.stack.pop();
            self.state.get_contract(self.callee())?.state().clone()
        } else {
            let state = self.state.get_contract(self.callee())?.state().clone();
            self.stack.pop();
            state
        };

        Ok((state, ret.expect("converted error")))
    }

    pub fn gas_meter(&self) -> GasMeter {
        self.gas_meter()
    }

    pub fn gas_meter_mut(&mut self) -> GasMeter {
        self.gas_meter()
    }

    pub fn top(&self) -> &StackFrame {
        self.stack.last().expect("Invalid stack")
    }

    pub fn callee(&self) -> &ContractId {
        &self.top().callee
    }

    pub fn caller(&self) -> &ContractId {
        if self.stack.len() > 1 {
            &self.stack[self.stack.len() - 2].callee
        } else {
            self.callee()
        }
    }

    pub fn memory<R, C: FnOnce(&[u8]) -> R>(&self, closure: C) -> R {
        self.top().memory(closure)
    }

    pub fn memory_mut<R, C: FnOnce(&mut [u8]) -> Result<R, CanonError>>(
        &mut self,
        closure: C,
    ) -> Result<R, CanonError> {
        self.stack
            .last_mut()
            .expect("Invalid stack")
            .memory_mut(closure)
    }

    pub fn state(&self) -> &NetworkState {
        self.state
    }

    pub fn state_mut(&mut self) -> &mut NetworkState {
        &mut self.state
    }
}

/// Convenience function to construct host traps
pub fn host_trap(host: VMError) -> Trap {
    Trap::new(TrapKind::Host(Box::new(host)))
}

impl<'a> Externals for CallContext<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match StandardABI::invoke(self, index, args) {
            Ok(ok) => Ok(ok),
            Err(e) => {
                if let VMError::Trap(t) = e {
                    Err(t)
                } else {
                    Err(host_trap(e))
                }
            }
        }
    }
}
