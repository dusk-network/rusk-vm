// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Source};
use dusk_abi::{ContractState, Query, ReturnValue, Transaction};
use tracing::{trace, trace_span};
use wasmer::{Exports, ImportObject, Instance, LazyInit, Module, NativeFunc};
use wasmer_middlewares::metering::set_remaining_points;

use crate::contract::ContractId;
use crate::env::Env;
use crate::gas::GasMeter;
use crate::memory::WasmerMemory;
use crate::modules::compile_module;
use crate::resolver::HostImportsResolver;
use crate::state::{Contracts, NetworkState, HOST_MODULES};
use crate::VMError;

pub struct StackFrame {
    callee: ContractId,
    ret: ReturnValue,
    memory: WasmerMemory,
    gas_meter: GasMeter,
    instance: Instance,
}

impl std::fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(return: {:?})", self.ret)
    }
}

impl StackFrame {
    fn new(
        callee: ContractId,
        memory: WasmerMemory,
        gas_meter: GasMeter,
        instance: Instance,
    ) -> StackFrame {
        StackFrame {
            callee,
            memory,
            ret: Default::default(),
            gas_meter,
            instance,
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
    block_height: u64,
}

impl<'a> CallContext<'a> {
    pub fn new(state: &'a mut NetworkState, block_height: u64) -> Self {
        CallContext {
            state,
            stack: vec![],
            block_height,
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
        let _span = trace_span!(
            "query",
            target = ?target,
            gas_limit = ?gas_meter.limit(),
            stack_index = ?self.stack.len()
        );

        let env = Env::new(self);

        let instance: Instance;

        if let Some(module) = HOST_MODULES.read().get_module(&target) {
            // is this a reserved module call?
            return Ok(module.execute(query)?);
        } else {
            let contract = self.state.get_contract(&target)?;

            let module = compile_module(
                &**contract.bytecode()?,
                self.state.get_module_config(),
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
            set_remaining_points(&instance, gas_meter.left());

            let mut memory = WasmerMemory {
                inner: LazyInit::new(),
            };
            memory.init(&instance.exports)?;
            memory.write(0, contract.state().as_bytes())?;
            memory.write(
                contract.state().as_bytes().len() as u64,
                query.as_bytes(),
            )?;

            self.stack.push(StackFrame::new(
                target,
                memory,
                gas_meter.clone(),
                instance.clone(),
            ));
        }

        let run_func: NativeFunc<i32, ()> =
            instance.exports.get_native_function("q")?;

        let r = run_func.call(0);

        match self.gas_reconciliation() {
            Ok(gas) => *gas_meter = gas,
            Err(e) => {
                gas_meter.exhaust();
                return Err(e);
            }
        }
        trace!(
            "Finished query with gas limit/spent: {}/{}",
            gas_meter.limit(),
            gas_meter.spent()
        );
        r?;

        let mut memory = WasmerMemory::new();
        memory.init(&instance.exports)?;
        let read_buffer = memory.read_from(0)?;
        let mut source = Source::new(read_buffer);
        let result = ReturnValue::decode(&mut source)?;
        self.stack.pop();
        Ok(result)
    }

    pub fn transact(
        &mut self,
        target: ContractId,
        transaction: Transaction,
        gas_meter: &'a mut GasMeter,
    ) -> Result<(ContractState, ReturnValue), VMError> {
        let _span = trace_span!(
            "transact",
            target = ?target,
            gas_limit = ?gas_meter.limit(),
            stack_index = ?self.stack.len()
        );

        let env = Env::new(self);

        let instance;

        {
            let contract = self.state.get_contract(&target)?;

            let module = compile_module(
                &**contract.bytecode()?,
                self.state.get_module_config(),
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
            set_remaining_points(&instance, gas_meter.left());

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
                target,
                memory,
                gas_meter.clone(),
                instance.clone(),
            ));
        }

        if let Ok(pre_run_func) =
            instance.exports.get_native_function::<i32, i32>("pre_t")
        {
            let pre_r = pre_run_func.call(2)?;
            assert_eq!(pre_r, 4);
        }

        let run_func: NativeFunc<i32, ()> =
            instance.exports.get_native_function("t")?;

        let r = run_func.call(0);

        match self.gas_reconciliation() {
            Ok(gas) => *gas_meter = gas,
            Err(e) => {
                gas_meter.exhaust();
                return Err(e);
            }
        }
        trace!(
            "Finished transact with gas limit/spent: {}/{}",
            gas_meter.limit(),
            gas_meter.spent()
        );
        r?;

        let ret = {
            let mut contract = self.state.get_contract_mut(&target)?;
            let mut memory = WasmerMemory::new();
            memory.init(&instance.exports)?;
            let read_buffer = memory.read_from(0)?;
            let mut source = Source::new(read_buffer);
            let state = ContractState::decode(&mut source)?;
            *(*contract).state_mut() = state;
            ReturnValue::decode(&mut source)?
        };

        let state = if self.stack.len() > 1 {
            self.stack.pop();
            self.state.get_contract(self.callee()?)?.state().clone()
        } else {
            let state =
                self.state.get_contract(self.callee()?)?.state().clone();
            self.stack.pop();
            state
        };

        Ok((state, ret))
    }

    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    pub fn gas_meter(&mut self) -> Result<&GasMeter, VMError> {
        let stack = &mut self.top_mut()?;
        let instance = &stack.instance;
        let gas_meter = &mut stack.gas_meter;

        gas_meter.update(instance, 0)?;

        Ok(&self.top()?.gas_meter)
    }

    pub fn top(&self) -> Result<&StackFrame, VMError> {
        self.stack.last().ok_or(VMError::EmptyStack)
    }

    pub fn top_mut(&mut self) -> Result<&mut StackFrame, VMError> {
        self.stack.last_mut().ok_or(VMError::EmptyStack)
    }

    pub fn callee(&self) -> Result<&ContractId, VMError> {
        Ok(&self.top()?.callee)
    }

    pub fn caller(&self) -> Result<&ContractId, VMError> {
        if self.stack.len() > 1 {
            Ok(&self.stack[self.stack.len() - 2].callee)
        } else {
            self.callee()
        }
    }

    pub fn read_memory_from(&self, offset: u64) -> Result<&[u8], VMError> {
        self.top()?.read_memory_from(offset)
    }

    pub fn read_memory(
        &self,
        offset: u64,
        length: usize,
    ) -> Result<&[u8], VMError> {
        self.top()?.read_memory(offset, length)
    }

    pub fn write_memory(
        &mut self,
        source_slice: &[u8],
        offset: u64,
    ) -> Result<(), VMError> {
        self.top_mut()?.write_memory(source_slice, offset)?;
        Ok(())
    }

    pub fn state_mut(&mut self) -> &mut Contracts {
        &mut self.state.staged
    }

    /// Reconcile the gas usage across the stack.
    fn gas_reconciliation(&mut self) -> Result<GasMeter, VMError> {
        // If there is more than one [`StackFrame`] on the stack, then the
        // gas needs to be reconciled.
        if self.stack.len() > 1 {
            let len = self.stack.len() - 2;
            let spent = self.gas_meter()?.spent();
            let parent = &mut self.stack[len];
            let parent_meter = &mut parent.gas_meter;
            let parent_instance = &parent.instance;

            // FIXME: This is a hack to make sure that the gas meter's parent
            // consumes the gas spent from its own inter-contract calls.
            // It doesn't take in account arbitrary `charge` calls to `GasMeter`
            // happened inside a contract execution, such as `host functions`,
            // that currently we don't have.
            // The API will change once we're going to work on VM2 and deciding
            // how to handle the gas consumption inside native calls.
            parent_meter.update(parent_instance, spent)?;
        }
        Ok(self.gas_meter()?.clone())
    }
}
