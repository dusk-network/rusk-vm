use wasmi::{
    Externals, FuncInstance, FuncRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Signature, Trap, ValueType,
};

const HOST_FN_TEST: usize = 0;

pub(crate) struct HostExternals;

impl Externals for HostExternals {
    fn invoke_index(
        &mut self,
        index: usize,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            HOST_FN_TEST => Ok(Some(RuntimeValue::I32(99))),
            _ => panic!("Unimplemented function at {}", index),
        }
    }
}

impl ModuleImportResolver for HostExternals {
    fn resolve_func(
        &self,
        field_name: &str,
        signature: &Signature,
    ) -> Result<FuncRef, wasmi::Error> {
        println!("resolve_func {}, signature {:?}", field_name, signature);
        match field_name {
            "host_fn" => Ok(FuncInstance::alloc_host(
                Signature::new(&[][..], Some(ValueType::I32)),
                HOST_FN_TEST,
            )),
            _ => unimplemented!(),
        }
    }
}
