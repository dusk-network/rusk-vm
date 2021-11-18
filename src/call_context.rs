// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Source};
use dusk_abi::{ContractState, Query, ReturnValue, Transaction};
use wasmer::{Exports, ImportObject, Instance, LazyInit, Module, NativeFunc};

use crate::contract::ContractId;
use crate::env::Env;
use crate::gas::GasMeter;
use crate::memory::WasmerMemory;
use crate::resolver::HostImportsResolver;
use crate::state::NetworkState;
use crate::VMError;

pub struct StackFrame {
    callee: ContractId,
    ret: ReturnValue,
    memory: WasmerMemory,
    gas_meter: GasMeter,
}

impl<'a> std::fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(return: {:?})", self.ret)
    }
}

impl<'a> StackFrame {
    fn new(
        callee: ContractId,
        memory: WasmerMemory,
        gas_meter: GasMeter,
    ) -> StackFrame {
        StackFrame {
            callee,
            memory,
            ret: Default::default(),
            gas_meter,
        }
    }

    fn write_memory(
        &mut self,
        source_slice: &[u8],
        offset: u64,
    ) -> Result<(), VMError> {
        self.memory.write(offset, source_slice)
    }

    fn read_memory_from(&self, offset: u64) -> Result<&[u8], VMError> {
        self.memory.read_from(offset)
    }

    fn read_memory(
        &self,
        offset: u64,
        length: usize,
    ) -> Result<&[u8], VMError> {
        self.memory.read(offset, length)
    }
}

pub struct CallContext<'a> {
    state: &'a mut NetworkState,
    stack: Vec<StackFrame>,
}

impl<'a> CallContext<'a> {
    pub fn new(state: &'a mut NetworkState) -> Self {
        CallContext {
            state,
            stack: vec![],
        }
    }

    fn register_namespace(
        namespace_name: &str,
        env: &Env,
        module: &Module,
        import_names: &[String],
        import_object: &mut ImportObject,
    ) {
        let mut namespace = Exports::new();
        HostImportsResolver::insert_into_namespace(
            &mut namespace,
            module.store(),
            env.clone(),
            import_names,
        );
        import_object.register(namespace_name, namespace);
    }

    pub fn query(
        &mut self,
        target: ContractId,
        query: Query,
        gas_meter: &'a mut GasMeter,
    ) -> Result<ReturnValue, VMError> {
        let env = Env::new(self);

        let instance: Instance;

        if let Some(module) = self.state.modules().borrow().get(&target) {
            // is this a reserved module call?
            return module.execute(query).map_err(VMError::from_store_error);
        } else {
            let contract = self.state.get_contract(&target)?;

            let module = self
                .state
                .get_module_from_cache(&target, contract.bytecode())?;

            let import_names: Vec<String> =
                module.imports().map(|i| i.name().to_string()).collect();
            let mut import_object = ImportObject::new();
            for namespace_name in ["env", "canon"] {
                CallContext::register_namespace(
                    namespace_name,
                    &env,
                    &module,
                    &import_names,
                    &mut import_object,
                );
            }
            instance = Instance::new(&module, &import_object)?;

            let mut memory = WasmerMemory {
                inner: LazyInit::new(),
            };
            memory.init(&instance.exports)?;
            memory.write(0, contract.state().as_bytes())?;
            memory.write(
                contract.state().as_bytes().len() as u64,
                query.as_bytes(),
            )?;

            self.stack.push(StackFrame::new(target, memory, *gas_meter));
        }

        let run_func: NativeFunc<i32, ()> =
            instance.exports.get_native_function("q")?;

        let r = run_func.call(0);
        self.gas_reconciliation(gas_meter)?;
        r?;

        let mut memory = WasmerMemory::new();
        memory.init(&instance.exports)?;
        let read_buffer = memory.read_from(0)?;
        let mut source = Source::new(read_buffer);
        let result = ReturnValue::decode(&mut source)
            .map_err(VMError::from_store_error)?;
        self.stack.pop();
        Ok(result)
    }

