use std::marker::PhantomData;

use dusk_abi::{CALL_DATA_SIZE, H256};
use kelvin::{ByteHash, ValRef, ValRefMut};

use wasmi::{
    ExternVal, Externals, ImportsBuilder, MemoryRef, ModuleImportResolver,
    ModuleInstance, RuntimeArgs, RuntimeValue, Trap, TrapKind,
};

use crate::contract::MeteredContract;
use crate::gas::GasMeter;
use crate::state::{NetworkState, Storage};
use crate::VMError;

pub trait Resolver<H: ByteHash>:
    Invoke<H> + ModuleImportResolver + Default + Clone
{
}

pub use crate::resolver::CompoundResolver as StandardABI;

pub struct StackFrame {
    pub context: H256,
    pub call_data: [u8; CALL_DATA_SIZE],
    pub memory: MemoryRef,
}

impl StackFrame {
    fn new(
        context: H256,
        call_data: [u8; CALL_DATA_SIZE],
        memory: MemoryRef,
    ) -> Self {
        StackFrame {
            context,
            call_data,
            memory,
        }
    }

    fn into_call_data(self) -> [u8; CALL_DATA_SIZE] {
        self.call_data
    }
}

pub trait Invoke<H: ByteHash>: Sized {
    fn invoke(
        context: &mut CallContext<Self, H>,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError>;
}

pub struct CallContext<'a, S, H: ByteHash> {
    state: &'a mut NetworkState<S, H>,
    stack: Vec<StackFrame>,
    gas_meter: &'a mut GasMeter,
    _marker: PhantomData<S>,
}

impl<'a, S: Resolver<H>, H: ByteHash> CallContext<'a, S, H> {
    pub fn new(
        state: &'a mut NetworkState<S, H>,
        gas_meter: &'a mut GasMeter,
    ) -> Self {
        CallContext {
            stack: vec![],
            state,
            gas_meter,
            _marker: PhantomData,
        }
    }

    pub fn call(
        &mut self,
        target: &H256,
        call_data: [u8; CALL_DATA_SIZE],
    ) -> Result<[u8; CALL_DATA_SIZE], VMError> {
        let resolver = S::default();
        let imports = ImportsBuilder::new().with_resolver("env", &resolver);

        let instance;

        match self.state.get_contract_state_mut(&target)? {
            None => return Err(VMError::UnknownContract),
            Some(mut contract_state) => {
                contract_state.contract_mut().ensure_compiled()?;

                let module = match contract_state.contract() {
                    MeteredContract::Module { ref module, .. } => module,
                    _ => unreachable!(),
                };

                instance =
                    ModuleInstance::new(&*module, &imports)?.assert_no_start();

                match instance.export_by_name("memory") {
                    Some(ExternVal::Memory(memref)) => self.stack.push(
                        StackFrame::new(target.clone(), call_data, memref),
                    ),
                    _ => return Err(VMError::MemoryNotFound),
                }
            }
        }

        match instance.invoke_export("call", &[], self) {
            Err(wasmi::Error::Trap(trap)) => {
                if let TrapKind::Host(t) = trap.kind() {
                    if let Some(vm_error) = (**t).downcast_ref::<VMError>() {
                        if let VMError::ContractReturn = vm_error {
                            // Return is fine, pass it through
                        } else {
                            return Err(wasmi::Error::Trap(trap).into());
                        }
                    } else {
                        return Err(wasmi::Error::Trap(trap).into());
                    }
                } else {
                    return Err(wasmi::Error::Trap(trap).into());
                }
            }
            Err(e) => return Err(e.into()),
            Ok(_) => (),
        }

        // return the call_data (now containing return value)
        Ok(self.stack.pop().expect("Invalid stack").into_call_data())
    }

    pub fn data(&self) -> &[u8] {
        let top = self.top();
        &top.call_data
    }

    pub fn memory(&self) -> &MemoryRef {
        &self.top().memory
    }

    pub fn memory_mut(&mut self) -> &mut MemoryRef {
        &mut self.top_mut().memory
    }

    pub fn gas_meter_mut(&mut self) -> &mut GasMeter {
        self.gas_meter
    }

    pub fn top(&self) -> &StackFrame {
        &self.stack.last().expect("Empty stack")
    }

