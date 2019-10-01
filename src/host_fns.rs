use std::collections::HashMap;

use dusk_abi::H256;

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
const ABI_CALL_DATA: usize = 5;

pub(crate) struct CallContext<'a> {
    memory: MemoryRef,
    caller: &'a H256,
    storage: &'a mut HashMap<H256, H256>,
    call_data: &'a [u8],
}

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

impl<'a> CallContext<'a> {
    pub fn new(
        memory: &MemoryRef,
        caller: &'a H256,
        storage: &'a mut HashMap<H256, H256>,
        call_data: &'a [u8],
    ) -> Self {
        CallContext {
            memory: memory.clone(),
            caller,
            storage,
            call_data,
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
                            H256::from_ptr(&a[key_ptr]),
                            H256::from_ptr(&a[val_ptr]),
                        )
                    }
                });
                self.storage.insert(key, val);
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
                    a[buffer_ofs..buffer_ofs + 32]
                        .copy_from_slice(self.caller.as_ref())

                    // self.caller
                    //     .to_big_endian(&mut a[buffer_ofs..buffer_ofs + 32]);
                });
                Ok(None)
            }
            ABI_CALL_DATA => {
                self.memory.with_direct_access_mut(|a| {
                    let slice = args_to_slice_mut(a, &args);
                    let len = self.call_data.len();
                    println!("len {}", len);
                    println!("slice_len {}", slice.len());
                    slice[0..len].copy_from_slice(self.call_data)
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
            "call_data" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                ABI_CALL_DATA,
            )),
            name => unimplemented!("{:?}", name),
        }
    }
}
