use dusk_abi::{
    encoding, CALL_DATA_SIZE, H256, STORAGE_KEY_SIZE, STORAGE_VALUE_SIZE,
};
use failure::{bail, Error};
use signatory::{ed25519, Signature as _, Verifier as _};

use wasmi::{
    ExternVal, Externals, FuncInstance, FuncRef, HostError, ImportsBuilder,
    MemoryRef, ModuleImportResolver, ModuleInstance, RuntimeArgs, RuntimeValue,
    Signature, Trap, TrapKind, ValueType,
};

use crate::state::{NetworkState, Storage};

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

// Signal that the contract finished execution
#[derive(Debug)]
struct ContractReturn;

impl core::fmt::Display for ContractReturn {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub enum CallKind {
    Deploy,
    Call,
}

impl HostError for ContractReturn {}

struct StackFrame<'a> {
    context: H256,
    call_data: &'a mut [u8; CALL_DATA_SIZE],
    call_kind: CallKind,
    memory: MemoryRef,
}

impl<'a> StackFrame<'a> {
    fn new(
        context: H256,
        call_data: &'a mut [u8; CALL_DATA_SIZE],
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
}

pub(crate) struct CallContext<'a> {
    state: &'a mut NetworkState,
    stack: Vec<StackFrame<'a>>,
}

impl<'a> CallContext<'a> {
    pub fn new(state: &'a mut NetworkState) -> Self {
        CallContext {
            stack: vec![],
            state,
        }
    }

    pub fn call(
        &mut self,
        target: H256,
        call_data: &'a mut [u8; CALL_DATA_SIZE],
        kind: CallKind,
    ) -> Result<(), Error> {
        let imports =
            ImportsBuilder::new().with_resolver("env", &HostImportResolver);

        let bytecode = self
            .state
            .get_contract_state(&target)
            .expect("no such contract")
            .contract()
            .bytecode();
        let module = wasmi::Module::from_buffer(bytecode)?;

        let instance =
            ModuleInstance::new(&module, &imports)?.assert_no_start();

        match instance.export_by_name("memory") {
            Some(ExternVal::Memory(memref)) => self
                .stack
                .push(StackFrame::new(target, call_data, kind, memref.clone())),
            _ => bail!("no memory found"),
        }

        let name = match self.top().call_kind {
            CallKind::Deploy => "deploy",
            CallKind::Call => "call",
        };

        instance.invoke_export(name, &[], self)?;
    }

    fn data(&self) -> &[u8] {
        let top = self.top();
        top.call_data
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

    fn top_mut(&mut self) -> &mut StackFrame<'a> {
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

    fn storage(&self) -> &Storage {
        self.state
            .get_contract_state(&self.caller())
            .expect("Invalid caller")
            .storage()
    }
    fn storage_mut(&mut self) -> &mut Storage {
        let caller = self.caller().clone();
        self.state
            .get_contract_state_mut(&caller)
            .expect("Invalid caller")
            .storage_mut()
    }

    fn balance(&self) -> u128 {
        self.state
            .get_contract_state(&self.caller())
            .expect("Invalid caller")
            .balance()
    }

    fn balance_mut(&mut self) -> &mut u128 {
        let caller = self.caller().clone();
        self.state
            .get_contract_state_mut(&caller)
            .expect("Invalid caller")
            .balance_mut()
    }
}

pub(crate) struct HostImportResolver;

fn args_to_slice<'a>(
    bytes: &'a [u8],
    args_ofs: usize,
    args: &RuntimeArgs,
) -> &'a [u8] {
    let args = args.as_ref();
    let ofs: u32 = args[args_ofs].try_into().unwrap();
    let len: u32 = args[args_ofs + 1].try_into().unwrap();
    unsafe { std::slice::from_raw_parts(&bytes[ofs as usize], len as usize) }
}

#[derive(Debug)]
struct ContractPanic(String);

// for some reason the derive does not work for Display in this case.
impl std::fmt::Display for ContractPanic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ContractPanic {
    fn new(msg: &str) -> Self {
        ContractPanic(msg.into())
    }
}

impl HostError for ContractPanic {}

