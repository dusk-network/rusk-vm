// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::{BranchRef, BranchRefMut};
use rusk_uplink::{
    ContractId, RawQuery, RawTransaction, ReturnValue, StoreContext,
};

use tracing::{trace, trace_span};
use wasmer::{Exports, ImportObject, Instance, LazyInit, Module, NativeFunc};
use wasmer_middlewares::metering::{
    get_remaining_points, set_remaining_points, MeteringPoints,
};
use wasmer_types::Value;

use crate::contract::ContractRef;
use crate::env::Env;
use crate::gas::GasMeter;
use crate::memory::WasmerMemory;
use crate::modules::compile_module;
use crate::resolver::HostImportsResolver;
use crate::state::NetworkState;
use crate::VMError;

const SCRATCH_NAME: &str = "scratch";

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
    store: StoreContext,
    target_store: StoreContext,
}

impl<'a> CallContext<'a> {
    pub fn new(
        state: &'a mut NetworkState,
        block_height: u64,
        store: StoreContext,
        target_store: StoreContext,
    ) -> Self {
        CallContext {
            state,
            stack: vec![],
            block_height,
            store,
            target_store,
        }
    }

    pub fn store(&self) -> &StoreContext {
        &self.store
    }

    pub fn target_store(&self) -> &StoreContext {
        &self.target_store
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

            let module = compile_module(
                contract.bytecode(),
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

            self.stack
                .push(StackFrame::new(target, memory, gas_meter.clone()));

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
                    let state = contract.state();
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

        let result = r.map_err(|a| VMError::ContractPanic(a.message()))?;
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

        let config = self.state.get_module_config().clone();

        let r = {
            let mut contract = self.state.get_contract_mut(&target)?;
            let contract = contract.leaf_mut();

            let module = compile_module(contract.bytecode(), &config)?;

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

            self.stack
                .push(StackFrame::new(target, memory, gas_meter.clone()));

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

        match self.gas_reconciliation(&instance) {
            Ok(gas) => *gas_meter = gas,
            Err(e) => {
                gas_meter.set_left(0);
                return Err(e);
            }
        }
        trace!(
            "Finished transaction with gas limit/spent: {}/{}",
            gas_meter.limit(),
            gas_meter.spent()
        );

        let result = r.map_err(|a| VMError::ContractPanic(a.message()))?;
        self.stack.pop();
        Ok(result)
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
