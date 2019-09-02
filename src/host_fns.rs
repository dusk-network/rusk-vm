use std::collections::HashMap;

use wasmi::{
    Externals, FuncInstance, FuncRef, MemoryRef, ModuleImportResolver,
    RuntimeArgs, RuntimeValue, Signature, Trap, ValueType,
};

use ethereum_types::U256;

const ABI_PANIC: usize = 0;
const ABI_DEBUG: usize = 1;
const ABI_STORAGE_SET: usize = 2;

pub(crate) struct HostExternals<'a> {
    memory: MemoryRef,
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

impl<'a> HostExternals<'a> {
    pub fn new(
        memory: &MemoryRef,
        storage: &'a mut HashMap<U256, U256>,
    ) -> Self {
        HostExternals {
            memory: memory.clone(),
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

impl<'a> Externals for HostExternals<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            ABI_PANIC => {
                self.memory.with_direct_access(|a| {
                    let slice = args_to_slice(a, &args);
                    let str = std::str::from_utf8(slice).unwrap();
                    panic!("Guest script panic! {:?}", str);
                });
                unreachable!()
            }
            ABI_STORAGE_SET => {
                println!("STORAGE_SET");
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

                println!("storage state: {:?}", self.storage);

                Ok(None)
            }
            ABI_DEBUG => Ok(None),
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
                ABI_PANIC,
            )),
            "abi_set_storage" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                ABI_STORAGE_SET,
            )),
            name => unimplemented!("{:?}", name),
        }
    }
}
