use std::marker::PhantomData;
use std::mem;

use dataview::Pod;
use dusk_abi::H256;
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
    callee: H256,
    argument_ptr: usize,
    argument_len: usize,
    return_ptr: usize,
    return_len: usize,
    memory: MemoryRef,
}

impl std::fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(arg: {} return: {})",
            self.argument_ptr, self.return_ptr
        )
    }
}

impl StackFrame {
    fn new(
        callee: H256,
        memory: MemoryRef,
        argument_ptr: usize,
        argument_len: usize,
        return_ptr: usize,
        return_len: usize,
    ) -> Self {
        StackFrame {
            callee,
            memory,
            argument_ptr,
            argument_len,
            return_ptr,
            return_len,
        }
    }

    fn memory<R, C: FnOnce(&[u8]) -> R>(&self, closure: C) -> R {
        self.memory.with_direct_access(closure)
    }

    fn memory_mut<R, C: FnOnce(&mut [u8]) -> R>(&self, closure: C) -> R {
        self.memory.with_direct_access_mut(closure)
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
    top_argument: &'a [u8],
    top_return: &'a mut [u8],
    _marker: PhantomData<S>,
}

impl<'a, S, H> CallContext<'a, S, H>
where
    S: Resolver<H>,
    H: ByteHash,
{
    pub fn new<A: Pod, R: Pod>(
        state: &'a mut NetworkState<S, H>,
        gas_meter: &'a mut GasMeter,
        top_argument: &'a A,
        top_return: &'a mut R,
    ) -> Self {
        CallContext {
            state,
            stack: vec![],
            gas_meter,
            top_argument: top_argument.as_bytes(),
            top_return: top_return.as_bytes_mut(),
            _marker: PhantomData,
        }
    }

    pub fn call(
        &mut self,
        target: H256,
        argument_ptr: usize,
        argument_len: usize,
        return_ptr: usize,
        return_len: usize,
    ) -> Result<Option<RuntimeValue>, VMError> {
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
                    Some(ExternVal::Memory(memref)) => {
                        self.stack.push(StackFrame::new(
                            target,
                            memref,
                            argument_ptr,
                            argument_len,
                            return_ptr,
                            return_len,
                        ))
                    }
                    _ => return Err(VMError::MemoryNotFound),
                }
            }
        }

        match instance.invoke_export("call", &[], self) {
            Err(wasmi::Error::Trap(trap)) => {
                if let TrapKind::Host(t) = trap.kind() {
                    if let Some(vm_error) = (**t).downcast_ref::<VMError>() {
                        if let VMError::ContractReturn(ofs, len) = vm_error {
                            // copy the return value to return pointer
                            self.copy_return(*ofs, *len);
                            self.stack.pop();
                            Ok(None)
                        } else {
                            Err(wasmi::Error::Trap(trap).into())
                        }
                    } else {
                        Err(wasmi::Error::Trap(trap).into())
                    }
                } else {
                    Err(wasmi::Error::Trap(trap).into())
                }
            }
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r),
        }
    }

    pub fn copy_return(&mut self, source_ptr: usize, len: usize) {
        if self.stack.len() > 1 {
            let (copy_from, rest) =
                self.stack.split_last_mut().expect("len > 1");
            let copy_to = rest.last().expect("len > 1");

            let dest_ptr = copy_from.return_ptr;
            let dest_len = copy_from.return_len;

            assert_eq!(dest_len, dest_len);

            copy_from.memory(|source| {
                copy_to.memory_mut(|dest| {
                    dest[dest_ptr..dest_ptr + len]
                        .copy_from_slice(&source[source_ptr..source_ptr + len])
                })
            })
        } else {
            let CallContext {
                ref stack,
                ref mut top_return,
                ..
            } = self;

            stack.last().expect("invalid stack").memory(|m| {
                top_return.copy_from_slice(&m[source_ptr..source_ptr + len]);
            })
        }
    }

    /// Copies the arguments from the previous stack frame into current memory
    pub fn copy_argument(&mut self, dest_ptr: usize, len: usize) {
        if self.stack.len() > 1 {
            //panic!("{:?}", self.stack);

            let (copy_to, rest) = self.stack.split_last_mut().expect("len > 1");
            let copy_from = rest.last().expect("len > 1");

            let source_ptr = copy_to.argument_ptr;
            let source_len = copy_to.argument_len;

            assert_eq!(source_len, len);

            copy_from.memory(|source| {
                copy_to.memory_mut(|dest| {
                    dest[dest_ptr..dest_ptr + len]
                        .copy_from_slice(&source[source_ptr..source_ptr + len])
                })
            })
        } else {
            let CallContext {
                ref mut stack,
                ref top_argument,
                ..
            } = self;

            stack.last_mut().expect("invalid stack").memory_mut(|m| {
                m[dest_ptr..dest_ptr + len].copy_from_slice(top_argument)
            })
        }
    }

    pub fn gas_meter_mut(&mut self) -> &mut GasMeter {
        self.gas_meter
    }

    pub fn top(&self) -> &StackFrame {
        self.stack.last().expect("Invalid stack")
    }

    pub fn callee(&self) -> H256 {
        self.top().callee
    }

    pub fn memory<R, C: FnOnce(&[u8]) -> R>(&self, closure: C) -> R {
        self.top().memory(closure)
    }

    pub fn memory_mut<R, C: FnOnce(&mut [u8]) -> R>(
        &mut self,
        closure: C,
    ) -> R {
        self.stack
            .last_mut()
            .expect("Invalid stack")
            .memory_mut(closure)
    }

    pub fn memory_storage_mut<
        R,
        C: FnOnce(&mut [u8], &mut Storage<H>) -> Result<R, VMError>,
    >(
        &mut self,
        closure: C,
    ) -> Result<R, VMError> {
        let CallContext {
            ref mut stack,
            ref mut state,
            ..
        } = self;

        let frame = stack.last_mut().expect("Invalid stack");

        let callee = &frame.callee;
        let mut state = state.get_contract_state_mut_or_default(callee)?;
        let storage = state.storage_mut();

        frame.memory_mut(|m| closure(m, storage))
    }

    pub fn write_at<V: Pod>(&mut self, ofs: usize, value: &V) {
        self.memory_mut(|m| {
            m[ofs..ofs + mem::size_of::<V>()].copy_from_slice(value.as_bytes());
        });
    }

    pub fn storage(&self) -> Result<impl ValRef<Storage<H>>, VMError> {
        match self.state.get_contract_state(&self.callee())? {
            Some(state) => Ok(state.wrap(|state| state.storage())),
            None => Err(VMError::UnknownContract),
        }
    }

    pub fn storage_mut(
        &mut self,
    ) -> Result<impl ValRefMut<Storage<H>>, VMError> {
        let callee = self.callee();
        Ok(self
            .state
            .get_contract_state_mut(&callee)?
            .expect("Invalid callee")
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
            .get_contract_state(&self.callee())?
            .expect("Invalid callee")
            .balance())
    }

    pub fn balance_mut(&mut self) -> Result<impl ValRefMut<u128>, VMError> {
        let callee = self.callee();
        Ok(self
            .state
            .get_contract_state_mut(&callee)?
            .expect("Invalid callee")
            .wrap_mut(|state| state.balance_mut()))
    }
}

/// Convenience function to construct host traps
pub fn host_trap(host: VMError) -> Trap {
    Trap::new(TrapKind::Host(Box::new(host)))
}

/// Convenience trait to extract arguments
pub trait ArgsExt {
    fn get(&self, i: usize) -> Result<i32, Trap>;
}

impl ArgsExt for RuntimeArgs<'_> {
    fn get(&self, i: usize) -> Result<i32, Trap> {
        self.as_ref()[i]
            .try_into::<i32>()
            .ok_or_else(|| host_trap(VMError::MissingArgument))
    }
}

impl<'a, S, H> Externals for CallContext<'a, S, H>
where
    S: Resolver<H>,
    H: ByteHash,
{
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
