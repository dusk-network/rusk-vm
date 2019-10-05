use dusk_abi::{encoding, H256};
use serde::Serialize;
use signatory::{ed25519, Signature as _, Verifier as _};

use wasmi::{
    Externals, FuncInstance, FuncRef, HostError, MemoryRef,
    ModuleImportResolver, RuntimeArgs, RuntimeValue, Signature, Trap, TrapKind,
    ValueType,
};

use crate::interfaces::ContractCall;
use crate::state::ContractState;

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

// Signal that the contract finished execution
#[derive(Debug)]
struct ContractReturn;

impl core::fmt::Display for ContractReturn {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for ContractReturn {}

pub(crate) struct CallContext<'a, C, R> {
    memory: MemoryRef,
    caller: H256,
    state: &'a mut ContractState,
    call: &'a mut ContractCall<C, R>,
}

// TODO: remove this
trait FromPtr {
    unsafe fn from_ptr(ptr: &u8) -> Self;
}

impl FromPtr for H256 {
    unsafe fn from_ptr(ptr: &u8) -> Self {
        let mut digest = H256::zero();
        digest
            .as_mut()
            .copy_from_slice(std::slice::from_raw_parts(ptr, 32));
        digest
    }
}

impl<'a, C, R> CallContext<'a, C, R> {
    pub fn new(
        memory: &MemoryRef,
        state: &'a mut ContractState,
        caller: H256,
        call: &'a mut ContractCall<C, R>,
    ) -> Self {
        CallContext {
            memory: memory.clone(),
            caller,
            state,
            call,
        }
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

fn args_to_slice_mut<'a>(
    bytes: &'a mut [u8],
    args: &RuntimeArgs,
) -> &'a mut [u8] {
    let args = args.as_ref();
    let ofs: u32 = args[0].try_into().unwrap();
    let len: u32 = args[1].try_into().unwrap();
    unsafe {
        std::slice::from_raw_parts_mut(&mut bytes[ofs as usize], len as usize)
    }
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

impl<'a, C: Serialize, R> Externals for CallContext<'a, C, R> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            ABI_PANIC => self.memory.with_direct_access(|a| {
                let slice = args_to_slice(a, 0, &args);
                let str = std::str::from_utf8(slice).unwrap();
                Err(Trap::new(TrapKind::Host(Box::new(ContractPanic::new(
                    str,
                )))))
            }),
            ABI_SET_STORAGE => {
                let (key, val) = self.memory.with_direct_access(|a| {
                    (
                        args_to_slice(a, 0, &args).into(),
                        args_to_slice(a, 2, &args).into(),
                    )
                });
                self.state.storage_mut().insert(key, val);

                Ok(None)
            }
            ABI_GET_STORAGE => self.memory.with_direct_access_mut(|a| {
                let key = args_to_slice(a, 0, &args);
                let val_buf =
                    args.as_ref()[2].try_into::<u32>().unwrap() as usize;

                match self.state.storage().get(key) {
                    Some(ref value) => {
                        a[val_buf..val_buf + value.len()]
                            .copy_from_slice(value);
                        Ok(Some(RuntimeValue::I32(1)))
                    }
                    None => Ok(Some(RuntimeValue::I32(0))),
                }
            }),
            ABI_DEBUG => {
                self.memory.with_direct_access(|a| {
                    let slice = args_to_slice(a, 0, &args);
                    let str = std::str::from_utf8(slice).unwrap();
                    println!("CONTRACT DEBUG: {:?}", str);
                });
                Ok(None)
            }
            ABI_CALLER => {
                let args = args.as_ref();
                let buffer_ofs = args[0].try_into::<u32>().unwrap() as usize;

                self.memory.with_direct_access_mut(|a| {
                    a[buffer_ofs..buffer_ofs + 32]
                        .copy_from_slice(self.caller.as_ref())
                });
                Ok(None)
            }
            ABI_CALL_DATA => {
                self.memory.with_direct_access_mut(|a| {
                    let slice = args_to_slice_mut(a, &args);
                    let len = self.call.data().len();
                    slice[0..len].copy_from_slice(self.call.data())
                });
                Ok(None)
            }
            ABI_VERIFY_ED25519_SIGNATURE => {
                let key_ptr =
                    args.as_ref()[0].try_into::<u32>().unwrap() as usize;
                let sig_ptr =
                    args.as_ref()[1].try_into::<u32>().unwrap() as usize;

                self.memory.with_direct_access_mut(|a| {
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
                //
                unimplemented!("got herez")
            }
            ABI_BALANCE => {
                // first argument is a pointer to a 16 byte buffer
                let args = args.as_ref();
                let buffer_ofs = args[0].try_into::<u32>().unwrap() as usize;

                self.memory.with_direct_access_mut(|a| {
                    encoding::encode(
                        &self.state.balance(),
                        &mut a[buffer_ofs..],
                    )
                    .unwrap();
                });

                Ok(None)
            }
            ABI_RETURN => {
                // first argument is a pointer to a `MAX_CALL_DATA_SIZE` byte buffer
                let args = args.as_ref();
                let buffer_ofs = args[0].try_into::<u32>().unwrap() as usize;

                // split self to make borrow checker happy
                let CallContext {
                    ref mut memory,
                    ref mut call,
                    ..
                } = self;

                // copy from memory into call_data
                memory.with_direct_access_mut(|a| {
                    call.data_mut()[..].copy_from_slice(
                        &a[buffer_ofs
                            ..buffer_ofs + dusk_abi::MAX_CALL_DATA_SIZE],
                    );
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
                    &[
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                        ValueType::I32,
                    ][..],
                    None,
                ),
                ABI_SET_STORAGE,
            )),
            "get_storage" => Ok(FuncInstance::alloc_host(
                Signature::new(
                    &[ValueType::I32, ValueType::I32, ValueType::I32][..],
                    Some(ValueType::I32),
                ),
                ABI_GET_STORAGE,
            )),
            "caller" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                ABI_CALLER,
            )),
            "call_data" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
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
            name => unimplemented!("{:?}", name),
        }
    }
}
