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

use wasmer::{imports, wat2wasm, Function, Instance, Module, NativeFunc, Store, ImportObject, Exports};
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

pub struct StackFrame<'a> {
    callee: ContractId,
    argument: Argument,
    ret: ReturnValue,
    memory: WasmerMemoryRef<'a>
}

impl std::fmt::Debug for StackFrame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(arg: {:?} return: {:?})", self.argument, self.ret)
    }
}

impl StackFrame<'_> {
    fn new_query<'a>(callee: ContractId, memory: WasmerMemoryRef<'a>, query: Query) -> StackFrame<'a> {
        StackFrame {
            callee,
            memory,
            argument: Argument::Query(query),
            ret: Default::default(),
        }
    }

    fn new_transaction<'a>(
        callee: ContractId,
        memory: WasmerMemoryRef<'a>,
        transaction: Transaction,
    ) -> StackFrame<'a> {
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

pub struct WasmerMemoryRef<'a> {
    memory_ref: &'a wasmer::Memory,
}

impl WasmerMemoryRef<'_> {
    pub fn new(memory_ref: &wasmer::Memory) -> WasmerMemoryRef {
        WasmerMemoryRef { memory_ref }
    }
    pub fn with_direct_access<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        let buf = unsafe { self.memory_ref.data_unchecked() };
        f(buf)
    }

    pub fn with_direct_access_mut<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> R {
        let buf = unsafe { self.memory_ref.data_unchecked_mut() };
        f(buf)
    }

}

pub struct CallContext<'a> {
    state: &'a mut NetworkState,
    stack: Vec<StackFrame<'a>>,
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

            let mut wasmer_memref = WasmerMemoryRef::new(wasmer_instance.exports.get_memory("memory").expect("wasmer memory exported"));
            wasmer_memref
                // .with_direct_access_mut::<Result<(), VMError>, FnOnce(&mut [u8]) ->Result<(), VMError>>(|m| {
                .with_direct_access_mut::<Result<(), VMError>, _>(|m| {
                    let mut sink = Sink::new(&mut *m);
                    // copy the raw bytes only, since the
                    // contract can infer it's own state and argument lengths
                    sink.copy_bytes(contract.state().as_bytes());
                    sink.copy_bytes(query.as_bytes());
                    Ok(())
                });

            self.stack // todo
                .push(StackFrame::new_query(target, wasmer_memref, query));
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
        let mut wasmer_memref = WasmerMemoryRef::new(wasmer_instance.exports.get_memory("memory").expect("wasmer memory exported"));
        let ret = wasmer_memref.with_direct_access_mut(|m| {
            let mut source = Source::new(&m[..]);
            let result = ReturnValue::decode(&mut source).expect("query result decoded");

            self.stack.pop();
            Ok(result)
        });
        ret
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
            let mut wasmer_memref = WasmerMemoryRef::new(wasmer_instance.exports.get_memory("memory").expect("wasmer memory exported"));
            wasmer_memref.with_direct_access_mut(|m| {
                let mut sink = Sink::new(&mut *m);
                // copy the raw bytes only, since the contract can
                // infer it's own state and argument lengths.
                sink.copy_bytes(contract.state().as_bytes());
                sink.copy_bytes(transaction.as_bytes());
            });
            self.stack.push(StackFrame::new_transaction( // todo
                target,
                wasmer_memref,
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
            let mut wasmer_memref = WasmerMemoryRef::new(wasmer_instance.exports.get_memory("memory").expect("wasmer memory exported"));
            wasmer_memref.with_direct_access_mut(|m| {
                let mut source = Source::new(&m[..]);

                // read new state
                let state = ContractState::decode(&mut source).expect("contract state decoded");

                // update new self state
                *(*contract).state_mut() = state;

                // read return value
                ReturnValue::decode(&mut source)
            })

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
