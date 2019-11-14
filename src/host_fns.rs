use dusk_abi::{
    encoding, CALL_DATA_SIZE, H256, STORAGE_KEY_SIZE, STORAGE_VALUE_SIZE,
};
use kelvin::{ValRef, ValRefMut};
use signatory::{ed25519, Signature as _, Verifier as _};

use wasmi::{
    ExternVal, Externals, FuncInstance, FuncRef, ImportsBuilder, MemoryRef,
    ModuleImportResolver, ModuleInstance, RuntimeArgs, RuntimeValue, Signature,
    Trap, TrapKind, ValueType,
};

use crate::gas::GasMeter;
use crate::state::{NetworkState, Storage};
use crate::VMError;

const ABI_PANIC: usize = 0;
const ABI_DEBUG: usize = 1;
const ABI_SET_STORAGE: usize = 2;
const ABI_GET_STORAGE: usize = 3;
const ABI_CALLER: usize = 4;
const ABI_CALL_DATA: usize = 5;
const ABI_VERIFY_ED25519_SIGNATURE: usize = 6;
const ABI_CALL_CONTRACT: usize = 7;
const ABI_BALANCE: usize = 8;
const ABI_RETURN: usize = 9;
const ABI_SELF_HASH: usize = 10;
const ABI_GAS: usize = 11;

#[derive(Debug, Clone, Copy)]
pub enum CallKind {
    Deploy,
    Call,
}

struct StackFrame {
    context: H256,
    call_data: [u8; CALL_DATA_SIZE],
    call_kind: CallKind,
    memory: MemoryRef,
}

impl StackFrame {
    fn new(
        context: H256,
        call_data: [u8; CALL_DATA_SIZE],
        call_kind: CallKind,
        memory: MemoryRef,
    ) -> Self {
        StackFrame {
            context,
            call_data,
            call_kind,
            memory,
        }
    }

    fn into_call_data(self) -> [u8; CALL_DATA_SIZE] {
        self.call_data
    }
}

pub(crate) struct CallContext<'a> {
    state: &'a mut NetworkState,
    stack: Vec<StackFrame>,
    gas_meter: Option<&'a mut GasMeter>,
}

impl<'a> CallContext<'a> {
    pub fn with_limit(
        state: &'a mut NetworkState,
        gas_meter: &'a mut GasMeter,
    ) -> Self {
        CallContext {
            stack: vec![],
            state,
            gas_meter: Some(gas_meter),
        }
    }

    pub fn new(state: &'a mut NetworkState) -> Self {
        CallContext {
            stack: vec![],
            state,
            gas_meter: None,
        }
    }

