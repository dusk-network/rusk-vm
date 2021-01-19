// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical::{ByteSink, ByteSource, Canon, Sink, Store};
use dusk_abi::{Query, ReturnValue, Transaction};

use wasmi::{
    Externals, ImportsBuilder, MemoryRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Trap, TrapKind,
};

use crate::contract::ContractId;
use crate::gas::GasMeter;
use crate::state::NetworkState;
use crate::VMError;

pub trait Resolver<S: Store>:
    Invoke<S> + ModuleImportResolver + Clone + Default
{
}

pub use crate::resolver::CompoundResolver as StandardABI;

#[derive(Debug)]
enum Argument {
    Query(Query),
    Transaction(Transaction),
}

pub struct StackFrame {
    callee: ContractId,
    argument: Argument,
    ret: ReturnValue,
    memory: MemoryRef,
}

impl std::fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(arg: {:?} return: {:?})", self.argument, self.ret)
    }
}

impl StackFrame {
    fn new_query(callee: &ContractId, memory: MemoryRef, query: Query) -> Self {
        StackFrame {
            callee: callee.clone(),
            memory,
            argument: Argument::Query(query),
            ret: Default::default(),
        }
    }

    fn new_transaction(
        callee: &ContractId,
        memory: MemoryRef,
        transaction: Transaction,
    ) -> Self {
        StackFrame {
            callee: callee.clone(),
            memory,
            argument: Argument::Transaction(transaction),
            ret: Default::default(),
        }
    }

    fn memory<R, C: FnOnce(&[u8]) -> R>(&self, closure: C) -> R {
        self.memory.with_direct_access(closure)
    }

    fn memory_mut<R, C: FnOnce(&mut [u8]) -> R>(&self, closure: C) -> R {
        self.memory.with_direct_access_mut(closure)
    }
}

pub trait Invoke<S: Store>: Sized {
    fn invoke(
        context: &mut CallContext<Self, S>,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>>;
}

pub struct CallContext<'a, E, S: Store> {
    state: &'a mut NetworkState<E, S>,
    stack: Vec<StackFrame>,
    store: S,
    gas_meter: &'a mut GasMeter,
}

impl<'a, E, S> CallContext<'a, E, S>
where
    E: Resolver<S>,
    S: Store,
{
    pub fn new(
        state: &'a mut NetworkState<E, S>,
        gas_meter: &'a mut GasMeter,
        store: &S,
    ) -> Result<Self, VMError<S>> {
        Ok(CallContext {
            state,
            stack: vec![],
            gas_meter,
            store: store.clone(),
        })
    }

    pub fn query(
        &mut self,
        target: &ContractId,
        query: Query,
    ) -> Result<ReturnValue, VMError<S>> {
        let resolver = E::default();
        let imports = ImportsBuilder::new().with_resolver("env", &resolver);

        let instance;

        let store = self.store.clone();

        match self.state.get_contract(target)? {
            None => panic!("FIXME: error handling"),
            Some(contract) => {
                let module = wasmi::Module::from_buffer(contract.bytecode())?;

                instance = wasmi::ModuleInstance::new(&module, &imports)?
                    .assert_no_start();

                match instance.export_by_name("memory") {
                    Some(wasmi::ExternVal::Memory(memref)) => {
                        // write contract state and argument to memory
                        memref
                            .with_direct_access_mut(|m| {
                                let mut sink =
                                    ByteSink::new(&mut m[..], &store);
                                // copy the raw bytes only, since the contract can infer
                                // it's own state and argument lengths
                                sink.copy_bytes(contract.state().as_bytes());
                                sink.copy_bytes(query.as_bytes());
                                Ok(())
                            })
                            .map_err(VMError::from_store_error)?;

                        self.stack
                            .push(StackFrame::new_query(target, memref, query));
                    }
                    _ => panic!("FIXME - error handling"),
                }
            }
        }

        // Perform the query call
        instance.invoke_export("q", &[wasmi::RuntimeValue::I32(0)], self)?;

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => memref
                .with_direct_access_mut(|m| {
                    let mut source = ByteSource::new(&m[..], &store);
                    let result = Canon::<S>::read(&mut source)?;

                    self.stack.pop();
                    Ok(result)
                })
                .map_err(VMError::from_store_error),
            _ => panic!("FIXME - error handling"),
        }
    }

    pub fn transact(
        &mut self,
        target: &ContractId,
        transaction: Transaction,
    ) -> Result<ReturnValue, VMError<S>> {
        let resolver = E::default();
        let imports = ImportsBuilder::new().with_resolver("env", &resolver);

        let instance;

        let store = self.store.clone();

        match self.state.get_contract(target)? {
            None => panic!("FIXME: error handling"),
            Some(contract) => {
                let module = wasmi::Module::from_buffer(contract.bytecode())?;

                instance = wasmi::ModuleInstance::new(&module, &imports)?
                    .assert_no_start();

                match instance.export_by_name("memory") {
                    Some(wasmi::ExternVal::Memory(memref)) => {
                        // write contract state and argument to memory

                        memref.with_direct_access_mut(|m| {
                            let mut sink = ByteSink::new(&mut m[..], &store);
                            // copy the raw bytes only, since the contract can infer
                            // it's own state and argument lengths.
                            sink.copy_bytes(contract.state().as_bytes());
                            sink.copy_bytes(transaction.as_bytes());
                        });

                        self.stack.push(StackFrame::new_transaction(
                            target,
                            memref,
                            transaction,
                        ));
                    }
                    _ => panic!("FIXME - error handling"),
                }
            }
        }

        // Perform the transact call
        instance.invoke_export("t", &[wasmi::RuntimeValue::I32(0)], self)?;

        let ret = match self.state.get_contract_mut(target)? {
            None => panic!("FIXME: error handling"),
            Some(mut contract) => {
                match instance.export_by_name("memory") {
                    Some(wasmi::ExternVal::Memory(memref)) => {
                        memref
                            .with_direct_access_mut(|m| {
                                let mut source =
                                    ByteSource::new(&m[..], &store);

                                // read new state
                                let state = Canon::<S>::read(&mut source)?;

                                // update new self state
                                *(*contract).state_mut() = state;

                                // read return value
                                Canon::<S>::read(&mut source)
                            })
                            .map_err(VMError::from_store_error)
                    }
                    _ => panic!("FIXME - error handling"),
                }
            }
        };

        // finally pop the stack
        self.stack.pop();
        ret
    }

    pub fn gas_meter_mut(&mut self) -> &mut GasMeter {
        self.gas_meter
    }

    pub fn top(&self) -> &StackFrame {
        self.stack.last().expect("Invalid stack")
    }

    pub fn callee(&self) -> &ContractId {
        &self.top().callee
    }

    pub fn memory<R, C: FnOnce(&[u8]) -> R>(&self, closure: C) -> R {
        self.top().memory(closure)
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn memory_mut<R, C: FnOnce(&mut [u8]) -> Result<R, S::Error>>(
        &mut self,
        closure: C,
    ) -> Result<R, S::Error> {
        self.stack
            .last_mut()
            .expect("Invalid stack")
            .memory_mut(closure)
    }

    pub fn state(&self) -> &NetworkState<E, S> {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut NetworkState<E, S> {
        &mut self.state
    }
}

/// Convenience function to construct host traps
pub fn host_trap<S: Store>(host: VMError<S>) -> Trap {
    Trap::new(TrapKind::Host(Box::new(host)))
}

impl<'a, E, S> Externals for CallContext<'a, E, S>
where
    E: Resolver<S>,
    S: Store,
{
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match E::invoke(self, index, args) {
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
