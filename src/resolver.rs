use std::marker::PhantomData;

use crate::host_fns::Resolver;
use crate::ops::*;
use crate::VMError;

use kelvin::ByteHash;
use wasmi::{
    self, FuncInstance, FuncRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Signature,
};

use crate::host_fns::{CallContext, Invoke};

macro_rules! abi_resolver {
    ( $visibility:vis $name:ident < $h:ident > { $( $id:expr => $op:path ),* } ) => {

        #[derive(Clone, Default)]
        $visibility struct $name<$h> (PhantomData<$h>);

        impl<$h: ByteHash> ModuleImportResolver for $name<$h> {
            fn resolve_func(&self, field_name: &str, _signature: &Signature) -> Result<FuncRef, wasmi::Error>
            where $(
                $op : AbiCall<$name<$h>, $h>,
                )*
            {
                match field_name {
                    $(
                        <$op as AbiCall<Self, $h>>::NAME => Ok(FuncInstance::alloc_host(
                            Signature::new(<$op as AbiCall<Self, $h>>::ARGUMENTS,
                                           <$op as AbiCall<Self, $h>>::RETURN),
                            $id,
                        ))
                    ),*

                    ,

                    _ => panic!("invalid function name {:?}", field_name)
                }
            }
        }

        impl<$h: ByteHash> Invoke<H> for $name<$h> {
            fn invoke(
                context: &mut CallContext<Self, $h>,
                index: usize,
                args: RuntimeArgs) -> Result<Option<RuntimeValue>, VMError> {

                match index {
                    $(
                        $id => <$op as AbiCall<Self, _>>::call(context, args)
                    ),*

                    ,

                    _ => panic!("invalid index {:?}", index)
                }
            }
        }

        impl<H: ByteHash> Resolver<H> for $name<$h> {}
    };
}

abi_resolver! {
    pub CompoundResolver<H> {
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