impl<'a> Externals for CallContext<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            ABI_PANIC => self.top().memory.with_direct_access(|a| {
                let slice = args_to_slice(a, 0, &args);
                let str = std::str::from_utf8(slice).unwrap();
                Err(Trap::new(TrapKind::Host(Box::new(ContractPanic::new(
                    str,
                )))))
            }),
            ABI_SET_STORAGE => {
                let key_ofs =
                    args.as_ref()[0].try_into::<u32>().unwrap() as usize;
                let val_ofs =
                    args.as_ref()[1].try_into::<u32>().unwrap() as usize;
                let val_len =
                    args.as_ref()[2].try_into::<u32>().unwrap() as usize;

                let mut key_buf = [0u8; STORAGE_KEY_SIZE];
                let mut val_buf = [0u8; STORAGE_VALUE_SIZE];

                self.top().memory.with_direct_access(|a| {
                    key_buf.copy_from_slice(
                        &a[key_ofs..key_ofs + STORAGE_KEY_SIZE],
                    );
                    val_buf[0..val_len]
                        .copy_from_slice(&a[val_ofs..val_ofs + val_len]);
                });
                self.storage_mut()
                    .insert(key_buf, val_buf[0..val_len].into());

                Ok(None)
            }
            // Return value indicates if the key was found or not
            ABI_GET_STORAGE => {
                // offset to where to write the value in memory
                let key_buf_ofs =
                    args.as_ref()[0].try_into::<u32>().unwrap() as usize;
                let val_buf_ofs =
                    args.as_ref()[1].try_into::<u32>().unwrap() as usize;

                let mut key_buf = [0u8; STORAGE_KEY_SIZE];

                self.top().memory.with_direct_access(|a| {
                    key_buf.copy_from_slice(
                        &a[key_buf_ofs..key_buf_ofs + STORAGE_KEY_SIZE],
                    );
                });

                match self.storage().get(&key_buf) {
                    Some(val) => {
                        let len = val.len();
                        self.top().memory.with_direct_access_mut(|a| {
                            a[val_buf_ofs..].copy_from_slice(&val)
                        });
                        Ok(Some(RuntimeValue::I32(len as i32)))
                    }
                    None => Ok(Some(RuntimeValue::I32(0))),
                }
            }
            ABI_DEBUG => {
                self.top().memory.with_direct_access(|a| {
                    let slice = args_to_slice(a, 0, &args);
                    let str = std::str::from_utf8(slice).unwrap();
                    println!("CONTRACT DEBUG: {:?}", str);
                });
                Ok(None)
            }
            ABI_CALLER => {
                let args = args.as_ref();
                let buffer_ofs = args[0].try_into::<u32>().unwrap() as usize;

                self.top().memory.with_direct_access_mut(|a| {
                    a[buffer_ofs..buffer_ofs + 32]
                        .copy_from_slice(self.caller().as_ref())
                });
                Ok(None)
            }
            ABI_SELF_HASH => {
                let args = args.as_ref();
                let buffer_ofs = args[0].try_into::<u32>().unwrap() as usize;

                self.top().memory.with_direct_access_mut(|a| {
                    a[buffer_ofs..buffer_ofs + 32]
                        .copy_from_slice(self.called().as_ref())
                });
                Ok(None)
            }
            ABI_CALL_DATA => {
                let call_data_ofs =
                    args.as_ref()[0].try_into::<u32>().unwrap() as usize;

                self.top().memory.with_direct_access_mut(|a| {
                    a[call_data_ofs..call_data_ofs + CALL_DATA_SIZE]
                        .copy_from_slice(self.data())
                });
                Ok(None)
            }
            ABI_VERIFY_ED25519_SIGNATURE => {
                let key_ptr =
                    args.as_ref()[0].try_into::<u32>().unwrap() as usize;
                let sig_ptr =
                    args.as_ref()[1].try_into::<u32>().unwrap() as usize;

                self.top().memory.with_direct_access_mut(|a| {
                    let pub_key = ed25519::PublicKey::from_bytes(
                        &a[key_ptr..key_ptr + 32],
                    )
                    .unwrap();

                    let signature = ed25519::Signature::from_bytes(
                        &a[sig_ptr..sig_ptr + 64],
                    )
                    .unwrap();

                    let data_slice = args_to_slice(a, 2, &args);

                    let verifier: signatory_dalek::Ed25519Verifier =
                        (&pub_key).into();

                    match verifier.verify(data_slice, &signature) {
                        Ok(_) => Ok(Some(RuntimeValue::I32(1))),
                        Err(_) => Ok(Some(RuntimeValue::I32(0))),
                    }
                })
            }
            ABI_CALL_CONTRACT => {
                let target_ofs =
                    args.as_ref()[0].try_into::<u32>().unwrap() as usize;
                let amount_ofs =
                    args.as_ref()[1].try_into::<u32>().unwrap() as usize;

                let mut call_buf = [0u8; CALL_DATA_SIZE];
                let mut target = H256::zero();
                let mut amount = u128::default();
                let mut data_len = 0;

                self.memory().with_direct_access(|a| {
                    target = encoding::decode(&a[target_ofs..target_ofs + 32])
                        .unwrap();
                    amount = encoding::decode(&a[amount_ofs..amount_ofs + 16])
                        .unwrap();
                    let data = args_to_slice(a, 2, &args);
                    data_len = data.len();
                    call_buf[0..data_len].copy_from_slice(data);
                });
                // assure sufficient funds are available
                if self.balance() >= amount {
                    *self.balance_mut() -= amount;
                    *self
                        .state
                        .get_contract_state_mut_or_default(&target)
                        .balance_mut() += amount;
                } else {
                    panic!("not enough funds")
                }

                if data_len > 0 {
                    //self.state.call_from_contract(target, data);

                    unimplemented!("{:?}", &call_buf[0..data_len]);
                }
                Ok(None)
            }
            ABI_BALANCE => {
                // first argument is a pointer to a 16 byte buffer
                let args = args.as_ref();
                let buffer_ofs = args[0].try_into::<u32>().unwrap() as usize;
                let balance = self.balance();

                self.memory_mut().with_direct_access_mut(|a| {
                    encoding::encode(&balance, &mut a[buffer_ofs..]).unwrap();
                });

                Ok(None)
            }
            ABI_RETURN => {
                let StackFrame {
                    ref mut memory,
                    call_data,
                    ..
                } = self.top_mut();

                // copy from memory into call_data
                memory.with_direct_access_mut(|a| {
                    let slice = args_to_slice(a, 0, &args);
                    call_data.copy_from_slice(slice);
                });

                Err(Trap::new(TrapKind::Host(Box::new(ContractReturn))))
            }
            _ => panic!("Unimplemented function at {}", index),
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
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
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
            name => unimplemented!("{:?}", name),
        }
    }
}
