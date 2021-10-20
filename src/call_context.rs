// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Source};
use dusk_abi::{ContractState, Query, ReturnValue, Transaction};

use crate::contract::ContractId;
use crate::gas::GasMeter;
use crate::state::NetworkState;
use crate::VMError;

use crate::resolver::{Env, ImportReference, HostImportsResolver};
use crate::memory::WasmerMemory;

use wasmer::{Instance, Module, NativeFunc, Store, ImportObject, Exports, LazyInit};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

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

    // fn memory<R, C: FnOnce(&[u8]) -> R>(&self, closure: C) -> R {
    //     self.memory.with_direct_access(closure)
    // }
    //
    // fn memory_mut<R, C: FnOnce(&mut [u8]) -> R>(&mut self, closure: C) -> R {
    //     self.memory.with_direct_access_mut(closure)
    // }

    fn write_memory(&mut self, source_slice: &[u8], offset: u64) -> Result<(), VMError>{
        unsafe { WasmerMemory::write_memory_bytes(self.memory.inner.get_unchecked(), offset, source_slice) };
        Ok(())
    }

    fn read_memory(&self) -> Result<Vec<u8>, VMError> {
        unsafe { WasmerMemory::read_memory_bytes(self.memory.inner.get_unchecked(), 0, self.memory.inner.get_unchecked().data_size() as usize) }
    }
}

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
            unsafe { WasmerMemory::write_memory_bytes(wasmer_memory.inner.get_unchecked(), 0, contract.state().as_bytes()) };
            unsafe { WasmerMemory::write_memory_bytes(wasmer_memory.inner.get_unchecked(), contract.state().as_bytes().len() as u64, query.as_bytes()) };

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
        let read_buffer = unsafe { WasmerMemory::read_memory_bytes(wasmer_memory.inner.get_unchecked(), 0, wasmer_memory.inner.get_unchecked().data_size() as usize)? };
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
            unsafe { WasmerMemory::write_memory_bytes(wasmer_memory.inner.get_unchecked(), 0, contract.state().as_bytes()) };
            unsafe { WasmerMemory::write_memory_bytes(wasmer_memory.inner.get_unchecked(), contract.state().as_bytes().len() as u64, transaction.as_bytes()) };

            self.stack.push(StackFrame::new_transaction(
                target,
                wasmer_memory,
                transaction,
            ));
        }
        // Perform the transact call
        // instance.invoke_export("t", &[wasmi::RuntimeValue::I32(0)], self)?;

        // WASMER
        let wasmer_run_func: NativeFunc<i32, ()> = wasmer_instance.exports.get_native_function("t").expect("wasmer invoked function t");
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
            let read_buffer = unsafe { WasmerMemory::read_memory_bytes(wasmer_memory.inner.get_unchecked(), 0, wasmer_memory.inner.get_unchecked().data_size() as usize)? };
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

    pub fn read_memory(&self) -> Result<Vec<u8>, VMError> {
        self.top().read_memory()
    }

    pub fn write_memory(&mut self, source_slice: &[u8], offset: u64) -> Result<(), VMError> {
        self.stack
            .last_mut()
            .expect("Invalid stack")
            .write_memory(source_slice, offset);
        Ok(())
    }

    pub fn state(&self) -> &NetworkState {
        self.state
    }

    pub fn state_mut(&mut self) -> &mut NetworkState {
        &mut self.state
    }
}

// Convenience function to construct host traps
// pub fn host_trap(host: VMError) -> Trap {
//     Trap::new(TrapKind::Host(Box::new(host)))
// }