    pub fn call(
        &mut self,
        target: H256,
        call_data: [u8; CALL_DATA_SIZE],
        kind: CallKind,
    ) -> Result<[u8; CALL_DATA_SIZE], VMError> {
        let imports =
            ImportsBuilder::new().with_resolver("env", &HostImportResolver);

        let mut skip_call = false;

        let (instance, name);

        match self.state.get_contract_state_mut(&target)? {
            None => return Err(VMError::UnknownContract),
            Some(contract_state) => {
                let module = wasmi::Module::from_buffer(
                    contract_state.contract().bytecode(),
                )?;

                instance =
                    ModuleInstance::new(&module, &imports)?.assert_no_start();

                match instance.export_by_name("memory") {
                    Some(ExternVal::Memory(memref)) => self
                        .stack
                        .push(StackFrame::new(target, call_data, kind, memref)),
                    _ => return Err(VMError::MemoryNotFound),
                }

                name = match kind {
                    CallKind::Deploy => {
                        if instance.export_by_name("deploy").is_none() {
                            skip_call = true
                        };
                        "deploy"
                    }
                    CallKind::Call => "call",
                };
            }
        }

        if !skip_call {
            match instance.invoke_export(name, &[], self) {
                Err(wasmi::Error::Trap(trap)) => {
                    if let TrapKind::Host(t) = trap.kind() {
                        if let Some(vm_error) = (**t).downcast_ref::<VMError>()
                        {
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
        }

        // return the call_data (now containing return value)
        Ok(self.stack.pop().expect("Invalid stack").into_call_data())
    }

    fn data(&self) -> &[u8] {
        let top = self.top();
        &top.call_data
    }

    fn memory(&self) -> &MemoryRef {
        &self.top().memory
    }

    fn memory_mut(&mut self) -> &mut MemoryRef {
        &mut self.top_mut().memory
    }

    fn top(&self) -> &StackFrame {
        &self.stack.last().expect("Empty stack")
    }

    fn top_mut(&mut self) -> &mut StackFrame {
        self.stack.last_mut().expect("Empty stack")
    }

    fn caller(&self) -> &H256 {
        // for top level, caller is the same as called.
        let i = self.stack.len().saturating_sub(1);
        &self.stack.get(i).expect("Empty stack").context
    }

    fn called(&self) -> &H256 {
        &self.top().context
    }

    fn storage(&self) -> Result<impl ValRef<Storage>, VMError> {
        match self.state.get_contract_state(&self.caller())? {
            Some(state) => Ok(state.wrap(|state| state.storage())),
            None => Err(VMError::UnknownContract),
        }
    }

    fn storage_mut(&mut self) -> Result<impl ValRefMut<Storage>, VMError> {
        let caller = *self.caller();
        Ok(self
            .state
            .get_contract_state_mut(&caller)?
            .expect("Invalid caller")
            .wrap_mut(|state| state.storage_mut()))
    }

    fn balance(&self) -> Result<u128, VMError> {
        Ok(self
            .state
            .get_contract_state(&self.caller())?
            .expect("Invalid caller")
            .balance())
    }

    fn balance_mut(&mut self) -> Result<impl ValRefMut<u128>, VMError> {
        let caller = *self.caller();
        Ok(self
            .state
            .get_contract_state_mut(&caller)?
            .expect("Invalid caller")
            .wrap_mut(|state| state.balance_mut()))
    }
}

pub(crate) struct HostImportResolver;

fn args_to_slice<'a>(
    bytes: &'a [u8],
    args_ofs: usize,
    args: &RuntimeArgs,
) -> Result<&'a [u8], Trap> {
    let args = args.as_ref();
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

// Convenience function to construct host traps
fn host_trap(host: VMError) -> Trap {
    Trap::new(TrapKind::Host(Box::new(host)))
}

// Convenience trait to extract arguments
trait ArgsExt {
    fn get(&self, i: usize) -> Result<usize, Trap>;
}

impl ArgsExt for RuntimeArgs<'_> {
    fn get(&self, i: usize) -> Result<usize, Trap> {
        self.as_ref()[i]
            .try_into::<i32>()
            .ok_or_else(|| host_trap(VMError::MissingArgument))
            .map(|i| i as usize)
    }
}

fn error_handling_invoke_index(
    context: &mut CallContext,
    index: usize,
    args: RuntimeArgs,
) -> Result<Option<RuntimeValue>, VMError> {
    match index {
        ABI_PANIC => {
            let panic_ofs = args.get(0)?;
            let panic_len = args.get(1)?;

            context.top().memory.with_direct_access(|a| {
                Err(
                    match String::from_utf8(
                        a[panic_ofs..panic_ofs + panic_len].to_vec(),
                    ) {
                        Ok(panic_msg) => {
                            host_trap(VMError::ContractPanic(panic_msg))
                        }
                        Err(_) => host_trap(VMError::InvalidUtf8),
                    },
                )
            })?
        }
        ABI_SET_STORAGE => {
            let key_ofs = args.get(0)?;
            let val_ofs = args.get(1)?;
            let val_len = args.get(2)?;

            let mut key_buf = H256::default();
            let mut val_buf = [0u8; STORAGE_VALUE_SIZE];

            context.top().memory.with_direct_access(|a| {
                key_buf
                    .as_mut()
                    .copy_from_slice(&a[key_ofs..key_ofs + STORAGE_KEY_SIZE]);
                val_buf[0..val_len]
                    .copy_from_slice(&a[val_ofs..val_ofs + val_len]);
            });
            context
                .storage_mut()?
                .insert(key_buf, val_buf[0..val_len].into())?;

            Ok(None)
        }
        // Return value indicates if the key was found or not
        ABI_GET_STORAGE => {
            // offset to where to write the value in memory
            let key_buf_ofs = args.get(0)?;
            let val_buf_ofs = args.get(1)?;

            let mut key_buf = H256::default();

            context.top().memory.with_direct_access(|a| {
                key_buf.as_mut().copy_from_slice(
                    &a[key_buf_ofs..key_buf_ofs + STORAGE_KEY_SIZE],
                );
            });

            match context.storage()?.get(&key_buf)? {
                Some(val) => {
                    let len = val.len();
                    context.top().memory.with_direct_access_mut(|a| {
                        a[val_buf_ofs..val_buf_ofs + len].copy_from_slice(&val)
                    });
                    Ok(Some(RuntimeValue::I32(len as i32)))
                }
                None => Ok(Some(RuntimeValue::I32(0))),
            }
        }
        ABI_DEBUG => context.top().memory.with_direct_access(|a| {
            let slice = args_to_slice(a, 0, &args)?;
            let str = std::str::from_utf8(slice)
                .map_err(|_| host_trap(VMError::InvalidUtf8))?;
            println!("CONTRACT DEBUG: {:?}", str);
            Ok(None)
        }),
        ABI_CALLER => {
            let buffer_ofs = args.get(0)?;

            context.top().memory.with_direct_access_mut(|a| {
                a[buffer_ofs..buffer_ofs + 32]
                    .copy_from_slice(context.caller().as_ref())
            });
            Ok(None)
        }
        ABI_SELF_HASH => {
            let buffer_ofs = args.get(0)?;

            context.top().memory.with_direct_access_mut(|a| {
                a[buffer_ofs..buffer_ofs + 32]
                    .copy_from_slice(context.called().as_ref())
            });
            Ok(None)
        }
        ABI_CALL_DATA => {
            let call_data_ofs = args.get(0)?;

            context.top().memory.with_direct_access_mut(|a| {
                a[call_data_ofs..call_data_ofs + CALL_DATA_SIZE]
                    .copy_from_slice(context.data())
            });
            Ok(None)
        }
        ABI_VERIFY_ED25519_SIGNATURE => {
            let key_ptr = args.get(0)?;
            let sig_ptr = args.get(1)?;

            context.top().memory.with_direct_access_mut(|a| {
                let pub_key =
                    ed25519::PublicKey::from_bytes(&a[key_ptr..key_ptr + 32])
                        .ok_or_else(|| {
                        host_trap(VMError::InvalidEd25519PublicKey)
                    })?;

                let signature = ed25519::Signature::from_bytes(
                    &a[sig_ptr..sig_ptr + 64],
                )
                .map_err(|_| host_trap(VMError::InvalidEd25519Signature))?;

                let data_slice = args_to_slice(a, 2, &args)?;

                let verifier: signatory_dalek::Ed25519Verifier =
                    (&pub_key).into();

                match verifier.verify(data_slice, &signature) {
                    Ok(_) => Ok(Some(RuntimeValue::I32(1))),
                    Err(_) => Ok(Some(RuntimeValue::I32(0))),
                }
            })
        }
        ABI_CALL_CONTRACT => {
            let target_ofs = args.get(0)?;
            let amount_ofs = args.get(1)?;
            let data_ofs = args.get(2)?;
            let data_len = args.get(3)?;

            let mut call_buf = [0u8; CALL_DATA_SIZE];
            let mut target = H256::zero();
            let mut amount = u128::default();

            context
                .memory()
                .with_direct_access::<Result<(), VMError>, _>(|a| {
                    target = encoding::decode(&a[target_ofs..target_ofs + 32])?;
                    amount = encoding::decode(&a[amount_ofs..amount_ofs + 16])?;
                    call_buf[0..data_len]
                        .copy_from_slice(&a[data_ofs..data_ofs + data_len]);
                    Ok(())
                })?;
            // assure sufficient funds are available
            if context.balance()? >= amount {
                *context.balance_mut()? -= amount;
                *context
                    .state
                    .get_contract_state_mut_or_default(&target)?
                    .balance_mut() += amount;
            } else {
                panic!("not enough funds")
            }

            if data_len > 0 {
                let return_buf =
                    context.call(target, call_buf, CallKind::Call)?;
                // write the return data back into memory
                context.memory().with_direct_access_mut(|a| {
                    a[data_ofs..data_ofs + CALL_DATA_SIZE]
                        .copy_from_slice(&return_buf)
                })
            }

            Ok(None)
        }
        ABI_BALANCE => {
            // first argument is a pointer to a 16 byte buffer
            let buffer_ofs = args.get(0)?;
            let balance = context.balance()?;

            context.memory_mut().with_direct_access_mut(|a| {
                encoding::encode(&balance, &mut a[buffer_ofs..]).map(|_| ())
            })?;

            Ok(None)
        }
        ABI_RETURN => {
            let buffer_ofs = args.get(0)?;

            let StackFrame {
                ref mut memory,
                call_data,
                ..
            } = context.top_mut();

            // copy return value from memory into call_data
            memory.with_direct_access_mut(|a| {
                call_data.copy_from_slice(
                    &a[buffer_ofs..buffer_ofs + CALL_DATA_SIZE],
                );
            });

            Err(VMError::ContractReturn)
        }
        ABI_GAS => {
            if let Some(ref mut meter) = context.gas_meter {
                let gas: u32 = args.nth_checked(0)?;
                if meter.charge(gas as u64).is_out_of_gas() {
                    return Err(VMError::OutOfGas);
                }
            }
            Ok(None)
        }
        _ => Err(VMError::InvalidApiCall),
    }
}

impl<'a> Externals for CallContext<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match error_handling_invoke_index(self, index, args) {
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
            name => unimplemented!("{:?}", name),
        }
    }
}
