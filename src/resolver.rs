use crate::host_fns::DynamicResolver;
use crate::ops::*;
use crate::VMError;

use wasmi::{
    self, FuncInstance, FuncRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Signature,
};

use crate::host_fns::{CallContext, Invoke};

macro_rules! abi_resolver {
    ( $visibility:vis $name:ident { $( $id:expr => $op:path ),* } ) => {

        #[derive(Clone, Default)]
        $visibility struct $name;

        impl ModuleImportResolver for $name {
            fn resolve_func(&self, field_name: &str, _signature: &Signature) -> Result<FuncRef, wasmi::Error>
            where $(
                $op : AbiCall<$name>,
                )*
            {
                match field_name {
                    $(
                        <$op as AbiCall<Self>>::NAME => Ok(FuncInstance::alloc_host(
                            Signature::new(<$op as AbiCall<Self>>::ARGUMENTS,
                                           <$op as AbiCall<Self>>::RETURN),
                            $id,
                        ))
                    ),*

                    ,

                    _ => panic!("invalid function name {:?}", field_name)
                }
            }
        }

        impl Invoke for $name {
            fn invoke(
                context: &mut CallContext<Self>,
                index: usize,
                args: RuntimeArgs) -> Result<Option<RuntimeValue>, VMError> {

                match index {
                    $(
                        $id => <$op as AbiCall<Self>>::call(context, args)
                    ),*

                    ,

                    _ => panic!("invalid index {:?}", index)
                }
            }
        }

        impl DynamicResolver for $name {}
    };
}

abi_resolver! {
    pub CompoundResolver {
        0 => panic::Panic,
        1 => debug::Debug,
        2 => storage::SetStorage,
        3 => storage::GetStorage,
        5 => call_data::CallData,
        7 => call_contract::CallContract,
        9 => balance::Balance,
        9 => ret::Return,
        10 => self_hash::SelfHash,
        11 => gas::Gas,
        10_000 => ed25519::Ed25519
    }
}