    pub fn top_mut(&mut self) -> &mut StackFrame {
        self.stack.last_mut().expect("Empty stack")
    }

    pub fn caller(&self) -> &H256 {
        // for top level, caller is the same as called.
        let i = self.stack.len().saturating_sub(1);
        &self.stack.get(i).expect("Empty stack").context
    }

    pub fn called(&self) -> &H256 {
        &self.top().context
    }

    pub fn storage(&self) -> Result<impl ValRef<Storage<H>>, VMError> {
        match self.state.get_contract_state(&self.caller())? {
            Some(state) => Ok(state.wrap(|state| state.storage())),
            None => Err(VMError::UnknownContract),
        }
    }

    pub fn storage_mut(
        &mut self,
    ) -> Result<impl ValRefMut<Storage<H>>, VMError> {
        let caller = *self.caller();
        Ok(self
            .state
            .get_contract_state_mut(&caller)?
            .expect("Invalid caller")
            .wrap_mut(|state| state.storage_mut()))
    }

    pub fn state(&self) -> &NetworkState<S, H> {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut NetworkState<S, H> {
        &mut self.state
    }

    pub fn balance(&self) -> Result<u128, VMError> {
        Ok(self
            .state
            .get_contract_state(&self.caller())?
            .expect("Invalid caller")
            .balance())
    }

    pub fn balance_mut(&mut self) -> Result<impl ValRefMut<u128>, VMError> {
        let caller = *self.caller();
        Ok(self
            .state
            .get_contract_state_mut(&caller)?
            .expect("Invalid caller")
            .wrap_mut(|state| state.balance_mut()))
    }
}

/// Convenience function to construct host traps
pub fn host_trap(host: VMError) -> Trap {
    Trap::new(TrapKind::Host(Box::new(host)))
}

/// Convenience trait to extract arguments
pub trait ArgsExt {
    fn get(&self, i: usize) -> Result<usize, Trap>;
    fn to_slice(&self, bytes: &[u8], args_ofs: usize) -> Result<&[u8], Trap>;
}

impl ArgsExt for RuntimeArgs<'_> {
    fn get(&self, i: usize) -> Result<usize, Trap> {
        self.as_ref()[i]
            .try_into::<i32>()
            .ok_or_else(|| host_trap(VMError::MissingArgument))
            .map(|i| i as usize)
    }

    fn to_slice(&self, bytes: &[u8], args_ofs: usize) -> Result<&[u8], Trap> {
        let args = self.as_ref();
        let ofs: u32 = args[args_ofs]
            .try_into()
            .ok_or_else(|| host_trap(VMError::MissingArgument))?;
        let len: u32 = args[args_ofs + 1]
            .try_into()
            .ok_or_else(|| host_trap(VMError::MissingArgument))?;
        unsafe {
            Ok(std::slice::from_raw_parts(
                &bytes[ofs as usize],
                len as usize,
            ))
        }
    }
}

impl<'a, S: Resolver<H>, H: ByteHash> Externals for CallContext<'a, S, H> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match S::invoke(self, index, args) {
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
<<<<<<< HEAD
=======

impl ModuleImportResolver for HostImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, wasmi::Error> {
        match field_name {
            "panic" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                ABI_PANIC,
            )),
            "debug" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                ABI_DEBUG,
            )),
            "set_storage" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    None,
                ),
                ABI_SET_STORAGE,
            )),
            "get_storage" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32),
                ),
                ABI_GET_STORAGE,
            )),
            "caller" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                ABI_CALLER,
            )),
            "self_hash" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                ABI_SELF_HASH,
            )),
            "call_data" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                ABI_CALL_DATA,
            )),
            "verify_ed25519_signature" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                    ][..],
                    Some(ValueType::I32),
                ),
                ABI_VERIFY_ED25519_SIGNATURE,
            )),
            "call_contract" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                    ][..],
                    None,
                ),
                ABI_CALL_CONTRACT,
            )),
            "balance" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                ABI_BALANCE,
            )),
            "ret" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                ABI_RETURN,
            )),
            "gas" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                ABI_GAS,
            )),
            "phoenix_store" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                ABI_PHOENIX_STORE,
            )),
            name => unimplemented!("{:?}", name),
        }
    }
}
>>>>>>> d18e0f5... remove proof argument for ABI_PHOENIX_STORE, and add the return argument to the function signature