    pub fn transact(
        &mut self,
        target_contract_id: ContractId,
        transaction: Transaction,
        gas_meter: &'a mut GasMeter,
    ) -> Result<(ContractState, ReturnValue), VMError> {
        let env = Env::new(self);

        let instance;

        {
            let contract = self.state.get_contract(&target_contract_id)?;

            let module = self.state.get_module_from_cache(
                &target_contract_id,
                contract.bytecode(),
            )?;

            let import_names: Vec<String> =
                module.imports().map(|i| i.name().to_string()).collect();
            let mut import_object = ImportObject::new();
            for namespace_name in ["env", "canon"] {
                CallContext::register_namespace(
                    namespace_name,
                    &env,
                    &module,
                    &import_names,
                    &mut import_object,
                );
            }
            instance = Instance::new(&module, &import_object)?;

            let mut memory = WasmerMemory {
                inner: LazyInit::new(),
            };
            memory.init(&instance.exports)?;
            memory.write(0, contract.state().as_bytes())?;
            memory.write(
                contract.state().as_bytes().len() as u64,
                transaction.as_bytes(),
            )?;
            self.stack.push(StackFrame::new(
                target_contract_id,
                memory,
                *gas_meter,
            ));
        }

        let run_func: NativeFunc<i32, ()> =
            instance.exports.get_native_function("t")?;
        let r = run_func.call(0);
        self.gas_reconciliation(gas_meter)?;
        r?;

        let ret = {
            let mut contract =
                self.state.get_contract_mut(&target_contract_id)?;
            let mut memory = WasmerMemory::new();
            memory.init(&instance.exports)?;
            let read_buffer = memory.read_from(0)?;
            let mut source = Source::new(read_buffer);
            let state = ContractState::decode(&mut source)
                .map_err(VMError::from_store_error)?;
            *(*contract).state_mut() = state;
            ReturnValue::decode(&mut source)
                .map_err(VMError::from_store_error)?
        };

        let state = if self.stack.len() > 1 {
            self.stack.pop();
            self.state.get_contract(self.callee())?.state().clone()
        } else {
            let state = self.state.get_contract(self.callee())?.state().clone();
            self.stack.pop();
            state
        };

        Ok((state, ret))
    }

    pub fn gas_meter(&self) -> &GasMeter {
        &self.top().gas_meter
    }

    pub fn gas_meter_mut(&mut self) -> &mut GasMeter {
        &mut self.stack.last_mut().expect("Invalid Stack").gas_meter
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

    pub fn read_memory_from(&self, offset: u64) -> Result<&[u8], VMError> {
        self.top().read_memory_from(offset)
    }

    pub fn read_memory(
        &self,
        offset: u64,
        length: usize,
    ) -> Result<&[u8], VMError> {
        self.top().read_memory(offset, length)
    }

    pub fn write_memory(
        &mut self,
        source_slice: &[u8],
        offset: u64,
    ) -> Result<(), VMError> {
        self.stack
            .last_mut()
            .expect("Invalid stack")
            .write_memory(source_slice, offset)?;
        Ok(())
    }

    pub fn state(&self) -> &NetworkState {
        self.state
    }

    pub fn state_mut(&mut self) -> &mut NetworkState {
        &mut self.state
    }

    /// Reconcile the gas usage across the stack.
    fn gas_reconciliation(&mut self, gas_meter: &mut GasMeter) -> Result<(), VMError> {
        // If there is more than one [`StackFrame`] on the stack, then the
        // gas needs to be reconciled.
        if self.stack.len() > 1 {
            let len = self.stack.len() - 2;
            let spent = self.gas_meter().spent();
            let parent_meter = &mut self.stack[len].gas_meter;

            parent_meter.charge(spent)?
        }
        *gas_meter = *self.gas_meter();

        Ok(())
    }
}
