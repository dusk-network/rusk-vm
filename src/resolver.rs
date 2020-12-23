// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::marker::PhantomData;

use crate::call_context::Resolver;
use crate::ops::*;
use crate::VMError;

use wasmi::{
    self, FuncInstance, FuncRef, ModuleImportResolver, RuntimeArgs,
    RuntimeValue, Signature,
};

use crate::call_context::{CallContext, Invoke};

macro_rules! abi_resolver {
    ( $visibility:vis $name:ident < $s:ident > { $( $id:expr, $op_name:expr => $op:path ),* } ) => {

        #[doc(hidden)]
        #[derive(Clone, Default)]
        $visibility struct $name<$s> (PhantomData<$s>);

        use canonical::Store;

        impl<$s: Store> ModuleImportResolver for $name<$s> {
            fn resolve_func(&self, field_name: &str, _signature: &Signature) -> Result<FuncRef, wasmi::Error>
            where $(
                $op : AbiCall<$name<$s>, $s>,
                )*
            {
                match field_name {
                    $(
                        $op_name => Ok(FuncInstance::alloc_host(
                            Signature::new(<$op as AbiCall<Self, $s>>::ARGUMENTS,
                                           <$op as AbiCall<Self, $s>>::RETURN),
                            $id,
                        ))
                    ),*

                    ,

                    _ => panic!("invalid function name {:?}", field_name)
                }
            }
        }

        impl<$s: Store> Invoke<S> for $name<$s> {
            fn invoke(
                context: &mut CallContext<Self, $s>,
                index: usize,
                args: RuntimeArgs) -> Result<Option<RuntimeValue>, VMError<S>> {

                match index {
                    $(
                        $id => <$op as AbiCall<Self, _>>::call(context, args)
                    ),*

                    ,

                    _ => panic!("invalid index {:?}", index)
                }
            }
        }

        impl<S: Store> Resolver<S> for $name<$s> {}
    };
}

abi_resolver! {
    pub CompoundResolver<S> {
        0, "sig" => panic::Panic,
        1, "debug" => debug::Debug,
        6, "query" => query::ExecuteQuery,
        7, "transact" => transact::ApplyTransaction,
        8, "ret" => ret::Return,
        9, "self_id" => self_id::SelfId,
        10, "gas" => gas::Gas
    }
}
