use std::collections::HashMap;

use ethereum_types::U256;
use failure::format_err;
use wasmi::{
    Externals, FuncInstance, FuncRef, HostError, MemoryRef,
    ModuleImportResolver, RuntimeArgs, RuntimeValue, Signature, Trap, TrapKind,
    ValueType,
};

const ABI_PANIC: usize = 0;
const ABI_DEBUG: usize = 1;
const ABI_STORAGE_SET: usize = 2;
#[allow(unused)]
const ABI_STORAGE_GET: usize = 3;
const ABI_CALLER: usize = 4;

pub(crate) struct CallContext<'a> {
    memory: MemoryRef,
    caller: U256,
    storage: &'a mut HashMap<U256, U256>,
}

trait FromPtr {
    unsafe fn from_ptr(ptr: &u8) -> Self;
}

impl FromPtr for U256 {
    unsafe fn from_ptr(ptr: &u8) -> Self {
        let slice = std::slice::from_raw_parts(ptr, 32);
        U256::from_little_endian(slice)
    }
}

impl<'a> CallContext<'a> {
    pub fn new(
        memory: &MemoryRef,
        caller: U256,
        storage: &'a mut HashMap<U256, U256>,
    ) -> Self {
        CallContext {
            memory: memory.clone(),
            caller,
            storage,
        }
    }
}

pub(crate) struct HostImportResolver;

fn args_to_slice<'a>(bytes: &'a [u8], args: &RuntimeArgs) -> &'a [u8] {
    let args = args.as_ref();
    let ofs: u32 = args[0].try_into().unwrap();
    let len: u32 = args[1].try_into().unwrap();
    unsafe { std::slice::from_raw_parts(&bytes[ofs as usize], len as usize) }
}

#[derive(Debug)]
struct ContractPanic(String);

// for some reason the derive does not work for Display in this case.
impl std::fmt::Display for ContractPanic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
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
            ABI_PANIC => self.memory.with_direct_access(|a| {
                let slice = args_to_slice(a, &args);
                let str = std::str::from_utf8(slice).unwrap();
                println!("CONTRACT PANIC: {:?}", str);
                Err(Trap::new(TrapKind::Host(Box::new(ContractPanic::new(
                    str,
                )))))
            }),
            ABI_STORAGE_SET => {
                let args = args.as_ref();
                let (key, val) = self.memory.with_direct_access(|a| {
                    let key_ptr = args[0].try_into::<u32>().unwrap() as usize;
                    let val_ptr = args[1].try_into::<u32>().unwrap() as usize;
                    unsafe {
                        (
                            U256::from_ptr(&a[key_ptr]),
                            U256::from_ptr(&a[val_ptr]),
                        )
                    }
                });
                self.storage.insert(key, val);

                println!("storage updated to {:?}", self.storage);

                Ok(None)
            }
            ABI_DEBUG => {
                self.memory.with_direct_access(|a| {
                    let slice = args_to_slice(a, &args);
                    let str = std::str::from_utf8(slice).unwrap();
                    println!("CONTRACT DEBUG: {:?}", str);
                });
                Ok(None)
            }
            ABI_CALLER => {
                let args = args.as_ref();
                let buffer_ofs = args[0].try_into::<u32>().unwrap() as usize;

                self.memory.with_direct_access_mut(|a| {
                    self.caller
                        .to_big_endian(&mut a[buffer_ofs..buffer_ofs + 32]);
                });
                Ok(None)
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
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                ABI_STORAGE_SET,
            )),
            "caller" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                ABI_CALLER,
            )),
            name => unimplemented!("{:?}", name),
        }
    }
}
