// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Source};
use dusk_abi::{ContractState, Query, ReturnValue, Transaction};
use tracing::{trace, trace_span};
use wasmer::{Exports, ImportObject, Instance, LazyInit, Module, NativeFunc};
use wasmer_middlewares::metering::{
    get_remaining_points, set_remaining_points, MeteringPoints,
};

use crate::contract::ContractId;
use crate::env::Env;
use crate::gas::GasMeter;
use crate::memory::WasmerMemory;
use crate::modules::{compile_module, HostModules, ModuleConfig};
use crate::resolver::HostImportsResolver;
use crate::state::Contracts;
use crate::VMError;

pub struct StackFrame {
    callee: ContractId,
    ret: ReturnValue,
    memory: WasmerMemory,
    gas_meter: GasMeter,
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
    contracts: &'a mut Contracts,
    config: ModuleConfig,
    modules: HostModules,
    stack: Vec<StackFrame>,
    block_height: u64,
}

impl<'a> CallContext<'a> {
    pub fn new(
        contracts: &'a mut Contracts,
        config: ModuleConfig,
        modules: HostModules,
        block_height: u64,
    ) -> Self {
        CallContext {
            contracts,
            config,
            modules,
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

        if let Some(module) = self.modules.get_module_ref(&target).get() {
            // is this a reserved module call?
            return module.execute(query).map_err(VMError::from_store_error);
        } else {
            let contract = self.contracts.get_contract(&target)?;

            let module = compile_module(contract.bytecode(), &self.config)?;

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

            self.stack
                .push(StackFrame::new(target, memory, gas_meter.clone()));
        }

        let run_func: NativeFunc<i32, ()> =
            instance.exports.get_native_function("q")?;

        let r = run_func.call(0);
        match self.gas_reconciliation(&instance) {
            Ok(gas) => *gas_meter = gas,
            Err(e) => {
                gas_meter.set_left(0);
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
        let result = ReturnValue::decode(&mut source)
            .map_err(VMError::from_store_error)?;
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
            let contract = self.contracts.get_contract(&target)?;

            let module = compile_module(contract.bytecode(), &self.config)?;

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
            self.stack
                .push(StackFrame::new(target, memory, gas_meter.clone()));
        }

        let run_func: NativeFunc<i32, ()> =
            instance.exports.get_native_function("t")?;
        let r = run_func.call(0);
        match self.gas_reconciliation(&instance) {
            Ok(gas) => *gas_meter = gas,
            Err(e) => {
                gas_meter.set_left(0);
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
            let mut contract = self.contracts.get_contract_mut(&target)?;
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
            self.contracts.get_contract(self.callee())?.state().clone()
        } else {
            let state =
                self.contracts.get_contract(self.callee())?.state().clone();
            self.stack.pop();
            state
        };

        Ok((state, ret))
    }

    pub fn block_height(&self) -> u64 {
        self.block_height
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

    pub fn state_mut(&mut self) -> &mut Contracts {
        self.contracts
    }

    /// Reconcile the gas usage across the stack.
    fn gas_reconciliation(
        &mut self,
        instance: &Instance,
    ) -> Result<GasMeter, VMError> {
        let remaining = get_remaining_points(instance);
        match remaining {
            MeteringPoints::Remaining(r) => {
                self.gas_meter_mut().set_left(r as u64)
            }
            MeteringPoints::Exhausted => {
                self.gas_meter_mut().set_left(0);
                return Err(VMError::OutOfGas);
            }
        }

        // If there is more than one [`StackFrame`] on the stack, then the
        // gas needs to be reconciled.
        if self.stack.len() > 1 {
            let len = self.stack.len() - 2;
            let spent = self.gas_meter().spent();
            let parent_meter = &mut self.stack[len].gas_meter;

            parent_meter.charge(spent)?
        }
        Ok(self.gas_meter().clone())
    }
}
