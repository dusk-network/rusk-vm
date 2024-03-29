// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::mem;

use microkelvin::{BranchRef, BranchRefMut, MaybeArchived};
use rusk_uplink::{
    ContractId, RawQuery, RawTransaction, ReturnValue, StoreContext,
};

use tracing::{trace, trace_span};
use wasmer::{Exports, ImportObject, Instance, LazyInit, Module, NativeFunc};
use wasmer_middlewares::metering::set_remaining_points;
use wasmer_types::Value;

use crate::env::Env;
use crate::gas::{Gas, GasMeter};
use crate::memory::WasmerMemory;
use crate::modules::compile_module;
use crate::resolver::HostImportsResolver;
use crate::state::{Event, NetworkState};
use crate::{Config, VMError};

const SCRATCH_NAME: &str = "scratch";
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
    events: Vec<Event>,
    block_height: u64,
    store: StoreContext,
}

impl<'a> CallContext<'a> {
    pub fn new(
        state: &'a mut NetworkState,
        block_height: u64,
        store: StoreContext,
    ) -> Self {
        CallContext {
            state,
            stack: vec![],
            events: vec![],
            block_height,
            store,
        }
    }

    pub fn store(&self) -> &StoreContext {
        &self.store
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
        query: RawQuery,
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

        let r = if let Some(_module) =
            self.state.modules().get_module_ref(&target).get()
        {
            // is this a reserved module call?
            todo!()
        // todo bogus error just to fix compilation errors
        } else {
            let contract = self.state.get_contract(&target)?;
            let contract = contract.leaf();

            let bytecode = match contract {
                MaybeArchived::Memory(m) => m.bytecode(),
                MaybeArchived::Archived(a) => a.bytecode(&self.store),
            };

            let module = compile_module(bytecode, self.state.config())?;

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

            self.stack.push(StackFrame::new(
                target,
                memory,
                gas_meter.clone(),
                instance.clone(),
            ));

            let run_func: NativeFunc<(u32, u32), u32> =
                instance.exports.get_native_function(query.name())?;

            let buf_offset = if let Value::I32(ofs) = instance
                .exports
                .get_global(SCRATCH_NAME)
                .map_err(|_| VMError::InvalidWASMModule)?
                .get()
            {
                ofs as usize
            } else {
                return Err(VMError::InvalidWASMModule);
            };

            let mut memory = WasmerMemory::new();
            memory.init(&instance.exports)?;

            // Write the current archived state and the query into contract
            // scratch buffer

            let (written_state, written_data) =
                memory.with_mut_slice_from(buf_offset, |mem| {
                    let state = match contract {
                        MaybeArchived::Memory(m) => m.state(),
                        MaybeArchived::Archived(a) => a.state(&self.store),
                    };

                    let len = state.len() as usize;

                    mem[0..len].copy_from_slice(state);

                    let data = query.data();

                    mem[len..len + data.len()].copy_from_slice(data);

                    (len, len + data.len())
                });

            let r = run_func.call(written_state as u32, written_data as u32);

            r.map(|result_written| {
                memory.with_slice_from(buf_offset, |mem| {
                    ReturnValue::new(&mem[..result_written as usize])
                })
            })
        };

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

        let result =
            r.map_err(|a| VMError::ContractPanic(target, a.message()))?;
        self.stack.pop();
        Ok(result)
    }

    pub fn transact(
        &mut self,
        target: ContractId,
        transaction: RawTransaction,
        gas_meter: &'a mut GasMeter,
    ) -> Result<ReturnValue, VMError> {
        let _span = trace_span!(
            "transact",
            target = ?target,
            gas_limit = ?gas_meter.limit(),
            stack_index = ?self.stack.len()
        );

        let env = Env::new(self);

        let instance;

        let r = {
            let config = self.state.config();
            let mut contract = self.state.get_contract_mut(&target)?;
            let contract = contract.leaf_mut();

            let module = compile_module(contract.bytecode(), config)?;

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

            self.stack.push(StackFrame::new(
                target,
                memory,
                gas_meter.clone(),
                instance.clone(),
            ));

            let run_func: NativeFunc<(u32, u32), u64> =
                instance.exports.get_native_function(transaction.name())?;

            let buf_offset = if let Value::I32(ofs) = instance
                .exports
                .get_global(SCRATCH_NAME)
                .map_err(|_| VMError::InvalidWASMModule)?
                .get()
            {
                ofs as usize
            } else {
                return Err(VMError::InvalidWASMModule);
            };

            let mut memory = WasmerMemory::new();
            memory.init(&instance.exports)?;

            // Copy the contract state and the transaction into scratch memory

            let (written_state, written_data) =
                memory.with_mut_slice_from(buf_offset, |mem| {
                    // copy the contract state into scratch memory
                    let state = contract.state();

                    let len = state.len();

                    mem[..len].copy_from_slice(state);

                    let data = transaction.data();
                    let data_len = data.len();

                    mem[len..len + data_len].copy_from_slice(data);

                    (len, len + data_len)
                });

            // note to self: refactor plz, this can be done with bit-shifting
            fn separate_tuple(tuple: u64) -> (u32, u32) {
                let bytes = tuple.to_le_bytes();
                let mut a = [0u8; 4];
                let mut b = [0u8; 4];
                a.copy_from_slice(&bytes[..4]);
                b.copy_from_slice(&bytes[4..]);
                (u32::from_le_bytes(a), u32::from_le_bytes(b))
            }

            let r = run_func.call(written_state as u32, written_data as u32);

            r.map(|result| {
                let (state_written, result_written) = separate_tuple(result);

                memory.with_slice_from(buf_offset, |mem| {
                    let new_state = &mem[..state_written as usize];

                    contract.set_state(new_state);

                    let result_len = result_written - state_written;
                    ReturnValue::with_state(
                        &mem[state_written as usize..][..result_len as usize],
                        &mem[..state_written as usize],
                    )
                })
            })
        };

        match self.gas_reconciliation() {
            Ok(gas) => *gas_meter = gas,
            Err(e) => {
                gas_meter.exhaust();
                return Err(e);
            }
        }
        trace!(
            "Finished transaction with gas limit/spent: {}/{}",
            gas_meter.limit(),
            gas_meter.spent()
        );

        let result =
            r.map_err(|a| VMError::ContractPanic(target, a.message()))?;
        self.stack.pop();
        Ok(result)
    }

    pub fn push_event(
        &mut self,
        origin: ContractId,
        name: String,
        data: Vec<u8>,
    ) {
        self.events.push(Event::new(origin, name, data));
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        mem::take(&mut self.events)
    }

    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    pub fn gas_meter(&mut self) -> Result<&GasMeter, VMError> {
        let stack = &mut self.top_mut();
        let instance = &stack.instance;
        let gas_meter = &mut stack.gas_meter;

        gas_meter.update(instance, 0)?;

        Ok(&self.top().gas_meter)
    }
    /// Charge gas to the meter in the topmost stack frame.
    pub fn charge_gas(&mut self, gas: Gas) -> Result<(), VMError> {
        let frame = &mut self.top_mut();
        let instance = &frame.instance;
        let gas_meter = &mut frame.gas_meter;

        gas_meter.update(instance, gas)?;

        Ok(())
    }

    pub fn config(&self) -> &'static Config {
        self.state.config()
    }

    pub fn top(&self) -> &StackFrame {
        self.stack.last().expect("Stack should not be empty")
    }

    pub fn top_mut(&mut self) -> &mut StackFrame {
        self.stack.last_mut().expect("Stack should not be empty")
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
        self.top_mut().write_memory(source_slice, offset)?;
        Ok(())
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
