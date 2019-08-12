use wasmi::{
    Externals, FuncInstance, FuncRef, MemoryRef, ModuleImportResolver,
    RuntimeArgs, RuntimeValue, Signature, Trap, ValueType,
};

const ABI_PANIC: usize = 1;

pub(crate) struct HostExternals(pub MemoryRef);
pub(crate) struct HostImportResolver;

fn args_to_slice<'a>(bytes: &'a [u8], args: &RuntimeArgs) -> &'a [u8] {
    let args = args.as_ref();
    let ofs: u32 = args[0].try_into().unwrap();
    let len: u32 = args[1].try_into().unwrap();
    unsafe { std::slice::from_raw_parts(&bytes[ofs as usize], len as usize) }
}

impl Externals for HostExternals {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            ABI_PANIC => {
                self.0.with_direct_access(|a| {
                    let slice = args_to_slice(a, &args);
                    let str = std::str::from_utf8(slice).unwrap();
                    panic!("Guest script panic! {:?}", str);
                });
                unreachable!()
            }
            // ABI_DEBUG => {
            //     println!("abi_debug called with {:?}", args);
            //     Ok(None)
            // }
            _ => panic!("Unimplemented function at {}", index),
        }
    }
}

impl ModuleImportResolver for HostImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        signature: &Signature,
    ) -> Result<FuncRef, wasmi::Error> {
        println!("resolve_func {}, signature {:?}", field_name, signature);
        match field_name {
            "panic" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                ABI_PANIC,
            )),
            "debug" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                ABI_PANIC,
            )),
            name => unimplemented!("{:?}", name),
        }
    }
}
