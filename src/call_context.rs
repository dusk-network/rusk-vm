// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

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

pub struct StackFrame {
    callee: ContractId,
    memory: MemoryRef,
}

// impl std::fmt::Debug for StackFrame {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "(arg: {:?} return: {:?})", self.ret)
//     }
// }

impl StackFrame {
    fn new(callee: ContractId, memory: MemoryRef) -> Self {
        StackFrame { callee, memory }
    }

    fn memory<R, C: FnOnce(&[u8]) -> R>(&self, closure: C) -> R {
        self.memory.with_direct_access(closure)
    }

    fn memory_mut<R, C: FnOnce(&mut [u8]) -> R>(&self, closure: C) -> R {
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

pub struct CallContext<'a> {
    state: &'a mut NetworkState,
    stack: Vec<StackFrame>,
    gas_meter: &'a mut GasMeter,
    argument: &'a [u8],
}

impl<'a> CallContext<'a> {
    pub fn new(
        state: &'a mut NetworkState,
        gas_meter: &'a mut GasMeter,
        argument: &'a [u8],
    ) -> Self {
        CallContext {
            state,
            stack: vec![],
            gas_meter,
            argument,
        }
    }

    pub fn query<R>(&mut self, target: ContractId) -> Result<R, VMError> {
        let resolver = StandardABI::default();
        let imports = ImportsBuilder::new().with_resolver("env", &resolver);

        let instance;

        if let Some(module) = self.state.modules().borrow().get(&target) {
            // is this a reserved module call?
            //return module.execute(query);
            todo!()
        } else {
            let contract = self.state.get_contract(&target).expect("todo");

            let module = wasmi::Module::from_buffer(contract.bytecode())?;

            instance = wasmi::ModuleInstance::new(&module, &imports)?
                .assert_no_start();

            match instance.export_by_name("memory") {
                Some(wasmi::ExternVal::Memory(memref)) => {
                    // // write contract state and argument to memory
                    // memref.with_direct_access_mut(|m| todo!());

                    // self.stack.push(StackFrame::new_query(target, memref));
                    todo!()
                }
                _ => return Err(VMError::MemoryNotFound),
            }
        }

        // Perform the query call
        instance.invoke_export("q", &[wasmi::RuntimeValue::I32(0)], self)?;

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref.with_direct_access_mut(|m| todo!())
            }
            _ => Err(VMError::MemoryNotFound),
        }
    }

    pub fn transact<R>(&mut self, target: ContractId) -> Result<R, VMError> {
        let resolver = StandardABI::default();
        let imports = ImportsBuilder::new().with_resolver("env", &resolver);

        let instance;
        {
            let contract = self.state.get_contract(&target).expect("todo");
            let module = wasmi::Module::from_buffer(contract.bytecode())?;

            instance = wasmi::ModuleInstance::new(&module, &imports)?
                .assert_no_start();

            match instance.export_by_name("memory") {
                Some(wasmi::ExternVal::Memory(memref)) => {
                    // write contract state and argument to memory

                    memref.with_direct_access_mut(|m| {
                        // copy the raw bytes only, since the contract can
                        // infer it's own state and argument lengths.
                        todo!()
                    });

                    self.stack.push(StackFrame::new(target, memref));
                }
                _ => return Err(VMError::MemoryNotFound),
            }
        }
        // Perform the transact call
        instance.invoke_export("t", &[wasmi::RuntimeValue::I32(0)], self)?;

        let ret = {
            let mut contract =
                self.state.get_contract_mut(&target).expect("todo");

            match instance.export_by_name("memory") {
                Some(wasmi::ExternVal::Memory(memref)) => {
                    memref.with_direct_access_mut(|m| todo!())
                }
                _ => return Err(VMError::MemoryNotFound),
            }
        };

        let state = if self.stack.len() > 1 {
            self.stack.pop();
            self.state
                .get_contract(self.callee())
                .expect("todo")
                .data()
                .clone()
        } else {
            let state = self
                .state
                .get_contract(self.callee())
                .expect("todo")
                .data()
                .clone();
            self.stack.pop();
            state
        };

        Ok(ret)
    }

    pub fn gas_meter(&self) -> &GasMeter {
        self.gas_meter
    }

    pub fn gas_meter_mut(&mut self) -> &mut GasMeter {
        self.gas_meter
    }

    pub fn top(&self) -> &StackFrame {
        self.stack.last().expect("Invalid stack")
    }

    pub fn top_mut(&mut self) -> &StackFrame {
        self.stack.last_mut().expect("Invalid stack")
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

    pub fn memory_mut<R, C: FnOnce(&mut [u8]) -> R>(
        &mut self,
        closure: C,
    ) -> R {
        self.top_mut().memory_mut(closure)
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
